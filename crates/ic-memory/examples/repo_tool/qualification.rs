//! Validation and package receipts bound to selected source.
use super::*;

// Each edit keeps rollback bytes beside the exact source-derived replacement.
struct MetadataEdit<'a> {
    path: &'a str,
    expected: &'a str,
    before: Vec<u8>,
}

impl MetadataEdit<'_> {
    fn publish<E: Execute>(&self, repo: &Repository<E>) -> Result<()> {
        if self.before != self.expected.as_bytes() {
            repo.exec
                .write_bytes(&repo.root.join(self.path), self.expected.as_bytes())?;
        }
        Ok(())
    }

    fn restore<E: Execute>(
        &self,
        repo: &Repository<E>,
        selection: &ReleaseSelection,
    ) -> Result<()> {
        let current = fs::read(repo.root.join(self.path))?;
        if current == self.before {
            // Preserve file identity and permissions when no replacement needs undoing.
            return Ok(());
        }
        let owned = if self.path == "Cargo.lock" {
            std::str::from_utf8(&current).is_ok_and(|text| {
                check_lock_update(&self.before, text, &selection.previous, &selection.version)
                    .is_ok()
            })
        } else {
            current == self.expected.as_bytes()
        };
        if owned {
            repo.exec
                .write_bytes(&repo.root.join(self.path), &self.before)?;
        }
        Ok(())
    }
}

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
        let surfaces = self.surfaces(selection)?;
        self.check_surfaces(selection, &surfaces, None, true)?;
        self.check_index(selection, &surfaces)?;
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
        self.check_surfaces(selection, &surfaces, None, true)?;
        self.check_index(selection, &surfaces)
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
        let stale_prepared = self.superseded_preparation(selection)?;
        self.retain_receipt(&path, "validated", &selection.version)?;
        if let Some(receipt) = &stale_prepared {
            self.retain_receipt(receipt, "prepared", &selection.version)?;
        }
        // Fresh successful validation supersedes only an admitted older source.
        // Preserve both receipts first; unchanged-source recovery keeps its active
        // prepared receipt and existing integrity checks.
        if let Some(receipt) = stale_prepared {
            fs::remove_file(receipt)?;
        }
        self.exec.write_json(&path, &evidence)
    }

    fn superseded_preparation(&self, selection: &ReleaseSelection) -> Result<Option<PathBuf>> {
        let path = self.receipt_path(&selection.version, Prepared)?;
        if !path.exists() {
            return Ok(None);
        }
        let prepared: PackageEvidence = serde_json::from_slice(&fs::read(&path)?)?;
        let previous = &prepared.validation.selection;
        if previous.source == selection.source {
            return Ok(None);
        }
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
        Ok(Some(path))
    }

    fn retain_receipt(&self, receipt: &Path, kind: &str, version: &str) -> Result<()> {
        if receipt.exists() {
            let retained = receipt
                .parent()
                .ok_or("validation path has no parent")?
                .join("attempts")
                .join(format!(
                    "{version}-{kind}.{}.{}.json",
                    std::process::id(),
                    SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
                ));
            self.exec.write_bytes(&retained, &fs::read(receipt)?)?;
        }
        Ok(())
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

    pub(super) fn preparation_commands(&self) -> Result<[Vec<String>; 3]> {
        let pin = format!("+{}", self.toolchain()?);
        Ok([
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

    pub(super) fn write_evidence(
        &self,
        evidence: &PackageEvidence,
        stage: ReceiptStage,
    ) -> Result<()> {
        self.exec.write_json(
            &self.receipt_path(&evidence.validation.selection.version, stage)?,
            evidence,
        )
    }

    pub(super) fn prepare_selected(&self, selection: &ReleaseSelection) -> Result<()> {
        self.validated(selection)?;
        require(
            self.git(&["rev-parse", "HEAD"])? == selection.source,
            "source changed before preparation",
        )?;
        let expected = self.surfaces(selection)?;
        self.check_surfaces(selection, &expected, None, true)?;
        let edits = expected
            .iter()
            .map(|(path, surface)| {
                Ok(MetadataEdit {
                    path,
                    expected: &surface.expected,
                    before: fs::read(self.root.join(path))?,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let result = (|| -> Result<()> {
            // Publish the root manifest last: the candidate version must imply
            // that all other metadata, including its selected lock, is complete.
            for edit in edits
                .iter()
                .filter(|edit| edit.path != "Cargo.toml")
                .chain(edits.iter().filter(|edit| edit.path == "Cargo.toml"))
            {
                edit.publish(self)?;
            }
            // Source-derived candidate bytes remain the same throughout this
            // transaction. Re-admit the live HEAD, receipt and validation inputs
            // after publication, then qualify against this exact projection.
            self.qualify_prepared(self.preparation_validation(selection)?, &expected)?;
            Ok(())
        })();
        if let Err(source) = result {
            // Durable publication may report a parent-sync error after rename.
            // Inspect current bytes before restoring only owned replacements.
            // A failed restoration must not hide the preparation error or stop
            // restoration of other independent, owned metadata files.
            let mut rollback_errors = Vec::new();
            for edit in &edits {
                if let Err(error) = edit.restore(self, selection) {
                    rollback_errors.push((edit.path.to_owned(), error));
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
        stage: ReceiptStage,
        location: ArchiveLocation,
    ) -> Result<PackageEvidence> {
        let evidence: PackageEvidence =
            serde_json::from_slice(&fs::read(self.receipt_path(&selection.version, stage)?)?)?;
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
        for (package, digest) in evidence.archives() {
            require(
                Self::sha256(&self.retained_package(digest)?)? == digest,
                "retained archive differs from release validation",
            )?;
            if location == Working {
                require(
                    Self::sha256(&self.package_path(package, &selection.version)?)? == digest,
                    "working package differs from release validation",
                )?;
            }
        }
        let mut commands = self.preparation_commands()?.to_vec();
        if stage == Final {
            commands.push(self.final_package_command()?);
        }
        require(
            evidence.commands == commands,
            "package evidence has unexpected commands",
        )?;
        Ok(evidence)
    }

    pub(super) fn prepared(&self, selection: &ReleaseSelection) -> Result<PackageEvidence> {
        let validation = self.preparation_validation(selection)?;
        let surfaces = self.surfaces(selection)?;
        self.qualify_prepared(validation, &surfaces)
    }

    pub(super) fn preparation_validation(
        &self,
        selection: &ReleaseSelection,
    ) -> Result<ValidationEvidence> {
        require(
            self.git(&["rev-parse", "HEAD"])? == selection.source,
            "prepared source changed",
        )?;
        self.validated(selection)
    }

    pub(super) fn qualify_prepared(
        &self,
        validation: ValidationEvidence,
        surfaces: &BTreeMap<String, ReleaseSurface>,
    ) -> Result<PackageEvidence> {
        let selection = &validation.selection;
        self.check_surfaces(selection, surfaces, None, false)?;
        if self.receipt_path(&selection.version, Prepared)?.exists() {
            // Existing evidence is checked, never silently repaired or replaced.
            return self.verify_evidence(selection, &selection.source, Prepared, Working);
        }
        let commands = self.preparation_commands()?;
        let [refresh, formatting, packaging] = &commands;
        self.run_command(refresh)?;
        check_lock_update(
            &validation.lock,
            &fs::read_to_string(self.root.join("Cargo.lock"))?,
            &selection.previous,
            &selection.version,
        )?;
        self.run_command(formatting)?;
        let lock_sha256 = Self::sha256(&self.root.join("Cargo.lock"))?;
        self.run_command(packaging)?;
        self.check_surfaces(selection, surfaces, None, false)?;
        require(
            self.git(&["rev-parse", "HEAD"])? == selection.source,
            "source changed during preparation",
        )?;
        self.verify_inputs(&validation, true)?;
        require(
            Self::sha256(&self.root.join("Cargo.lock"))? == lock_sha256,
            "packaging changed selected dependencies",
        )?;
        let head = selection.source.clone();
        let evidence =
            PackageEvidence::record(self, validation, head, lock_sha256, commands.to_vec())?;
        self.write_evidence(&evidence, Prepared)?;
        Ok(evidence)
    }
}
