//! Registry reconciliation and coordinated publication.
use super::*;

// The registry may add fields. These required facts still have exact types and
// must match the independently qualified package before it counts as published.
#[derive(Deserialize)]
struct RegistryResponse {
    version: RegistryVersion,
}

#[derive(Deserialize)]
struct RegistryVersion {
    #[serde(rename = "crate")]
    name: String,
    num: String,
    yanked: bool,
    checksum: String,
}

fn registry_response(response: &str, name: &str, version: &str, digest: &str) -> Result<bool> {
    let (body, status) = response
        .rsplit_once('\n')
        .ok_or("missing registry HTTP status")?;
    if status == "404" {
        return Ok(false);
    }
    require(
        status == "200",
        &format!(
            "registry observation failed for {name} {version} (HTTP {status}); publication was not retried"
        ),
    )?;
    let record: RegistryResponse = serde_json::from_str(body)?;
    let record = record.version;
    require(
        record.name == name && record.num == version && !record.yanked && record.checksum == digest,
        "published package differs from the qualified archive or is yanked",
    )?;
    Ok(true)
}

impl<E: Execute> Repository<E> {
    fn registry_contains(&self, name: &str, version: &str, digest: &str) -> Result<bool> {
        require(packages().contains(&name), "unknown publication package")?;
        self.version_parts(version)?;
        let response = self.output(
            "curl",
            &[
                "--silent",
                "--show-error",
                "--location",
                "--user-agent",
                "ic-persistence-release (https://github.com/dragginzgame/ic-persistence)",
                "--max-time",
                "60",
                "--write-out",
                "\n%{http_code}",
                &format!("https://crates.io/api/v1/crates/{name}/{version}"),
            ],
        )?;
        registry_response(&response, name, version, digest)
    }

    pub(super) fn publish(&self, dry_run: bool) -> Result<()> {
        let release = self.release_commit()?;
        let version = &release.selection.version;
        let head = &release.commit;
        let tag = self.check_tag(version, head)?;
        let evidence = self.verify_evidence(&release.selection, head, Final, Working)?;
        // Observe every package before any effect. A conflict in a later package
        // must not permit an earlier upload; dependencies publish first.
        let packages = evidence
            .archives()
            .map(|(name, digest)| {
                let published = !dry_run && self.registry_contains(name, version, digest)?;
                Ok((name, digest, published))
            })
            .collect::<Result<Vec<_>>>()?;
        for (name, digest, exists) in packages {
            require(
                self.release_commit()? == release,
                "release changed before publication",
            )?;
            require(
                self.check_tag(version, head)? == tag,
                "release tag changed before publication",
            )?;
            if exists {
                println!("Already published and checksum verified: {name} {version}");
                continue;
            }
            let parent = self.target()?.join("release-validation/publication");
            fs::create_dir_all(&parent)?;
            let attempt = parent.join(format!(
                "{name}.{}.{}",
                std::process::id(),
                SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
            ));
            // Shared owns per-attempt locking, stdout/stderr and both Cargo and
            // logger statuses. This adapter owns package order and reconciliation.
            self.run(
                "env",
                &[
                    &format!("RUSTUP_TOOLCHAIN={}", self.toolchain()?),
                    "bash",
                    "scripts/ci/run-cargo-publication.sh",
                    if dry_run { "check" } else { "publish" },
                    attempt.to_str().ok_or("non-UTF-8 publication path")?,
                    head,
                    "crates-io",
                    "--package",
                    &format!("{name}@{version}"),
                    "--no-verify",
                ],
            )?;
            require(
                self.release_commit()? == release,
                "release changed during publication",
            )?;
            require(
                self.check_tag(version, head)? == tag,
                "release tag changed during publication",
            )?;
            if !dry_run {
                require(
                    self.registry_contains(name, version, digest)?,
                    "publication is not yet visible; rerun publish later to reconcile checksums before continuing",
                )?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record() -> serde_json::Value {
        serde_json::json!({
            "version": {
                "crate": "ic-memory", "num": "0.13.0", "yanked": false, "checksum": "digest",
                "additional_registry_field": true,
            },
            "additional_response_field": [],
        })
    }

    fn observe(record: &serde_json::Value) -> Result<bool> {
        registry_response(&format!("{record}\n200"), "ic-memory", "0.13.0", "digest")
    }

    #[test]
    fn registry_identity_requires_matching_unyanked_bytes() {
        assert!(observe(&record()).unwrap());
        for (field, value) in [
            ("crate", serde_json::json!("ic-storage")),
            ("num", serde_json::json!("0.13.1")),
            ("checksum", serde_json::json!("different")),
            ("yanked", serde_json::json!(true)),
        ] {
            let mut record = record();
            record["version"][field] = value;
            assert!(observe(&record).is_err());
        }
    }

    #[test]
    fn registry_facts_must_be_present_and_typed() {
        for field in ["crate", "num", "checksum", "yanked"] {
            let mut missing = record();
            missing["version"].as_object_mut().unwrap().remove(field);
            assert!(observe(&missing).is_err());
            let mut wrong_type = record();
            wrong_type["version"][field] = serde_json::json!([]);
            assert!(observe(&wrong_type).is_err());
        }
        for record in [serde_json::json!({}), serde_json::json!({"version": null})] {
            assert!(observe(&record).is_err());
        }
    }

    #[test]
    fn only_an_explicit_not_found_status_means_absence() {
        assert!(!registry_response("not-json\n404", "ic-memory", "0.13.0", "digest").unwrap());
        for response in ["", "{}", "{}\n403", "{}\n503", "not-json\n200", "{}\n200 "] {
            assert!(registry_response(response, "ic-memory", "0.13.0", "digest").is_err());
        }
    }
}
