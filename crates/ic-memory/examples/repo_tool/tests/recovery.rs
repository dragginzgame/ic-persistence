//! Recovery release contracts.

use super::*;

#[test]
fn version_preparation_failures_restore_files_and_preserve_evidence_and_artifacts() {
    for failure in [
        BuildFault::Validate,
        BuildFault::Formatting,
        BuildFault::Package,
        BuildFault::DependencyUpdate,
    ] {
        let fixture = Fixture::new();
        let receipt = fixture.repo.receipt_path("0.13.0", Final).unwrap();
        fs::create_dir_all(receipt.parent().unwrap()).unwrap();
        fs::write(&receipt, "previous evidence").unwrap();
        let sentinel = fixture.repo.target().unwrap().join("consumer-evidence");
        fs::write(&sentinel, "keep").unwrap();
        fixture.fail_build(failure.clone());
        assert!(fixture.prepare().is_err());
        assert_eq!(fixture.repo.version(None).unwrap(), "0.12.3");
        if failure == BuildFault::DependencyUpdate {
            assert!(
                fs::read_to_string(fixture.repo.root.join("Cargo.lock"))
                    .unwrap()
                    .contains("name = \"other\"\nversion = \"0.12.4\"")
            );
        } else {
            fixture.assert_original();
        }
        for (path, text) in &fixture.repo.exec.state.borrow().git.source {
            if failure == BuildFault::DependencyUpdate && path == "Cargo.lock" {
                continue;
            }
            assert_eq!(
                fs::read_to_string(fixture.repo.root.join(path)).unwrap(),
                *text
            );
        }
        assert_eq!(fs::read_to_string(receipt).unwrap(), "previous evidence");
        assert_eq!(fs::read_to_string(sentinel).unwrap(), "keep");
        if failure == BuildFault::Formatting {
            assert!(
                !fixture
                    .repo
                    .exec
                    .state
                    .borrow()
                    .calls
                    .iter()
                    .any(|call| { is_package_call(call) })
            );
        }
    }
}

#[test]
fn preparation_retains_primary_and_rollback_errors_and_restores_other_owned_files() {
    struct RollbackFailures<'a>(&'a Substitute);

    impl Execute for RollbackFailures<'_> {
        fn cargo_target(&self, root: &Path) -> Result<PathBuf> {
            self.0.cargo_target(root)
        }

        fn capture(&self, root: &Path, program: &str, args: &[&str]) -> Result<String> {
            self.0.capture(root, program, args)
        }

        fn execute(&self, root: &Path, program: &str, args: &[&str]) -> Result<()> {
            self.0.execute(root, program, args)?;
            if program == "bash" && args.first() == Some(&"scripts/ci/package-workspace.sh") {
                // A missing surface models failed inspection during restoration.
                fs::rename(root.join("CHANGELOG.md"), root.join("retained-changelog"))?;
                // A foreign replacement must never be overwritten by rollback.
                fs::write(root.join(PACKAGE_README), b"concurrent edit")?;
                return Err(std::io::Error::from(std::io::ErrorKind::UnexpectedEof).into());
            }
            Ok(())
        }

        fn write_bytes(&self, path: &Path, bytes: &[u8]) -> Result<()> {
            if path.ends_with("Cargo.lock") && bytes == LOCK.as_bytes() {
                return Err(std::io::Error::from(std::io::ErrorKind::PermissionDenied).into());
            }
            self.0.write_bytes(path, bytes)
        }
    }

    let fixture = Fixture::new();
    fixture.repo.verify(&fixture.selection).unwrap();
    let validation = fixture.repo.validation_path("0.13.0").unwrap();
    let saved = fs::read(&validation).unwrap();
    let repo = Repository {
        root: fixture.repo.root.clone(),
        exec: RollbackFailures(&fixture.repo.exec),
    };
    let error = repo.prepare_selected(&fixture.selection).unwrap_err();
    let failure = error.downcast_ref::<PreparationError>().unwrap();
    assert_eq!(
        failure
            .source
            .downcast_ref::<std::io::Error>()
            .unwrap()
            .kind(),
        std::io::ErrorKind::UnexpectedEof
    );
    let failures: BTreeMap<_, _> = failure
        .rollback_errors
        .iter()
        .map(|(path, error)| {
            (
                path.as_str(),
                error.downcast_ref::<std::io::Error>().unwrap().kind(),
            )
        })
        .collect();
    assert_eq!(
        failures,
        BTreeMap::from([
            ("CHANGELOG.md", std::io::ErrorKind::NotFound),
            ("Cargo.lock", std::io::ErrorKind::PermissionDenied),
        ])
    );
    assert_eq!(repo.version(None).unwrap(), "0.12.3");
    assert_eq!(
        fs::read(repo.root.join(PACKAGE_README)).unwrap(),
        b"concurrent edit"
    );
    assert_eq!(
        fs::read_to_string(repo.root.join("docs/changelog/0.13.md")).unwrap(),
        fixture.repo.exec.state.borrow().git.source["docs/changelog/0.13.md"]
    );
    assert_eq!(fs::read(validation).unwrap(), saved);
    assert!(repo.package_path("ic-memory", "0.13.0").unwrap().exists());
    assert!(!repo.receipt_path("0.13.0", Prepared).unwrap().exists());
    assert!(failure.source().unwrap().is::<std::io::Error>());
    let diagnostic = failure.to_string();
    for path in failures.keys() {
        assert!(diagnostic.contains(path));
    }
}

