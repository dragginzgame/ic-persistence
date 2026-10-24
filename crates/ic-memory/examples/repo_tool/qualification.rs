//! Validation and package receipts bound to selected source.
use super::*;

impl<E: Execute> Repository<E> {
    pub(super) fn validation_path(&self, version: &str) -> Result<PathBuf> {
        self.version_parts(version)?;
        Ok(self
            .target()?
            .join("release-validation")
            .join(format!("{version}-validated.json")))
    }

    pub(super) fn validation_command(&self) -> Result<Vec<String>> {
        Ok(vec![
            "make".to_owned(),
            "--no-print-directory".to_owned(),
            "validate".to_owned(),
            format!("VALIDATION_TOOLCHAIN={}", self.toolchain()?),
        ])
    }

    pub(super) fn run_command(&self, command: &[String]) -> Result<()> {
        self.run(
            &command[0],
            &command[1..].iter().map(String::as_str).collect::<Vec<_>>(),
        )
    }

    pub(super) fn preflight(&self, selection: &ReleaseSelection) -> Result<()> {
        self.configuration()?;
        self.check_selection(selection)?;
        require(
            self.git(&["rev-parse", "HEAD"])? == selection.source,
            "source HEAD differs from the saved release",
        )?;
        require(
            self.version(None)? == selection.previous,
            "preflight requires the saved base version",
        )?;
        self.check_surfaces(selection, None, true)?;
        self.check_index(selection, &self.surfaces(selection)?)?;
        let lock = fs::read(self.root.join("Cargo.lock"))?;
        let interrupted = lock != self.text("Cargo.lock", Some(&selection.source))?.as_bytes();
        if interrupted {
            // The exact candidate lock can precede the manifest after an
            // interrupted write. Reuse source-bound successful validation;
            // locked Cargo fetching cannot reconcile that temporary mismatch.
            self.validated(selection)?;
        } else {
            check_lock_update(
                &lock,
                std::str::from_utf8(&lock)?,
                &selection.previous,
                &selection.previous,
            )?;
        }
        self.remote_ready(selection)?;
        // Remote inspection and offline cache verification cannot change the
        // tracked lockfile whose selection will qualify this attempt.
        if !interrupted {
            self.run(
                "cargo",
                &[
                    &format!("+{}", self.toolchain()?),
                    "fetch",
                    "--locked",
                    "--offline",
                ],
            )?;
            // Only coherent, admitted release sources may prepare tools. The
            // complete common selection is required by the offline full gate.
            self.run("make", &["--no-print-directory", "install-tools"])?;
        }
        self.run(
            "make",
            &["--no-print-directory", "tools-check", "format-tools-check"],
        )?;
        require(
            fs::read(self.root.join("Cargo.lock"))? == lock,
            "dependency selection changed during preflight",
        )?;
        require(
            self.git(&["rev-parse", "HEAD"])? == selection.source,
            "source changed during preflight",
        )?;
        self.check_surfaces(selection, None, true)?;
        self.check_index(selection, &self.surfaces(selection)?)
    }

    pub(super) fn verify(&self, selection: &ReleaseSelection) -> Result<()> {
        self.run("bash", &["scripts/ci/check-make-execution.sh"])?;
        let configuration = self.configuration()?;
        self.check_selection(selection)?;
        self.clean()?;
        require(
            self.git(&["rev-parse", "HEAD"])? == selection.source
                && self.version(None)? == selection.previous,
            "validation requires the saved source and base version",
        )?;
        let lock = fs::read(self.root.join("Cargo.lock"))?;
        check_lock_update(
            &lock,
            std::str::from_utf8(&lock)?,
            &selection.previous,
            &selection.previous,
        )?;
        let evidence = ValidationEvidence {
            selection: selection.clone(),
            lock,
            identities: self.identities()?,
            configuration,
            command: self.validation_command()?,
        };
        self.run_command(&evidence.command)?;
        self.clean()?;
        require(
            self.git(&["rev-parse", "HEAD"])? == selection.source,
            "source changed during validation",
        )?;
        self.verify_inputs(&evidence, false)?;
        let path = self.validation_path(&selection.version)?;
        let prepared_path = self.receipt_path(&selection.version, true)?;
        let stale_prepared = if prepared_path.exists() {
            let prepared: PackageEvidence = serde_json::from_slice(&fs::read(&prepared_path)?)?;
            let previous = &prepared.validation.selection;
            if previous.source == selection.source {
                None
            } else {
                require(
                    previous.kind == selection.kind
                        && previous.previous == selection.previous
                        && previous.version == selection.version
                        && previous.remote == selection.remote
                        && previous.branch == selection.branch,
                    "superseded preparation identifies another release or destination",
                )?;
                require(
                    prepared.package_head == previous.source,
                    "superseded package evidence differs from its preparation source",
                )?;
                self.git(&[
                    "merge-base",
                    "--is-ancestor",
                    &previous.source,
                    &selection.source,
                ])?;
                Some(prepared_path)
            }
        } else {
            None
        };
        for (receipt, kind) in std::iter::once((&path, "validated"))
            .chain(stale_prepared.as_ref().map(|receipt| (receipt, "prepared")))
            .filter(|(receipt, _)| receipt.exists())
        {
            let retained = receipt
                .parent()
                .ok_or("validation path has no parent")?
                .join("attempts")
                .join(format!(
                    "{}-{kind}.{}.{}.json",
                    selection.version,
                    std::process::id(),
                    SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
                ));
            self.exec.write_bytes(&retained, &fs::read(receipt)?)?;
        }
        // Fresh successful validation supersedes only an admitted older source.
        // Preserve both receipts first; unchanged-source recovery keeps its active
        // prepared receipt and existing integrity checks.
        if let Some(receipt) = stale_prepared {
            fs::remove_file(receipt)?;
        }
        self.exec.write_json(&path, &evidence)
    }

