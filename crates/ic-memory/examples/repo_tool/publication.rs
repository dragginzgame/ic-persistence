//! Registry reconciliation and coordinated publication.
use super::*;

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
        let record: serde_json::Value = serde_json::from_str(body)?;
        let record = &record["version"];
        require(
            record["crate"].as_str() == Some(name)
                && record["num"].as_str() == Some(version)
                && record["yanked"].as_bool() == Some(false)
                && record["checksum"].as_str() == Some(digest),
            "published package differs from the qualified archive or is yanked",
        )?;
        Ok(true)
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