#[test]
fn failed_first_publication_preserves_untouched_metadata_identity_and_permissions() {
    use std::os::unix::fs::{MetadataExt as _, PermissionsExt as _};

    let fixture = Fixture::new();
    fixture.repo.verify(&fixture.selection).unwrap();
    let validation = fixture.repo.validation_path("0.13.0").unwrap();
    let saved = fs::read(&validation).unwrap();
    let metadata: BTreeMap<_, _> = fixture
        .repo
        .surfaces(&fixture.selection)
        .unwrap()
        .into_keys()
        .map(|path| {
            let file = fixture.repo.root.join(&path);
            fs::set_permissions(&file, fs::Permissions::from_mode(0o600)).unwrap();
            let metadata = fs::metadata(file).unwrap();
            (path, (metadata.ino(), metadata.mode()))
        })
        .collect();
    fixture.fail_files(FileFault::WriteBefore("CHANGELOG.md".into()));
    let error = fixture
        .repo
        .prepare_selected(&fixture.selection)
        .unwrap_err();
    assert!(error.is::<std::io::Error>());
    fixture.assert_original();
    for (path, identity) in metadata {
        let actual = fs::metadata(fixture.repo.root.join(path)).unwrap();
        assert_eq!((actual.ino(), actual.mode()), identity);
    }
    assert_eq!(fs::read(validation).unwrap(), saved);
    assert!(
        !fixture
            .repo
            .receipt_path("0.13.0", Prepared)
            .unwrap()
            .exists()
    );
}