    pub(super) fn verify_inputs(&self, evidence: &ValidationEvidence, updated: bool) -> Result<()> {
        require(
            evidence.lock
                == self
                    .text("Cargo.lock", Some(&evidence.selection.source))?
                    .as_bytes(),
            "validation lockfile differs from the saved source",
        )?;
        require(
            self.configuration()? == evidence.configuration,
            "build configuration differs from release validation",
        )?;
        require(
            self.identities()? == evidence.identities,
            "compiler identities differ from release validation",
        )?;
        require(
            evidence.command == self.validation_command()?,
            "validation evidence has unexpected commands",
        )?;
        self.check_selection(&evidence.selection)?;
        let current = fs::read(self.root.join("Cargo.lock"))?;
        if current != evidence.lock {
            require(updated, "selected lockfile differs from release validation")?;
            check_lock_update(
                &evidence.lock,
                std::str::from_utf8(&current)?,
                &evidence.selection.previous,
                &evidence.selection.version,
            )?;
        }
        Ok(())
    }

    pub(super) fn validated(&self, selection: &ReleaseSelection) -> Result<ValidationEvidence> {
        let evidence: ValidationEvidence =
            serde_json::from_slice(&fs::read(self.validation_path(&selection.version)?)?)?;
        require(
            evidence.selection == *selection,
            "validation identifies another release intent",
        )?;
        self.verify_inputs(&evidence, true)?;
        Ok(evidence)
    }

    pub(super) fn preparation_commands(&self) -> Result<Vec<Vec<String>>> {
        let pin = format!("+{}", self.toolchain()?);
        Ok(vec![
            vec![
                "cargo".to_owned(),
                pin.clone(),
                "update".to_owned(),
                "--workspace".to_owned(),
                "--offline".to_owned(),
            ],
            vec![
                "make".to_owned(),
                "--no-print-directory".to_owned(),
                "fmt-check".to_owned(),
                format!("VALIDATION_TOOLCHAIN={}", self.toolchain()?),
            ],
            vec![
                "bash".to_owned(),
                "scripts/ci/package-workspace.sh".to_owned(),
                pin.trim_start_matches('+').to_owned(),
                "--allow-dirty".to_owned(),
            ],
        ])
    }

    pub(super) fn final_package_command(&self) -> Result<Vec<String>> {
        Ok(vec![
            "bash".to_owned(),
            "scripts/ci/package-workspace.sh".to_owned(),
            self.toolchain()?,
        ])
    }

    pub(super) fn write_evidence(&self, evidence: &PackageEvidence, prepared: bool) -> Result<()> {
        self.exec.write_json(
            &self.receipt_path(&evidence.validation.selection.version, prepared)?,
            evidence,
        )
    }

