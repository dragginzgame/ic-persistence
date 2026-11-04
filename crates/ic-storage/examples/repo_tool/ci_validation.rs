//! CI observations reuse release path ownership; they never create release authority.
use super::*;

#[derive(Debug, Serialize)]
pub(super) struct Change {
    pub comparison_base: String,
    pub changed: Vec<String>,
}

fn commit(value: &str) -> bool {
    matches!(value.len(), 40 | 64)
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

// Rename detection is disabled: both old and new paths must fit the same owner.
// Executable, symlink, submodule, unmerged and unexpected records stay full.
pub(super) fn changed_paths(raw: &str) -> Result<Vec<String>> {
    require(raw.ends_with('\0'), "missing terminated CI diff")?;
    let fields: Vec<_> = raw[..raw.len() - 1].split('\0').collect();
    require(fields.len().is_multiple_of(2), "incomplete CI diff")?;
    let mut changed = Vec::new();
    for pair in fields.as_chunks::<2>().0 {
        let parts: Vec<_> = pair[0].split_whitespace().collect();
        require(
            parts.len() == 5
                && matches!(
                    (parts[0], parts[1], parts[4]),
                    (":000000", "100644", "A")
                        | (":100644", "000000", "D")
                        | (":100644", "100644", "M")
                )
                && commit(parts[2])
                && commit(parts[3]),
            "non-regular or unknown CI change",
        )?;
        let path = pair[1];
        require(
            !path.is_empty()
                && !path.bytes().any(|b| b.is_ascii_control())
                && Path::new(path)
                    .components()
                    .all(|part| matches!(part, std::path::Component::Normal(_))),
            "unsafe CI diff path",
        )?;
        changed.push(path.to_owned());
    }
    Ok(changed)
}

pub(super) fn selected_group(changed: &[String], lock: &str) -> Result<Option<&'static str>> {
    let mut selected = None;
    for group in ["metrics-storage", "logging-storage"] {
        if !changed.is_empty()
            && changed
                .iter()
                .all(|path| validation_plan::adapter_local(group, path))
            && validation_plan::independent_adapter(lock, group)?
        {
            // Shared handoff-only changes do not identify an adapter owner.
            if selected.is_some() {
                return Ok(None);
            }
            selected = Some(group);
        }
    }
    Ok(selected)
}

impl<E: Execute> Repository<E> {
    pub(super) fn ci_change(
        &self,
        event: &str,
        base: &str,
        head: &str,
        reference: &str,
    ) -> Result<Change> {
        require(
            event == "pull_request",
            "pushes and other events keep repository-wide CI",
        )?;
        require(commit(base) && commit(head), "missing CI commit identities")?;
        require(
            reference.starts_with("refs/heads/")
                && reference.len() > 11
                && !reference
                    .bytes()
                    .any(|b| b.is_ascii_whitespace() || b.is_ascii_control()),
            "unknown CI reference",
        )?;
        self.output("git", &["cat-file", "-e", &format!("{base}^{{commit}}")])?;
        let comparison_base = self.git(&["merge-base", base, head])?;
        require(
            commit(&comparison_base),
            "missing or ambiguous CI merge base",
        )?;
        let raw = self.output(
            "git",
            &[
                "diff",
                "--raw",
                "-z",
                "--no-renames",
                "--no-abbrev",
                "--ignore-submodules=none",
                "--no-relative",
                &comparison_base,
                head,
                "--",
            ],
        )?;
        Ok(Change {
            comparison_base,
            changed: changed_paths(&raw)?,
        })
    }

    pub(super) fn validate_ci(
        &self,
        event: &str,
        base: &str,
        head: &str,
        reference: &str,
    ) -> Result<()> {
        self.clean()?;
        require(
            commit(head) && self.git(&["rev-parse", "HEAD"])? == head,
            "CI checkout differs from tested source",
        )?;
        let change = self.ci_change(event, base, head, reference);
        let group = match &change {
            Ok(change) => selected_group(&change.changed, &self.text("Cargo.lock", Some(head))?)?,
            Err(error) => {
                eprintln!("Full CI selected: {error}");
                None
            }
        };
        let toolchain = self.toolchain()?;
        for prefix in ["target", "target/ci-plans"] {
            let path = self.root.join(prefix);
            if path.exists() {
                require(
                    path.is_dir() && !fs::symlink_metadata(&path)?.file_type().is_symlink(),
                    "redirected CI evidence parent",
                )?;
            } else {
                fs::create_dir(&path)?;
            }
        }
        let directory = format!(
            "target/ci-plans/attempt.{}.{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
        );
        fs::create_dir(self.root.join(&directory))?;
        let path = format!("{directory}/plan.json");
        let context_path = format!("{path}.context.json");
        self.exec.write_bytes(&self.root.join(&context_path), &validation_plan::canonical(&serde_json::json!({
            "schema":1, "kind":"ci-selection", "source":head, "event":event,
            "base":base, "reference":reference, "group":group,
            "observation":change.as_ref().ok(), "fallback":change.as_ref().err().map(ToString::to_string)
        }))?)?;
        eprintln!("Retained CI selection: {context_path}");
        if let Some(group) = group {
            let change = change?;
            let roster = self.admit_validation_policy(head, group)?;
            let mut inputs: Vec<_> = self
                .output("git", &["ls-tree", "-r", "--name-only", "-z", head])?
                .split('\0')
                .filter(|path| !path.is_empty())
                .map(str::to_owned)
                .collect();
            require(
                inputs.contains(&validation_plan::POLICY.to_owned())
                    && inputs.contains(&"Makefile".to_owned()),
                "incomplete CI inventory",
            )?;
            inputs.push(context_path);
            inputs.sort();
            for input in &inputs {
                validation_plan::regular_path(&self.root, input)?;
            }
            let plan = validation_plan::Plan {
                schema: 1,
                source: head.to_owned(),
                group: group.to_owned(),
                scope: "group".to_owned(),
                policy: validation_plan::POLICY.to_owned(),
                inputs,
                stages: validation_plan::stages(group, &change.changed, true, &roster),
            };
            self.exec
                .write_bytes(&self.root.join(&path), &validation_plan::canonical(&plan)?)?;
            let command = validation_run::plan_command(&toolchain, &path, head, group);
            self.run(
                &command[0],
                &command[1..].iter().map(String::as_str).collect::<Vec<_>>(),
            )?;
            // The native workflow uploads this consumer fixture's archive. Full
            // gates produce it within validate-tooling; narrow gates still owe it.
            self.run("make", &["--no-print-directory", "test-failure-evidence"])?;
        } else {
            self.run(
                "make",
                &[
                    "--no-print-directory",
                    "validate-toolchain",
                    &format!("VALIDATION_TOOLCHAIN={toolchain}"),
                ],
            )?;
        }
        require(
            self.git(&["rev-parse", "HEAD"])? == head,
            "CI source moved during validation",
        )?;
        self.clean()
    }
}