#[test]
fn metadata_publication_failures_reconcile_visible_bytes_and_retry_saved_intent() {
    for after_publication in [false, true] {
        for path in [
            "Cargo.lock",
            "CHANGELOG.md",
            "docs/changelog/0.13.md",
            "Cargo.toml",
        ] {
            let fixture = Fixture::new();
            fixture.repo.verify(&fixture.selection).unwrap();
            let receipt = fixture.repo.validation_path("0.13.0").unwrap();
            let saved = fs::read(&receipt).unwrap();
            let sentinel = fixture.repo.target().unwrap().join("consumer-evidence");
            fs::write(&sentinel, "keep").unwrap();
            fixture.fail_files(if after_publication {
                FileFault::WriteAfter(path.into())
            } else {
                FileFault::WriteBefore(path.into())
            });
            assert!(fixture.repo.prepare_selected(&fixture.selection).is_err());
            for (path, text) in &fixture.repo.exec.state.borrow().git.source {
                assert_eq!(
                    fs::read(fixture.repo.root.join(path)).unwrap(),
                    text.as_bytes()
                );
            }
            assert_eq!(fs::read(&receipt).unwrap(), saved);
            assert_eq!(fs::read_to_string(&sentinel).unwrap(), "keep");
            let published_errors = fixture
                .repo
                .exec
                .state
                .borrow()
                .calls
                .iter()
                .filter(|call| call[0] == "published-write-error")
                .count();
            assert_eq!(published_errors, usize::from(after_publication));
            fixture.repo.prepare_selected(&fixture.selection).unwrap();
            fixture.repo.prepared(&fixture.selection).unwrap();
            assert_eq!(fixture.repo.version(None).unwrap(), "0.13.0");
            assert_eq!(fs::read(receipt).unwrap(), saved);
            assert_eq!(fs::read_to_string(sentinel).unwrap(), "keep");
            assert_eq!(
                fixture
                    .repo
                    .exec
                    .state
                    .borrow()
                    .calls
                    .iter()
                    .filter(|call| { is_make_call(call, "validate") })
                    .count(),
                1
            );
        }
    }
}

#[test]
fn committed_preparation_restart_retains_receipts_before_fresh_preparation() {
    let mut fixture = Fixture::new();
    fixture.prepare().unwrap();
    let validation = fixture.repo.validation_path("0.13.0").unwrap();
    let prepared = fixture.repo.receipt_path("0.13.0", Prepared).unwrap();
    let old_validation = fs::read(&validation).unwrap();
    let old_prepared = fs::read(&prepared).unwrap();
    let old_evidence: PackageEvidence = serde_json::from_slice(&old_prepared).unwrap();
    let archive = fixture
        .repo
        .retained_package(&old_evidence.package_sha256)
        .unwrap();
    let old_archive = fs::read(&archive).unwrap();
    // Substitute a clean descendant committing rollback to the original metadata.
    // No real Git commit, tag or release effect is created.
    let mut descendant = fixture.repo.exec.state.borrow().git.source.clone();
    descendant.insert("source.rs".to_owned(), "// committed fix\n".to_owned());
    for (path, text) in &descendant {
        fs::write(fixture.repo.root.join(path), text).unwrap();
    }
    fixture.repo.exec.state.borrow_mut().git.descendant = Some(descendant);
    fixture.selection.source = "fix".to_owned();
    fixture.repo.preflight(&fixture.selection).unwrap();
    fixture.fail_build(BuildFault::Validate);
    assert!(fixture.repo.verify(&fixture.selection).is_err());
    assert_eq!(fs::read(&validation).unwrap(), old_validation);
    assert_eq!(fs::read(&prepared).unwrap(), old_prepared);
    fixture.repo.exec.state.borrow_mut().clear_faults();
    for conflict in ["destination", "receipt"] {
        let bytes = if conflict == "destination" {
            let mut foreign: serde_json::Value = serde_json::from_slice(&old_prepared).unwrap();
            foreign["validation"]["selection"]["remote"] = serde_json::json!("other");
            serde_json::to_vec(&foreign).unwrap()
        } else {
            b"corrupted prepared receipt".to_vec()
        };
        fs::write(&prepared, &bytes).unwrap();
        assert!(
            fixture.repo.verify(&fixture.selection).is_err(),
            "{conflict}"
        );
        assert_eq!(fs::read(&prepared).unwrap(), bytes);
        assert_eq!(fs::read(&validation).unwrap(), old_validation);
        fs::write(&prepared, &old_prepared).unwrap();
    }
    fixture.fail_git(GitFault::History);
    assert!(fixture.repo.verify(&fixture.selection).is_err());
    assert_eq!(fs::read(&prepared).unwrap(), old_prepared);
    assert_eq!(fs::read(&validation).unwrap(), old_validation);
    fixture.repo.exec.state.borrow_mut().clear_faults();
    let attempts = validation.parent().unwrap().join("attempts");
    fs::write(&attempts, b"blocked retention directory").unwrap();
    assert!(fixture.repo.verify(&fixture.selection).is_err());
    assert_eq!(fs::read(&prepared).unwrap(), old_prepared);
    assert_eq!(fs::read(&validation).unwrap(), old_validation);
    fs::remove_file(&attempts).unwrap();
    fixture.repo.verify(&fixture.selection).unwrap();
    fixture.repo.prepare_selected(&fixture.selection).unwrap();
    let current: PackageEvidence = serde_json::from_slice(&fs::read(&prepared).unwrap()).unwrap();
    assert_eq!(current.validation.selection.source, "fix");
    let retained: Vec<_> = fs::read_dir(attempts)
        .unwrap()
        .map(|entry| fs::read(entry.unwrap().path()).unwrap())
        .collect();
    assert!(retained.contains(&old_validation));
    assert!(retained.contains(&old_prepared));
    assert_eq!(fs::read(&archive).unwrap(), old_archive);
}