    pub(super) fn prepare_selected(&self, selection: &ReleaseSelection) -> Result<()> {
        self.validated(selection)?;
        require(
            self.git(&["rev-parse", "HEAD"])? == selection.source,
            "source changed before preparation",
        )?;
        self.check_surfaces(selection, None, true)?;
        let expected = self.surfaces(selection)?;
        let mut backups = BTreeMap::new();
        for path in expected.keys() {
            backups.insert(path.clone(), fs::read(self.root.join(path))?);
        }
        let result = (|| -> Result<()> {
            // Cargo.toml is last: observing the candidate means all other source
            // metadata, including the selected root lock, is complete. The
            // prepared check can then finish offline workspace verification and
            // package qualification after an interrupted operation.
            for (path, text) in expected
                .iter()
                .filter(|(path, _)| path.as_str() != "Cargo.toml")
            {
                if backups[path].as_slice() != text.as_bytes() {
                    self.exec
                        .write_bytes(&self.root.join(path), text.as_bytes())?;
                }
            }
            if backups["Cargo.toml"].as_slice() != expected["Cargo.toml"].as_bytes() {
                self.exec.write_bytes(
                    &self.root.join("Cargo.toml"),
                    expected["Cargo.toml"].as_bytes(),
                )?;
            }
            self.prepared(selection)?;
            Ok(())
        })();
        if let Err(source) = result {
            // Durable publication may report a parent-sync error after rename.
            // Inspect current bytes before restoring only owned replacements.
            // A failed restoration must not hide the preparation error or stop
            // restoration of other independent, owned metadata files.
            let mut rollback_errors = Vec::new();
            for (path, bytes) in backups {
                let restored = (|| -> Result<()> {
                    let current = fs::read(self.root.join(&path))?;
                    if current == bytes {
                        // No replacement needs undoing. Preserve the existing
                        // file identity and permissions without another write.
                        return Ok(());
                    }
                    let owned = if path == "Cargo.lock" {
                        std::str::from_utf8(&current).is_ok_and(|text| {
                            check_lock_update(&bytes, text, &selection.previous, &selection.version)
                                .is_ok()
                        })
                    } else {
                        current == expected[&path].as_bytes()
                    };
                    if owned {
                        self.exec.write_bytes(&self.root.join(&path), &bytes)?;
                    }
                    Ok(())
                })();
                if let Err(error) = restored {
                    rollback_errors.push((path, error));
                }
            }
            if rollback_errors.is_empty() {
                Err(source)
            } else {
                Err(PreparationError {
                    source,
                    rollback_errors,
                }
                .into())
            }
        } else {
            result
        }
    }

    pub(super) fn verify_evidence(
        &self,
        selection: &ReleaseSelection,
        head: &str,
        prepared: bool,
        working: bool,
    ) -> Result<PackageEvidence> {
        let evidence: PackageEvidence =
            serde_json::from_slice(&fs::read(self.receipt_path(&selection.version, prepared)?)?)?;
        require(
            evidence.validation.selection == *selection,
            "package evidence identifies another release intent",
        )?;
        self.verify_inputs(&evidence.validation, true)?;
        require(
            Self::sha256(&self.root.join("Cargo.lock"))? == evidence.lock_sha256,
            "selected lockfile differs from package qualification",
        )?;
        require(
            evidence.package_head == head,
            "package evidence identifies another Git HEAD",
        )?;
        for (package, digest) in [
            ("ic-memory", &evidence.package_sha256),
            ("ic-storage", &evidence.storage_sha256),
        ] {
            require(
                Self::sha256(&self.retained_package(digest)?)? == *digest,
                "retained archive differs from release validation",
            )?;
            if working {
                require(
                    Self::sha256(&self.package_path(package, &selection.version)?)? == *digest,
                    "working package differs from release validation",
                )?;
            }
        }
        let mut commands = self.preparation_commands()?;
        if !prepared {
            commands.push(self.final_package_command()?);
        }
        require(
            evidence.commands == commands,
            "package evidence has unexpected commands",
        )?;
        Ok(evidence)
    }

    pub(super) fn prepared(&self, selection: &ReleaseSelection) -> Result<PackageEvidence> {
        require(
            self.git(&["rev-parse", "HEAD"])? == selection.source,
            "prepared source changed",
        )?;
        let validation = self.validated(selection)?;
        self.check_surfaces(selection, None, false)?;
        if self.receipt_path(&selection.version, true)?.exists() {
            // Existing evidence is checked, never silently repaired or replaced.
            return self.verify_evidence(selection, &selection.source, true, true);
        }
        let commands = self.preparation_commands()?;
        self.run_command(&commands[0])?;
        check_lock_update(
            &validation.lock,
            &fs::read_to_string(self.root.join("Cargo.lock"))?,
            &selection.previous,
            &selection.version,
        )?;
        self.run_command(&commands[1])?;
        let lock_sha256 = Self::sha256(&self.root.join("Cargo.lock"))?;
        self.run_command(&commands[2])?;
        self.check_surfaces(selection, None, false)?;
        require(
            self.git(&["rev-parse", "HEAD"])? == selection.source,
            "source changed during preparation",
        )?;
        self.verify_inputs(&validation, true)?;
        require(
            Self::sha256(&self.root.join("Cargo.lock"))? == lock_sha256,
            "packaging changed selected dependencies",
        )?;
        let evidence = PackageEvidence {
            validation,
            package_head: selection.source.clone(),
            lock_sha256,
            package_sha256: self.record_package("ic-memory", &selection.version)?,
            storage_sha256: self.record_package("ic-storage", &selection.version)?,
            commands,
        };
        self.write_evidence(&evidence, true)?;
        Ok(evidence)
    }
}