#[test]
fn preparation_restart_recovers_from_validation_receipt_publication_errors() {
    for after_publication in [false, true] {
        let mut fixture = Fixture::new();
        fixture.prepare().unwrap();
        let validation = fixture.repo.validation_path("0.13.0").unwrap();
        let prepared = fixture.repo.receipt_path("0.13.0", Prepared).unwrap();
        let old_validation = fs::read(&validation).unwrap();
        let old_prepared = fs::read(&prepared).unwrap();
        let evidence: PackageEvidence = serde_json::from_slice(&old_prepared).unwrap();
        let archive = fixture
            .repo
            .retained_package(&evidence.package_sha256)
            .unwrap();
        let old_archive = fs::read(&archive).unwrap();
        let mut descendant = fixture.repo.exec.state.borrow().git.source.clone();
        descendant.insert("source.rs".to_owned(), "// committed fix\n".to_owned());
        for (path, text) in &descendant {
            fs::write(fixture.repo.root.join(path), text).unwrap();
        }
        fixture.repo.exec.state.borrow_mut().git.descendant = Some(descendant);
        fixture.selection.source = "fix".to_owned();
        fixture.repo.exec.state.borrow_mut().calls.clear();
        fixture.fail_files(if after_publication {
            FileFault::WriteAfter("0.13.0-validated.json".into())
        } else {
            FileFault::WriteBefore("0.13.0-validated.json".into())
        });
        assert!(
            fixture.repo.verify(&fixture.selection).is_err(),
            "after_publication={after_publication}"
        );
        let visible: ValidationEvidence =
            serde_json::from_slice(&fs::read(&validation).unwrap()).unwrap();
        assert_eq!(
            visible.selection.source,
            if after_publication { "fix" } else { "source" }
        );
        assert!(
            !fixture
                .repo
                .exec
                .state
                .borrow()
                .calls
                .iter()
                .any(|call| { is_cargo_call(call, "update") || is_package_call(call) })
        );

        // Retry through fresh validation, regardless of whether publication
        // left the previous or replacement receipt visible.
        fixture.repo.verify(&fixture.selection).unwrap();
        fixture.repo.prepare_selected(&fixture.selection).unwrap();
        let current: PackageEvidence =
            serde_json::from_slice(&fs::read(&prepared).unwrap()).unwrap();
        assert_eq!(current.validation.selection, fixture.selection);
        let retained: Vec<_> = fs::read_dir(validation.parent().unwrap().join("attempts"))
            .unwrap()
            .map(|entry| fs::read(entry.unwrap().path()).unwrap())
            .collect();
        assert!(retained.contains(&old_validation));
        assert!(retained.contains(&old_prepared));
        assert_eq!(fs::read(&archive).unwrap(), old_archive);
    }
}

#[test]
fn interrupted_preparation_finishes_saved_candidate_without_revalidating() {
    for interruption in ["partial", "partial-lock", "manifest", "lock", "package"] {
        let fixture = Fixture::new();
        fixture.repo.verify(&fixture.selection).unwrap();
        let evidence = fs::read(fixture.repo.validation_path("0.13.0").unwrap()).unwrap();
        let expected = fixture.repo.surfaces(&fixture.selection).unwrap();
        for (path, text) in &expected {
            if match interruption {
                "partial" => path == "CHANGELOG.md",
                "partial-lock" => path.as_str() <= "Cargo.lock",
                _ => true,
            } {
                fs::write(fixture.repo.root.join(path), &text.expected).unwrap();
            }
        }
        if matches!(interruption, "lock" | "package") {
            fixture
                .repo
                .run_command(&fixture.repo.preparation_commands().unwrap()[0])
                .unwrap();
        }
        if interruption == "package" {
            fixture.fail_build(BuildFault::Package);
            assert!(fixture.repo.prepared(&fixture.selection).is_err());
            assert!(
                fixture
                    .repo
                    .package_path("ic-memory", "0.13.0")
                    .unwrap()
                    .exists()
            );
            fixture.repo.exec.state.borrow_mut().clear_faults();
        }
        if matches!(interruption, "partial" | "partial-lock") {
            fixture.repo.preflight(&fixture.selection).unwrap();
            if interruption == "partial-lock" {
                assert!(
                    !fixture
                        .repo
                        .exec
                        .state
                        .borrow()
                        .calls
                        .iter()
                        .any(|call| { is_cargo_call(call, "fetch") })
                );
            }
            // Already installed candidate bytes must survive a write refusal.
            let unchanged = if interruption == "partial" {
                "CHANGELOG.md"
            } else {
                "Cargo.lock"
            };
            fixture.fail_files(FileFault::WriteBefore(unchanged.into()));
            fixture.repo.prepare_selected(&fixture.selection).unwrap();
        } else {
            fixture.repo.prepared(&fixture.selection).unwrap();
        }
        for (path, text) in expected {
            assert_eq!(
                fs::read_to_string(fixture.repo.root.join(path)).unwrap(),
                text.expected
            );
        }
        assert_eq!(
            fs::read(fixture.repo.validation_path("0.13.0").unwrap()).unwrap(),
            evidence
        );
        let calls = fixture.repo.exec.state.borrow().calls.clone();
        assert_eq!(
            calls
                .iter()
                .filter(|call| is_make_call(call, "validate"))
                .count(),
            1
        );
        let start = calls.len();
        fixture.repo.prepared(&fixture.selection).unwrap();
        assert!(
            !fixture.repo.exec.state.borrow().calls[start..]
                .iter()
                .any(|call| is_cargo_call(call, "update")
                    || is_package_call(call)
                    || is_make_call(call, "fmt-check")
                    || is_make_call(call, "validate"))
        );
        assert!(fixture.repo.exec.state.borrow().git.release.is_none());
    }
}

#[test]
fn partial_lock_preflight_requires_saved_validation() {
    for invalid in ["missing", "selection", "configuration"] {
        let fixture = Fixture::new();
        fixture.repo.verify(&fixture.selection).unwrap();
        let path = fixture.repo.validation_path("0.13.0").unwrap();
        if invalid == "missing" {
            fs::remove_file(&path).unwrap();
        } else {
            let mut evidence: ValidationEvidence =
                serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
            if invalid == "selection" {
                evidence.selection.date = "2026-10-04".to_owned();
            } else {
                evidence
                    .configuration
                    .insert("RUSTFLAGS".to_owned(), "--cfg foreign".to_owned());
            }
            fs::write(&path, serde_json::to_vec(&evidence).unwrap()).unwrap();
        }
        let expected = fixture.repo.surfaces(&fixture.selection).unwrap();
        fs::write(
            fixture.repo.root.join("Cargo.lock"),
            &expected["Cargo.lock"].expected,
        )
        .unwrap();
        assert!(fixture.repo.preflight(&fixture.selection).is_err());
        assert_eq!(fixture.repo.version(None).unwrap(), "0.12.3");
        assert_eq!(
            fs::read_to_string(fixture.repo.root.join("Cargo.lock")).unwrap(),
            expected["Cargo.lock"].expected
        );
        assert!(
            !fixture
                .repo
                .exec
                .state
                .borrow()
                .calls
                .iter()
                .any(|call| { is_cargo_call(call, "fetch") })
        );
    }
}

#[test]
fn preparation_recovery_preserves_conflicting_inputs_and_corrupted_evidence() {
    for conflict in ["metadata", "lock", "receipt", "archive"] {
        let fixture = Fixture::new();
        fixture.prepare().unwrap();
        let receipt = fixture.repo.receipt_path("0.13.0", Prepared).unwrap();
        let evidence: PackageEvidence =
            serde_json::from_slice(&fs::read(&receipt).unwrap()).unwrap();
        let path = match conflict {
            "metadata" => fixture.repo.root.join(PACKAGE_README),
            "lock" => fixture.repo.root.join("Cargo.lock"),
            "receipt" => receipt,
            "archive" => fixture
                .repo
                .retained_package(&evidence.package_sha256)
                .unwrap(),
            _ => unreachable!(),
        };
        fs::write(&path, b"conflicting retained input").unwrap();
        let start = fixture.repo.exec.state.borrow().calls.len();
        assert!(
            fixture.repo.prepared(&fixture.selection).is_err(),
            "{conflict}"
        );
        assert_eq!(fs::read(&path).unwrap(), b"conflicting retained input");
        assert!(
            !fixture.repo.exec.state.borrow().calls[start..]
                .iter()
                .any(|call| is_cargo_call(call, "update") || is_package_call(call))
        );
    }
}

#[test]
fn archive_publication_errors_preserve_validation_and_reconcile_visible_bytes() {
    for phase in [FileFault::ArchiveBefore, FileFault::ArchiveAfter] {
        let fixture = Fixture::new();
        fixture.repo.verify(&fixture.selection).unwrap();
        let validation = fixture.repo.validation_path("0.13.0").unwrap();
        let validation_bytes = fs::read(&validation).unwrap();
        let digest = Sha256Digest::compute(b"prepared-archive").to_string();
        let retained = fixture.repo.retained_package(&digest).unwrap();
        fs::create_dir_all(retained.parent().unwrap()).unwrap();
        let foreign = retained.with_extension("crate.tmp");
        fs::write(&foreign, b"unrelated evidence").unwrap();
        fixture.fail_files(phase.clone());
        assert!(fixture.repo.prepare_selected(&fixture.selection).is_err());
        fixture.assert_original();
        assert_eq!(fs::read(&validation).unwrap(), validation_bytes);
        assert!(
            !fixture
                .repo
                .receipt_path("0.13.0", Prepared)
                .unwrap()
                .exists()
        );
        assert_eq!(fs::read(&foreign).unwrap(), b"unrelated evidence");
        if phase == FileFault::ArchiveAfter {
            assert_eq!(fs::read(&retained).unwrap(), b"prepared-archive");
        } else {
            assert!(!retained.exists());
        }
        let transfers = || {
            fixture
                .repo
                .exec
                .state
                .borrow()
                .calls
                .iter()
                .filter(|call| {
                    call.first()
                        .is_some_and(|command| command == "stream-write")
                })
                .count()
        };
        let before_retry = transfers();
        fixture.repo.exec.files.borrow_mut().fault = None;
        fixture.repo.prepare_selected(&fixture.selection).unwrap();
        let evidence = fixture.repo.prepared(&fixture.selection).unwrap();
        assert_eq!(evidence.package_sha256, digest);
        assert_eq!(fs::read(&retained).unwrap(), b"prepared-archive");
        assert_eq!(fs::read(&validation).unwrap(), validation_bytes);
        assert_eq!(fs::read(&foreign).unwrap(), b"unrelated evidence");
        assert_eq!(
            transfers() - before_retry,
            usize::from(phase == FileFault::ArchiveBefore) + packages().len() - 1
        );
        fs::write(&retained, b"corrupted retained archive").unwrap();
        assert!(fixture.repo.record_package("ic-memory", "0.13.0").is_err());
        assert_eq!(fs::read(&retained).unwrap(), b"corrupted retained archive");
    }
}
