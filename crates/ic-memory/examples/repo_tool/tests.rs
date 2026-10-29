//! Release effects are substituted; native Git checks use disposable indexes.
//! No fixture commits, tags, pushes or live publication occur.

use super::*;
use std::cell::RefCell;

mod build;
mod files;
mod git;
mod registry;
use build::{BuildEffects, BuildFault};
use files::{FileEffects, FileFault};
use git::{GitFault, GitState};
use registry::{RegistryEffects, RegistryFault};

#[test]
fn command_arguments_scope_dry_run_to_publication() {
    for args in [
        vec!["repo-tool"],
        vec!["repo-tool", "version", "--dry-run"],
        vec!["repo-tool", "publish", "unexpected"],
        vec!["repo-tool", "unknown"],
    ] {
        assert!(Arguments::try_parse_from(args).is_err());
    }
    for (args, expected) in [
        (vec!["repo-tool", "publish"], false),
        (vec!["repo-tool", "publish", "--dry-run"], true),
    ] {
        assert!(matches!(
            Arguments::try_parse_from(args).unwrap().command,
            ToolCommand::Publish { dry_run } if dry_run == expected
        ));
    }
}

#[cfg(unix)]
#[test]
fn package_retention_refuses_a_replaced_fifo_without_a_writer() {
    if env::var_os("IC_MEMORY_PACKAGE_FIFO_CHILD").is_some() {
        let fixture = Fixture::new();
        let package = fixture.repo.package_path("ic-memory", "0.13.0").unwrap();
        fs::create_dir_all(package.parent().unwrap()).unwrap();
        fs::write(&package, b"package bytes").unwrap();
        fixture.repo.exec.files.borrow_mut().archive_change = Some(package);
        fixture.fail_files(FileFault::ArchiveFifo);
        let error = fixture
            .repo
            .record_package("ic-memory", "0.13.0")
            .unwrap_err();
        let ic_host_fs::durable::NamedWriteError::Producer {
            source,
            cleanup_error: None,
        } = error
            .downcast_ref::<ic_host_fs::durable::NamedWriteError<ic_host_artifacts::artifact::CopyError>>()
            .unwrap()
        else {
            panic!("expected clean producer refusal");
        };
        assert!(matches!(
            source,
            ic_host_artifacts::artifact::CopyError::Input(
                ic_host_artifacts::artifact::ArtifactError::NotRegularFile
            )
        ));
        let digest = Sha256Digest::compute(b"package bytes").to_string();
        assert!(!fixture.repo.retained_package(&digest).unwrap().exists());
        fs::write(
            env::var_os("IC_MEMORY_PACKAGE_FIFO_RESULT").unwrap(),
            b"refused",
        )
        .unwrap();
        return;
    }

    // A regression to blocking File::open must fail without hanging the suite.
    let parent = Fixture::new();
    let result = parent.repo.root.join("fifo-result");
    let mut command = Command::new(env::current_exe().unwrap());
    command
        .args([
            "--exact",
            "repo_tool::tests::package_retention_refuses_a_replaced_fifo_without_a_writer",
            "--test-threads=1",
        ])
        .env("IC_MEMORY_PACKAGE_FIFO_CHILD", "1")
        .env("IC_MEMORY_PACKAGE_FIFO_RESULT", &result);
    capture_command(
        &mut command,
        OutputLimits {
            stdout: OutputLimit::Terminate(16 * 1024),
            stderr: OutputLimit::Terminate(16 * 1024),
            timeout: Some(std::time::Duration::from_secs(15)),
        },
    )
    .and_then(CompletedExecution::require_success)
    .and_then(CompletedExecution::require_complete)
    .unwrap();
    assert_eq!(fs::read(result).unwrap(), b"refused");
}

const PACKAGE_MANIFEST: &str = "crates/ic-memory/Cargo.toml";

// Only these narrow command shapes are semantic effects in this fixture.
// Keep raw argv logs and exact-dispatch assertions independent of these predicates.
fn is_cargo_call(call: &[String], command: &str) -> bool {
    call.first().is_some_and(|program| program == "cargo")
        && call.get(2).is_some_and(|argument| argument == command)
}

fn is_make_call(call: &[String], target: &str) -> bool {
    call.first().is_some_and(|program| program == "make")
        && call
            .get(1)
            .is_some_and(|argument| argument == "--no-print-directory")
        && call.get(2).is_some_and(|argument| argument == target)
}

fn is_package_call(call: &[String]) -> bool {
    is_cargo_call(call, "package")
        || call.first().is_some_and(|program| program == "bash")
            && call
                .get(1)
                .is_some_and(|script| script == "scripts/ci/package-workspace.sh")
}

const MANIFEST: &str = "[workspace]\nmembers = [\"crates/ic-memory\", \"crates/ic-storage\", \"crates/ic-metrics-stable\", \"crates/ic-logging-stable\"]\n[workspace.package]\nversion = \"0.12.3\"\nrust-version = \"1.88.0\"\n[workspace.dependencies]\nic-memory = { path = \"crates/ic-memory\", version = \"0.12.3\" }\nic-storage = { path = \"crates/ic-storage\", version = \"0.12.3\" }\nother = \"0.12.3\"\n";
const PACKAGE: &str = "[package]\nname = \"ic-memory\"\nversion.workspace = true\n";
const LOCK: &str = "version = 4\n[[package]]\nname = \"ic-logging-stable\"\nversion = \"0.12.3\"\n[[package]]\nname = \"ic-metrics-stable\"\nversion = \"0.12.3\"\ndependencies = [\"ic-memory\", \"ic-storage\"]\n[[package]]\nname = \"ic-storage\"\nversion = \"0.12.3\"\ndependencies = [\"ic-memory\"]\n[[package]]\nname = \"ic-memory\"\nversion = \"0.12.3\"\n[[package]]\nname = \"other\"\nversion = \"0.12.3\"\n";

#[test]
fn native_metadata_refuses_invalid_selected_manifest_without_touching_lock() {
    let fixture = Fixture::new();
    let lock = fs::read(fixture.repo.root.join("Cargo.lock")).unwrap();
    fs::write(fixture.repo.root.join("Cargo.toml"), "[invalid manifest").unwrap();
    let error = Processes.cargo_target(&fixture.repo.root).unwrap_err();
    let ToolError::Execution(failure) = error.downcast_ref::<ToolError>().unwrap() else {
        panic!("expected the selected Host's execution failure");
    };
    assert!(!failure.evidence.status.unwrap().success());
    assert_ne!(failure.evidence.stderr.as_slice(), b"");
    assert_eq!(
        fs::read(fixture.repo.root.join("Cargo.lock")).unwrap(),
        lock
    );
}

#[test]
fn captured_failure_preserves_status_and_both_output_streams() {
    let fixture = Fixture::new();
    let error = Processes
        .capture(
            &fixture.repo.root,
            "sh",
            &["-c", "printf stdout; printf stderr >&2; exit 37"],
        )
        .unwrap_err();
    let ToolError::Execution(failure) = error.downcast_ref::<ToolError>().unwrap() else {
        panic!("expected an execution failure");
    };
    assert_eq!(failure.evidence.status.unwrap().code(), Some(37));
    assert_eq!(failure.evidence.stdout, b"stdout");
    assert_eq!(failure.evidence.stderr, b"stderr");
}

#[test]
fn tag_changes_during_shared_admission_stop_publication() {
    for failure in [
        GitFault::TagAdmissionRefused,
        GitFault::TagAdmissionObject,
        GitFault::TagAdmissionMissing,
    ] {
        let fixture = Fixture::new();
        fixture.release();
        fixture.fail_git(failure.clone());
        assert!(fixture.repo.publish(true).is_err(), "{failure:?}");
        assert!(!fixture.repo.exec.state.borrow().calls.iter().any(|call| {
            call.first().is_some_and(|arg| arg == "cargo") && is_cargo_call(call, "publish")
        }));
    }
}

#[test]
fn helper_output_cannot_change_locked_dependency_selection() {
    let fixture = Fixture::new();
    let same_name_registry = format!(
        "{LOCK}[[package]]\nname = \"ic-memory\"\nversion = \"0.12.3\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\nchecksum = \"unchanged\"\n"
    );
    let rendered =
        Repository::<Processes>::replace_lock_version(&same_name_registry, "0.12.3", "0.13.0")
            .unwrap();
    assert_eq!(
        rendered,
        same_name_registry.replacen("version = \"0.12.3\"", "version = \"0.13.0\"", 4)
    );
    let changed = LOCK.replace("0.12.3", "0.13.0");
    let error = check_lock_update(LOCK.as_bytes(), &changed, "0.12.3", "0.13.0").unwrap_err();
    assert!(matches!(
        error.downcast_ref::<ic_build::cargo::lock_projection::LockProjectionError>(),
        Some(ic_build::cargo::lock_projection::LockProjectionError::ChangedInputs)
    ));
    fixture.assert_original();
}

#[test]
fn coordinated_version_projection_and_receipts_include_all_four_packages() {
    let fixture = Fixture::new();
    fixture.prepare().unwrap();
    let lock: toml::Value =
        toml::from_str(&fixture.repo.text("Cargo.lock", None).unwrap()).unwrap();
    for name in packages() {
        let record = lock["package"]
            .as_array()
            .unwrap()
            .iter()
            .find(|record| record["name"].as_str() == Some(name))
            .unwrap();
        assert_eq!(record["version"].as_str(), Some("0.13.0"));
    }
    let prepared: PackageEvidence = serde_json::from_slice(
        &fs::read(fixture.repo.receipt_path("0.13.0", Prepared).unwrap()).unwrap(),
    )
    .unwrap();
    assert_eq!(
        prepared
            .archives()
            .map(|(name, _)| name)
            .collect::<Vec<_>>(),
        packages()
    );
    assert!(packages().contains(&"ic-logging-stable"));
}

#[test]
fn changed_transform_input_is_retained_and_refused_on_retry() {
    let fixture = Fixture::new();
    let repo = Repository {
        root: fixture.repo.root.clone(),
        exec: Processes,
    };
    assert!(
        repo.transform_text(
            "sh",
            &["-c", "printf other > \"$1\"; printf ready", "fixture"],
            "input",
            &[],
        )
        .is_err()
    );
    let input = repo
        .target()
        .unwrap()
        .join("release-validation/inputs")
        .join(format!("{}.input", Sha256Digest::compute(b"input")));
    assert_eq!(fs::read(&input).unwrap(), b"other");
    assert!(repo.transform_text("cat", &[], "input", &[]).is_err());
    assert_eq!(fs::read(&input).unwrap(), b"other");
    fixture.assert_original();
}

#[test]
fn large_changelog_finalization_preserves_complete_history() {
    let fixture = Fixture::new();
    let text = format!(
        "## [0.13.0]\n\n{}\n## [0.12.3] - 2026-10-04\n\n## [Draft]\n\nArchived wording.\n",
        "A detailed note.\n".repeat(8192)
    );
    let updated = fixture
        .repo
        .release_changelog(
            &fixture.repo.root.join(CHANGELOG_FINALIZER),
            &text,
            "0.12.3",
            "0.13.0",
            "2026-10-05",
        )
        .unwrap();
    assert_eq!(
        updated,
        text.replacen("## [0.13.0]", "## [0.13.0] - 2026-10-05", 1)
    );
    fixture.assert_original();
}

#[test]
fn release_version_requires_the_named_package_to_inherit_the_workspace_identity() {
    let fixture = Fixture::new();
    assert_eq!(fixture.repo.version(None).unwrap(), "0.12.3");
    for name in packages() {
        for manifest in [
            String::new(),
            "[workspace]\n".to_owned(),
            format!("[package]\nname = {name:?}\n"),
            "[package]\nname = \"another-package\"\nversion.workspace = true\n".to_owned(),
            format!("[package]\nname = {name:?}\nversion = \"0.12.3\"\n"),
        ] {
            fs::write(fixture.repo.root.join(package_manifest(name)), manifest).unwrap();
            assert!(fixture.repo.member_release_manifest(None, name).is_err());
            assert!(
                fixture
                    .repo
                    .member_release_manifest(Some("source"), name)
                    .is_ok()
            );
            if name == "ic-memory" {
                assert!(fixture.repo.version(None).is_err());
                assert_eq!(fixture.repo.version(Some("source")).unwrap(), "0.12.3");
            }
        }
    }
}

struct Fixture {
    repo: Repository<Substitute>,
    selection: ReleaseSelection,
}

impl Fixture {
    fn new() -> Self {
        Self::new_in(&env::temp_dir())
    }

    fn new_in(parent: &Path) -> Self {
        // Exclusive creation owns uniqueness, including across PID reuse.
        // Never reuse or remove an occupied path: it may hold failed evidence.
        let mut sequence = 0_u64;
        let root = loop {
            let candidate = parent.join(format!(
                "ic-memory-tooling-{}-{sequence}",
                std::process::id()
            ));
            match fs::create_dir(&candidate) {
                Ok(()) => break candidate,
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => sequence += 1,
                Err(error) => panic!("could not create fixture {}: {error}", candidate.display()),
            }
        };
        let mut source = BTreeMap::from([
            (".gitignore".to_owned(), "target/\n".to_owned()),
            (
                "crates/ic-memory/src/lib.rs".to_owned(),
                "// fixture library\n".to_owned(),
            ),
            ("Cargo.lock".to_owned(), LOCK.to_owned()),
            ("Cargo.toml".to_owned(), MANIFEST.to_owned()),
            (PACKAGE_MANIFEST.to_owned(), PACKAGE.to_owned()),
            (
                PACKAGE_README.to_owned(),
                "# Fixture\n\nic-memory = \"0.12.3\"\n".to_owned(),
            ),
            (
                "CHANGELOG.md".to_owned(),
                "# Changelog\n\n## [0.13.0]\n\nFixture changes.\n\n## 0.12.3\n\nHistory.\n"
                    .to_owned(),
            ),
            ("source.rs".to_owned(), "// committed source\n".to_owned()),
            (
                "docs/changelog/0.13.md".to_owned(),
                "# Detail\n\n## [0.13.0]\n\nFixture changes.\n".to_owned(),
            ),
        ]);
        for (name, library) in packages().into_iter().skip(1).zip([
            "// storage fixture\n",
            "// metrics fixture\n",
            "// logging fixture\n",
        ]) {
            source.insert(
                package_manifest(name),
                format!("[package]\nname = {name:?}\nversion.workspace = true\n"),
            );
            source.insert(format!("crates/{name}/src/lib.rs"), library.to_owned());
        }
        for helper in [
            CHANGELOG_FINALIZER,
            RELEASE_INPUT_ADMISSION,
            "scripts/ci/StableVersion.pm",
        ] {
            source.insert(
                helper.to_owned(),
                fs::read_to_string(
                    Path::new(env!("CARGO_MANIFEST_DIR"))
                        .join("../..")
                        .join(helper),
                )
                .unwrap(),
            );
        }
        for (path, text) in &source {
            fs::create_dir_all(root.join(path).parent().unwrap()).unwrap();
            fs::write(root.join(path), text).unwrap();
        }
        fs::write(
            root.join("rust-toolchain.toml"),
            "[toolchain]\nchannel = \"1.99.0\"\n",
        )
        .unwrap();
        let exec = Substitute {
            files: RefCell::new(FileEffects::default()),
            state: RefCell::new(State {
                git: GitState {
                    source,
                    ..GitState::default()
                },
                ..State::default()
            }),
        };
        Self {
            repo: Repository { root, exec },
            selection: ReleaseSelection {
                kind: "minor".to_owned(),
                previous: "0.12.3".to_owned(),
                version: "0.13.0".to_owned(),
                date: "2026-10-05".to_owned(),
                source: "source".to_owned(),
                remote: "origin".to_owned(),
                branch: "main".to_owned(),
            },
        }
    }

    fn prepare(&self) -> Result<()> {
        self.repo.preflight(&self.selection)?;
        self.repo.verify(&self.selection)?;
        self.repo.preflight(&self.selection)?;
        self.repo.prepare_selected(&self.selection)
    }

    fn stage(&self) -> Result<()> {
        self.repo.prepared(&self.selection)?;
        self.repo.exec.state.borrow_mut().git.staged = true;
        Ok(())
    }

    fn commit(&self) -> Result<()> {
        if self.repo.exec.state.borrow().git.release.is_none() {
            self.repo.commit_check(&self.selection)?;
            let mut state = self.repo.exec.state.borrow_mut();
            let release = state
                .git
                .source
                .keys()
                .map(|path| {
                    (
                        path.clone(),
                        fs::read_to_string(self.repo.root.join(path)).unwrap(),
                    )
                })
                .collect();
            state.git.release = Some(release);
            state
                .calls
                .push(vec!["shared-runner".to_owned(), "commit".to_owned()]);
        }
        self.repo
            .selected_committed_check(&self.selection, "release")?;
        self.repo.exec.state.borrow_mut().git.tag = true;
        Ok(())
    }

    fn release(&self) {
        self.prepare().unwrap();
        self.stage().unwrap();
        self.commit().unwrap();
    }

    fn fail_build(&self, fault: BuildFault) {
        let mut state = self.repo.exec.state.borrow_mut();
        state.clear_faults();
        state.build.fail = Some(fault);
    }

    fn fail_git(&self, fault: GitFault) {
        let mut state = self.repo.exec.state.borrow_mut();
        state.clear_faults();
        state.git.fail = Some(fault);
    }

    fn fail_registry(&self, fault: RegistryFault) {
        let mut state = self.repo.exec.state.borrow_mut();
        state.clear_faults();
        state.registry.fail = Some(fault);
    }

    fn fail_files(&self, fault: FileFault) {
        self.repo.exec.files.borrow_mut().fault = Some(fault);
    }

    fn assert_original(&self) {
        assert_eq!(
            fs::read_to_string(self.repo.root.join("Cargo.toml")).unwrap(),
            MANIFEST
        );
        assert_eq!(
            fs::read_to_string(self.repo.root.join("Cargo.lock")).unwrap(),
            LOCK
        );
        assert!(self.repo.exec.state.borrow().git.release.is_none());
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        if std::thread::panicking() {
            let saved = (|| -> Result<()> {
                let state = self.repo.exec.state.try_borrow()?;
                fs::write(
                    self.repo.root.join("fixture-state.json"),
                    serde_json::to_vec_pretty(&*state)?,
                )?;
                fs::write(
                    self.repo.root.join("fixture-files.json"),
                    serde_json::to_vec_pretty(&*self.repo.exec.files.try_borrow()?)?,
                )?;
                Ok(())
            })();
            if let Err(error) = saved {
                eprintln!("Could not save fixture command state: {error}");
            }
            eprintln!("Release fixture retained: {}", self.repo.root.display());
            return;
        }
        fs::remove_dir_all(&self.repo.root).unwrap();
    }
}

#[test]
fn failed_release_fixture_retains_receipts_and_command_state() {
    let mut retained = PathBuf::new();
    let mut receipt = PathBuf::new();
    let mut file_state = Vec::new();
    let failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let fixture = Fixture::new();
        retained.clone_from(&fixture.repo.root);
        fixture.repo.verify(&fixture.selection).unwrap();
        receipt = fixture
            .repo
            .validation_path(&fixture.selection.version)
            .unwrap();
        fixture.fail_files(FileFault::WriteBefore("Cargo.lock".into()));
        file_state = serde_json::to_vec_pretty(&*fixture.repo.exec.files.borrow()).unwrap();
        panic!("simulate a failed assertion after validation");
    }));
    assert!(failure.is_err());
    let state: State =
        serde_json::from_slice(&fs::read(retained.join("fixture-state.json")).unwrap()).unwrap();
    assert!(
        state
            .calls
            .iter()
            .any(|call| is_make_call(call, "validate"))
    );
    assert_eq!(
        fs::read_to_string(retained.join("Cargo.lock")).unwrap(),
        LOCK
    );
    let evidence: ValidationEvidence = serde_json::from_slice(&fs::read(receipt).unwrap()).unwrap();
    assert_eq!(evidence.selection.source, "source");
    assert_eq!(
        fs::read(retained.join("fixture-files.json")).unwrap(),
        file_state
    );
    fs::remove_dir_all(retained).unwrap();

    let successful = Fixture::new();
    let root = successful.repo.root.clone();
    drop(successful);
    assert!(!root.exists());
}

#[test]
fn fixture_creation_preserves_existing_directories_and_files() {
    let parent = Fixture::new();
    let retained = parent
        .repo
        .root
        .join(format!("ic-memory-tooling-{}-0", std::process::id()));
    fs::create_dir(&retained).unwrap();
    fs::write(retained.join("evidence"), b"retained failure").unwrap();
    let foreign = parent
        .repo
        .root
        .join(format!("ic-memory-tooling-{}-1", std::process::id()));
    fs::write(&foreign, b"unrelated file").unwrap();

    let next = Fixture::new_in(&parent.repo.root);
    assert_eq!(next.repo.version(None).unwrap(), "0.12.3");
    assert_ne!(next.repo.root, retained);
    assert_ne!(next.repo.root, foreign);
    drop(next);
    assert_eq!(
        fs::read(retained.join("evidence")).unwrap(),
        b"retained failure"
    );
    assert_eq!(fs::read(foreign).unwrap(), b"unrelated file");
}

#[test]
fn durable_publication_replaces_complete_files_and_preserves_rejected_targets() {
    let fixture = Fixture::new();
    let target = fixture.repo.root.join("durable/release.json");
    Processes.write_bytes(&target, b"original receipt").unwrap();
    let mut original = fs::File::open(&target).unwrap();
    let source = fixture.repo.root.join("complete-source");
    fs::write(&source, b"complete replacement").unwrap();
    Processes
        .copy_verified(
            &source,
            &target,
            Sha256Digest::compute(b"complete replacement"),
        )
        .unwrap();
    assert_eq!(fs::read(&target).unwrap(), b"complete replacement");
    let mut original_bytes = Vec::new();
    std::io::Read::read_to_end(&mut original, &mut original_bytes).unwrap();
    assert_eq!(original_bytes, b"original receipt");

    let rejected = fixture.repo.root.join("durable/retained-directory");
    fs::create_dir(&rejected).unwrap();
    fs::write(rejected.join("evidence"), "keep").unwrap();
    let entries = || {
        fs::read_dir(target.parent().unwrap())
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect::<std::collections::BTreeSet<_>>()
    };
    let before = entries();
    let error = Processes
        .copy_verified(&source, &target, Sha256Digest::compute(b"different bytes"))
        .unwrap_err();
    assert!(matches!(
        error
            .downcast_ref::<ic_host_fs::durable::NamedWriteError<ic_host_artifacts::artifact::CopyError>>()
            .unwrap(),
        ic_host_fs::durable::NamedWriteError::Producer {
            source,
            cleanup_error: None,
        } if matches!(source, ic_host_artifacts::artifact::CopyError::Input(
            ic_host_artifacts::artifact::ArtifactError::DigestMismatch { .. }
        ))
    ));
    assert_eq!(fs::read(&target).unwrap(), b"complete replacement");
    assert_eq!(entries(), before);

    let error = Processes
        .write_bytes(&rejected, b"invalid replacement")
        .unwrap_err();
    assert!(matches!(
        error
            .downcast_ref::<ic_host_fs::durable::NamedWriteError<std::io::Error>>()
            .unwrap(),
        ic_host_fs::durable::NamedWriteError::BeforePublication {
            cleanup_error: None,
            ..
        }
    ));
    assert_eq!(
        fs::read_to_string(rejected.join("evidence")).unwrap(),
        "keep"
    );
    assert_eq!(fs::read(&target).unwrap(), b"complete replacement");
    assert_eq!(entries(), before);
}

#[test]
fn streamed_receipts_preserve_json_and_reject_partial_serialization() {
    struct RejectedValue;
    impl Serialize for RejectedValue {
        fn serialize<S: serde::Serializer>(&self, _: S) -> std::result::Result<S::Ok, S::Error> {
            Err(serde::ser::Error::custom("reject after the receipt prefix"))
        }
    }

    let fixture = Fixture::new();
    let path = fixture.repo.root.join("receipts/validated.json");
    // Encoding passive receipts needs no simulated validation, package, commit
    // or tag. Lifecycle admission is exercised by the release/recovery tests.
    let validation = ValidationEvidence {
        selection: fixture.selection.clone(),
        lock: LOCK.as_bytes().to_vec(),
        identities: ["cargo", "rustc", "msrv"].map(str::to_owned),
        configuration: BTreeMap::new(),
        command: vec!["validate".to_owned()],
    };
    Processes.write_json(&path, &validation).unwrap();
    let original = fs::read(&path).unwrap();
    Processes.write_json(&path, &validation).unwrap();
    assert_eq!(fs::read(&path).unwrap(), original);
    for head in ["source", "release"] {
        let path = fixture.repo.root.join(format!("receipts/{head}.json"));
        let evidence = PackageEvidence {
            validation: serde_json::from_slice(&original).unwrap(),
            package_head: head.to_owned(),
            lock_sha256: "lock".to_owned(),
            package_sha256: "memory".to_owned(),
            storage_sha256: "storage".to_owned(),
            metrics_sha256: "metrics".to_owned(),
            logging_sha256: "logging".to_owned(),
            commands: vec![vec!["package".to_owned()]],
        };
        assert_eq!(
            serde_json::to_value(&evidence).unwrap(),
            serde_json::json!({
                "validation": serde_json::to_value(&validation).unwrap(),
                "package_head": head,
                "lock_sha256": "lock",
                "package_sha256": "memory",
                "storage_sha256": "storage",
                "metrics_sha256": "metrics",
                "logging_sha256": "logging",
                "commands": [["package"]],
            })
        );
        assert_eq!(
            evidence.archives().collect::<Vec<_>>(),
            [
                ("ic-memory", "memory"),
                ("ic-storage", "storage"),
                ("ic-metrics-stable", "metrics"),
                ("ic-logging-stable", "logging")
            ]
        );
        let mut unknown = serde_json::to_value(&evidence).unwrap();
        unknown["unknown"] = serde_json::json!(true);
        assert!(serde_json::from_value::<PackageEvidence>(unknown).is_err());
        let mut incomplete = serde_json::to_value(&evidence).unwrap();
        incomplete.as_object_mut().unwrap().remove("logging_sha256");
        assert!(serde_json::from_value::<PackageEvidence>(incomplete).is_err());
        Processes.write_json(&path, &evidence).unwrap();
        let original = fs::read(&path).unwrap();
        let evidence: PackageEvidence = serde_json::from_slice(&original).unwrap();
        Processes.write_json(&path, &evidence).unwrap();
        assert_eq!(fs::read(path).unwrap(), original);
    }

    let entries = || fs::read_dir(path.parent().unwrap()).unwrap().count();
    let before = entries();
    let error = Processes
        .write_json(&path, &(&validation, RejectedValue))
        .unwrap_err();
    assert!(matches!(
        error
            .downcast_ref::<ic_host_fs::durable::NamedWriteError<serde_json::Error>>()
            .unwrap(),
        ic_host_fs::durable::NamedWriteError::Producer {
            source,
            cleanup_error: None,
        } if source.is_data()
    ));
    assert_eq!(fs::read(&path).unwrap(), original);
    assert_eq!(entries(), before);
    assert_eq!(
        fixture.repo.exec.state.borrow().calls,
        Vec::<Vec<String>>::new()
    );
}

#[derive(Default, Deserialize, Serialize)]
struct State {
    git: GitState,
    build: BuildEffects,
    registry: RegistryEffects,
    calls: Vec<Vec<String>>,
}

impl State {
    fn clear_faults(&mut self) {
        self.git.fail = None;
        self.build.fail = None;
        self.registry.fail = None;
    }
}

struct Substitute {
    state: RefCell<State>,
    files: RefCell<FileEffects>,
}

impl Execute for Substitute {
    fn cargo_target(&self, root: &Path) -> Result<PathBuf> {
        let mut state = self.state.borrow_mut();
        state.calls.push(vec!["cargo-metadata".to_owned()]);
        state.build.target(root)
    }

    fn write_json(&self, path: &Path, value: &impl Serialize) -> Result<()> {
        // Reuse the fixture's before/after-publication fault injection. The
        // native streaming boundary is exercised separately below.
        self.write_bytes(path, &serde_json::to_vec_pretty(value)?)
    }

    fn copy_verified(&self, source: &Path, path: &Path, expected: Sha256Digest) -> Result<()> {
        self.files.borrow_mut().copy_verified(
            source,
            path,
            expected,
            &mut self.state.borrow_mut().calls,
        )
    }

    fn write_bytes(&self, path: &Path, bytes: &[u8]) -> Result<()> {
        self.files
            .borrow_mut()
            .write_bytes(path, bytes, &mut self.state.borrow_mut().calls)
    }

    fn execute(&self, root: &Path, program: &str, args: &[&str]) -> Result<()> {
        self.capture(root, program, args).map(|_| ())
    }

    fn capture(&self, root: &Path, program: &str, args: &[&str]) -> Result<String> {
        if program == "env" {
            return self.publication(root, args);
        }
        let mut state = self.state.borrow_mut();
        state.calls.push(
            std::iter::once(program)
                .chain(args.iter().copied())
                .map(str::to_owned)
                .collect(),
        );
        let State {
            git,
            build,
            registry,
            calls,
        } = &mut *state;
        match program {
            "git" => git.capture(root, args, calls),
            "bash" if args.first() == Some(&TAG_CHECKER) => git.check_tag(args),
            "curl" => registry.observe(root, args),
            "cargo" if args.get(1) == Some(&"publish") => registry.publish(root, args, git),
            "perl" | "awk" => Processes.capture(root, program, args),
            "bash"
                if args == ["scripts/ci/check-make-execution.sh"]
                    && build.fail == Some(BuildFault::MakeExecution) =>
            {
                build.capture(root, program, args, git)
            }
            "bash" if args.first() != Some(&"scripts/ci/package-workspace.sh") => Processes
                .capture(
                    &Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."),
                    program,
                    args,
                ),
            _ => build.capture(root, program, args, git),
        }
    }
}

impl Substitute {
    fn publication(&self, root: &Path, args: &[&str]) -> Result<String> {
        require(
            args.len() == 10
                && args[0] == "RUSTUP_TOOLCHAIN=1.99.0"
                && args[1] == "bash"
                && args[2] == "scripts/ci/run-cargo-publication.sh"
                && matches!(args[3], "check" | "publish")
                && args[5] == "release"
                && args[6] == "crates-io"
                && args[7] == "--package"
                && args[9] == "--no-verify",
            "unexpected coordinated publication attempt",
        )?;
        let (name, version) = args[8].split_once('@').ok_or("missing exact selector")?;
        require(
            version
                == toml_string(
                    &fs::read_to_string(root.join("Cargo.toml"))?,
                    "workspace.package",
                    "version",
                )?,
            "wrong publication version",
        )?;
        let mut forwarded = vec![
            "+1.99.0",
            "publish",
            "--locked",
            "--registry",
            "crates-io",
            "--package",
            name,
            "--no-verify",
        ];
        if args[3] == "check" {
            forwarded.push("--dry-run");
        }
        self.capture(root, "cargo", &forwarded)
    }
}

#[test]
fn retained_package_paths_require_digest_authority_before_target_discovery() {
    let fixture = Fixture::new();
    for digest in ["", "../other", &"A".repeat(64), &"0".repeat(63)] {
        assert!(fixture.repo.retained_package(digest).is_err());
    }
    assert_eq!(fixture.repo.exec.state.borrow().calls.len(), 0);
    let digest = "0".repeat(64);
    assert_eq!(
        fixture.repo.retained_package(&digest).unwrap(),
        fixture
            .repo
            .root
            .join("configured output/release-validation/artifacts")
            .join(format!("{digest}.crate"))
    );
}

#[test]
fn compiler_environment_overrides_refuse_preparation_and_release_effects() {
    const ROOT: &str = "IC_MEMORY_TEST_COMPILER_ROOT";
    const STATE: &str = "IC_MEMORY_TEST_COMPILER_STATE";
    const OVERRIDES: [&str; 8] = [
        "RUSTC",
        "RUSTC_WRAPPER",
        "RUSTC_WORKSPACE_WRAPPER",
        "RUSTDOC",
        "CARGO_BUILD_RUSTC",
        "CARGO_BUILD_RUSTC_WRAPPER",
        "CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER",
        "CARGO_BUILD_RUSTDOC",
    ];
    if let Some(state) = env::var_os(STATE) {
        // Separate processes exercise real inherited environment without
        // unsafe mutation of the concurrent Rust test harness environment.
        let fixture = Fixture::new();
        let receipt = fixture.repo.receipt_path("0.13.0", Prepared).unwrap();
        fs::create_dir_all(receipt.parent().unwrap()).unwrap();
        fs::write(&receipt, "previous evidence").unwrap();
        assert!(fixture.prepare().is_err());
        fixture.assert_original();
        assert_eq!(fs::read(receipt).unwrap(), b"previous evidence");
        assert!(!fixture.repo.exec.state.borrow().calls.iter().any(|call| {
            call[0] == "make" || is_cargo_call(call, "update") || is_package_call(call)
        }));

        let repo = Repository {
            root: PathBuf::from(env::var_os(ROOT).unwrap()),
            exec: Substitute {
                files: RefCell::new(FileEffects::default()),
                state: RefCell::new(
                    serde_json::from_str::<State>(state.to_str().unwrap()).unwrap(),
                ),
            },
        };
        let call_start = repo.exec.state.borrow().calls.len();
        assert!(repo.qualify_release().is_err());
        assert!(repo.push_check(&fixture.selection, "release").is_err());
        assert!(repo.publish(true).is_err());
        assert!(
            !repo.exec.state.borrow().calls[call_start..]
                .iter()
                .any(|call| {
                    call.get(1).is_some_and(|arg| arg == "push")
                        || (is_package_call(call) || is_cargo_call(call, "publish"))
                })
        );
        return;
    }

    let fixture = Fixture::new();
    fixture.release();
    let state = serde_json::to_string(&*fixture.repo.exec.state.borrow()).unwrap();
    let evidence = fixture
        .repo
        .verify_evidence(&fixture.selection, "release", Final, Working)
        .unwrap();
    let paths = [
        fixture.repo.receipt_path("0.13.0", Prepared).unwrap(),
        fixture.repo.receipt_path("0.13.0", Final).unwrap(),
        fixture.repo.package_path("ic-memory", "0.13.0").unwrap(),
        fixture
            .repo
            .retained_package(&evidence.package_sha256)
            .unwrap(),
    ];
    let before: Vec<_> = paths.iter().map(|path| fs::read(path).unwrap()).collect();
    for name in OVERRIDES {
        for value in ["", "alternate-tool"] {
            let mut child = Command::new(env::current_exe().unwrap());
            child.args(["--exact", "repo_tool::tests::compiler_environment_overrides_refuse_preparation_and_release_effects"]);
            for variable in OVERRIDES {
                child.env_remove(variable);
            }
            let output = child
                .env(ROOT, &fixture.repo.root)
                .env(STATE, &state)
                .env(name, value)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{name}={value:?}: {}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            for (path, bytes) in paths.iter().zip(&before) {
                assert_eq!(fs::read(path).unwrap(), *bytes);
            }
        }
    }
}

#[test]
fn compiler_configuration_overrides_refuse_preparation() {
    for filename in ["config", "config.toml"] {
        for field in [
            "rustc",
            "rustc-wrapper",
            "rustc-workspace-wrapper",
            "rustdoc",
        ] {
            for value in ["", "alternate-tool"] {
                let fixture = Fixture::new();
                let directory = fixture.repo.root.join(".cargo");
                fs::create_dir(&directory).unwrap();
                let path = directory.join(filename);
                let text = format!("[build]\n{field} = {value:?}\n");
                fs::write(&path, &text).unwrap();
                assert!(fixture.prepare().is_err());
                fixture.assert_original();
                assert_eq!(fs::read_to_string(path).unwrap(), text);
                assert!(
                    !fixture
                        .repo
                        .exec
                        .state
                        .borrow()
                        .calls
                        .iter()
                        .any(|call| call[0] == "make")
                );
            }
        }
    }
    let fixture = Fixture::new();
    let directory = fixture.repo.root.join(".cargo");
    fs::create_dir(&directory).unwrap();
    fs::write(
        directory.join("config.toml"),
        "[build]\njobs = 1\n[profile.dev]\nopt-level = 1\n",
    )
    .unwrap();
    fixture.release();
    fixture.repo.publish(true).unwrap();

    let fixture = Fixture::new();
    let directory = fixture.repo.root.join(".cargo");
    fs::create_dir(&directory).unwrap();
    fs::write(
        directory.join("config.toml"),
        "token = 'private-test-value' trailing\n",
    )
    .unwrap();
    let error = fixture.repo.configuration().unwrap_err().to_string();
    assert!(!error.contains("private-test-value"));
}

#[test]
fn configuration_budgets_refuse_before_gates_or_metadata_mutation() {
    for budget in ["file", "total", "files", "depth"] {
        let fixture = Fixture::new();
        let directory = fixture.repo.root.join(".cargo");
        fs::create_dir(&directory).unwrap();
        let root = directory.join("config.toml");
        match budget {
            "file" => {
                fs::write(
                    &root,
                    format!("#{}", "x".repeat(CONFIGURATION_LIMITS.file_bytes)),
                )
                .unwrap();
            }
            "total" | "files" => {
                let count = if budget == "total" {
                    5
                } else {
                    CONFIGURATION_LIMITS.files
                };
                let contents = if budget == "total" {
                    format!("#{}", "x".repeat(CONFIGURATION_LIMITS.file_bytes - 1))
                } else {
                    String::new()
                };
                let includes: Vec<_> = (0..count)
                    .map(|index| format!("part-{index}.toml"))
                    .collect();
                fs::write(&root, format!("include = {includes:?}\n")).unwrap();
                for name in includes {
                    fs::write(directory.join(name), &contents).unwrap();
                }
            }
            "depth" => {
                fs::write(&root, "include = 'part-0.toml'\n").unwrap();
                for index in 0..=CONFIGURATION_LIMITS.depth {
                    fs::write(
                        directory.join(format!("part-{index}.toml")),
                        format!("include = 'part-{}.toml'\n", index + 1),
                    )
                    .unwrap();
                }
                fs::write(
                    directory.join(format!("part-{}.toml", CONFIGURATION_LIMITS.depth + 1)),
                    "",
                )
                .unwrap();
            }
            _ => unreachable!(),
        }
        let before = fs::read(&root).unwrap();
        let error = fixture.prepare().unwrap_err();
        assert!(
            error
                .downcast_ref::<ic_build::cargo::environment::ConfigurationError>()
                .is_some(),
            "{budget}: {error}"
        );
        fixture.assert_original();
        assert_eq!(fs::read(root).unwrap(), before);
        assert!(
            !fixture
                .repo
                .exec
                .state
                .borrow()
                .calls
                .iter()
                .any(|call| call[0] == "make")
        );
    }
}

#[test]
fn included_compiler_configuration_refuses_preparation() {
    let fixture = Fixture::new();
    let directory = fixture.repo.root.join(".cargo");
    fs::create_dir(&directory).unwrap();
    fs::write(directory.join("config.toml"), "include = 'compiler.toml'\n").unwrap();
    fs::write(
        directory.join("compiler.toml"),
        "[build]\nrustc-wrapper = 'outside-receipt-compiler'\n",
    )
    .unwrap();
    assert!(fixture.prepare().is_err());
    fixture.assert_original();
    assert!(
        !fixture
            .repo
            .exec
            .state
            .borrow()
            .calls
            .iter()
            .any(|call| { call[0] == "make" || is_cargo_call(call, "update") })
    );
}

#[test]
fn included_and_shadowed_configuration_changes_invalidate_release_evidence() {
    let fixture = Fixture::new();
    let directory = fixture.repo.root.join(".cargo");
    fs::create_dir(&directory).unwrap();
    let files = [
        (directory.join("config"), "include = 'shared.toml'\n"),
        (directory.join("config.toml"), "[build]\njobs = 1\n"),
        (directory.join("shared.toml"), "[build]\njobs = 2\n"),
    ];
    for (path, bytes) in &files {
        fs::write(path, bytes).unwrap();
    }
    fixture.release();
    let observed = fixture.repo.configuration().unwrap();
    for (path, bytes) in &files {
        assert_eq!(
            observed[&format!("config:{}", path.display())],
            Sha256Digest::compute(bytes.as_bytes()).to_string()
        );
        // Even a comment-only change must invalidate existing raw-byte custody.
        fs::write(path, format!("{bytes}# changed\n")).unwrap();
        let calls = fixture.repo.exec.state.borrow().calls.len();
        assert!(
            fixture
                .repo
                .push_check(&fixture.selection, "release")
                .is_err()
        );
        assert!(fixture.repo.publish(true).is_err());
        assert!(
            !fixture.repo.exec.state.borrow().calls[calls..]
                .iter()
                .any(|call| {
                    call.get(1).is_some_and(|arg| arg == "push") || is_cargo_call(call, "publish")
                })
        );
        fs::write(path, bytes).unwrap();
        fixture
            .repo
            .verify_evidence(&fixture.selection, "release", Final, Working)
            .unwrap();
    }
}

#[test]
fn inherited_compiler_configuration_refuses_preparation() {
    let fixture = Fixture::new();
    let directory = fixture.repo.root.join(".cargo");
    fs::create_dir(&directory).unwrap();
    fs::write(
        directory.join("config.toml"),
        "[build]\nrustc-wrapper = 'alternate-tool'\n",
    )
    .unwrap();
    let root = fixture.repo.root.join("nested");
    fs::create_dir(&root).unwrap();
    let source = fixture.repo.exec.state.borrow().git.source.clone();
    for path in source
        .keys()
        .map(String::as_str)
        .chain(["rust-toolchain.toml"])
    {
        fs::create_dir_all(root.join(path).parent().unwrap()).unwrap();
        fs::copy(fixture.repo.root.join(path), root.join(path)).unwrap();
    }
    let repo = Repository {
        root,
        exec: Substitute {
            files: RefCell::new(FileEffects::default()),
            state: RefCell::new(State {
                git: GitState {
                    source,
                    ..GitState::default()
                },
                ..State::default()
            }),
        },
    };
    assert!(repo.preflight(&fixture.selection).is_err());
    assert_eq!(repo.version(None).unwrap(), "0.12.3");
    assert!(
        !repo
            .exec
            .state
            .borrow()
            .calls
            .iter()
            .any(|call| call[0] == "make")
    );
}

#[test]
fn release_tool_setup_and_offline_admission_precede_gates_and_version_mutation() {
    for phase in [BuildFault::ToolSetup, BuildFault::ToolCheck] {
        let fixture = Fixture::new();
        fixture.fail_build(phase.clone());
        assert!(fixture.prepare().is_err());
        fixture.assert_original();
        assert!(!fixture.repo.exec.state.borrow().calls.iter().any(|call| {
            call.iter()
                .any(|argument| argument == "validate" || argument == "update")
        }));
    }
    let fixture = Fixture::new();
    fixture.prepare().unwrap();
    let state = fixture.repo.exec.state.borrow();
    let position = |argument: &str| {
        state
            .calls
            .iter()
            .position(|call| call.iter().any(|value| value == argument))
            .unwrap()
    };
    assert!(position("fetch") < position("install-tools"));
    assert!(position("install-tools") < position("tools-check"));
    assert!(position("tools-check") < position("validate"));
    assert!(position("validate") < position("update"));
}

#[test]
fn release_preserves_saved_readme_bytes_without_interpreting_examples() {
    for readme in [
        "ic-memory = \"0.34\"\n",
        "# café\r\n  ic-memory='0.1' # retain \"comment\"\r\n",
        "ic-memory = { features = [\"example\"], version = '0.13', default-features = false }\n",
        "ic-memory = \"0.13.0\"",
        "# Guide without a dependency example\n",
        "ic-memory = \"0.12\"\nic-memory = \"0.13\"\n",
        "ic-memory = \"unterminated\n",
        "ic-memory = { path = \"../ic-memory\" }\n",
    ] {
        let fixture = Fixture::new();
        fixture
            .repo
            .exec
            .state
            .borrow_mut()
            .git
            .source
            .insert(PACKAGE_README.to_owned(), readme.to_owned());
        fs::write(fixture.repo.root.join(PACKAGE_README), readme).unwrap();
        // Preserving source bytes must not depend on replacing the same file.
        fixture.fail_files(FileFault::WriteBefore(PACKAGE_README.into()));
        fixture.release();
        fixture.repo.publish(true).unwrap();
        assert_eq!(
            fs::read(fixture.repo.root.join(PACKAGE_README)).unwrap(),
            readme.as_bytes()
        );
    }
}

#[test]
fn release_input_admission_failures_preserve_files_and_receipts() {
    for script in [
        None,
        Some("exit 19;"),
        Some("print \"0\\t13\\n\";"),
        Some("print \"0\\t13\\t0\\textra\\n\";"),
    ] {
        let fixture = Fixture::new();
        let helper = fixture.repo.root.join(RELEASE_INPUT_ADMISSION);
        match script {
            Some(script) => fs::write(helper, script).unwrap(),
            None => fs::remove_file(helper).unwrap(),
        }
        let receipt = fixture.repo.root.join("previous.receipt");
        fs::write(&receipt, b"prior evidence").unwrap();
        assert!(fixture.prepare().is_err());
        fixture.assert_original();
        assert_eq!(fs::read(&receipt).unwrap(), b"prior evidence");
        assert!(!fixture.repo.exec.state.borrow().calls.iter().any(|call| {
            matches!(
                call[0].as_str(),
                "make" | "cargo" | "shared-runner" | "write" | "stream-write"
            )
        }));
    }
    let fixture = Fixture::new();
    fs::write(
        fixture.repo.root.join(RELEASE_INPUT_ADMISSION),
        "print \"2000-01-01\\n\";",
    )
    .unwrap();
    assert!(fixture.repo.validate_date("2026-10-15").is_err());
}

#[test]
fn release_versions_and_pending_notes_preserve_dependency_versions_and_history() {
    let fixture = Fixture::new();
    let finalize = |text: &str, version: &str| {
        fixture.repo.release_changelog(
            &fixture.repo.root.join(CHANGELOG_FINALIZER),
            text,
            "0.12.3",
            version,
            "2026-10-05",
        )
    };
    for invalid in [
        "0.01.3",
        "0.12",
        "0.12.3-dev",
        "0.12.+3",
        "18446744073709551616.0.0",
    ] {
        assert!(fixture.repo.version_parts(invalid).is_err());
    }
    assert_eq!(
        fixture
            .repo
            .version_parts("18446744073709551615.0.0")
            .unwrap(),
        [u64::MAX, 0, 0]
    );
    let updated = fixture
        .repo
        .replace_workspace_version(MANIFEST, "0.12.3", "0.13.0")
        .unwrap();
    assert!(updated.contains("other = \"0.12.3\""));
    let current = "# Changelog\n\n## [0.13.0]\n\nChanges.\n\n## 0.12.3\n\nHistory.\n";
    assert_eq!(
        finalize(current, "0.13.0").unwrap(),
        current.replacen("## [0.13.0]", "## [0.13.0] - 2026-10-05", 1)
    );
    assert!(finalize(current, "0.12.4").is_err());
    assert_eq!(
        finalize("## [0.13.0]\n\n## 0.12.3\nHistory", "0.13.0").unwrap(),
        "## [0.13.0] - 2026-10-05\n\n## 0.12.3\nHistory"
    );
    assert_eq!(
        finalize("## [0.13.0]", "0.13.0").unwrap(),
        "## [0.13.0] - 2026-10-05\n"
    );
    for historical in ["0.13.0", "[0.13.0]", "[0.13.0] - 2026-10-05"] {
        let duplicate = format!("{current}\n## {historical}\n\nAlready released.\n");
        assert!(finalize(&duplicate, "0.13.0").is_err());
    }
    for date in [
        "1900-02-29",
        "2026-02-29",
        "2026-04-31",
        "2026-13-01",
        "2026-10-05\n",
        "0000-01-01",
    ] {
        assert!(fixture.repo.validate_date(date).is_err());
    }
    assert!(fixture.repo.validate_date("2000-02-29").is_ok());
    assert!(fixture.repo.validate_date("2024-02-29").is_ok());
}

#[test]
fn empty_numbered_notes_prepare_without_weakening_release_identity() {
    let fixture = Fixture::new();
    for path in ["CHANGELOG.md", "docs/changelog/0.13.md"] {
        let text =
            "# Notes\n\n## [0.13.0]\n\n## [0.12.3] - 2026-10-04\n\nHistory without a final newline";
        fs::write(fixture.repo.root.join(path), text).unwrap();
        fixture
            .repo
            .exec
            .state
            .borrow_mut()
            .git
            .source
            .insert(path.to_owned(), text.to_owned());
    }
    fixture.prepare().unwrap();
    fixture.stage().unwrap();
    fixture.commit().unwrap();
    fixture.repo.publish(true).unwrap();
    for path in ["CHANGELOG.md", "docs/changelog/0.13.md"] {
        assert_eq!(
            fs::read_to_string(fixture.repo.root.join(path)).unwrap(),
            "# Notes\n\n## [0.13.0] - 2026-10-05\n\n## [0.12.3] - 2026-10-04\n\nHistory without a final newline"
        );
    }
}

#[test]
fn shared_note_finalization_preserves_history_and_refuses_conflicting_targets() {
    for path in ["CHANGELOG.md", "docs/changelog/0.13.md"] {
        let fixture = Fixture::new();
        let edited = format!(
            "{}\nAdditional reviewed notes.\n",
            fixture.repo.exec.state.borrow().git.source[path]
        );
        fs::write(fixture.repo.root.join(path), &edited).unwrap();
        for staged in [false, true] {
            fixture.repo.exec.state.borrow_mut().git.staged = staged;
            let error = fixture
                .repo
                .preflight(&fixture.selection)
                .unwrap_err()
                .to_string();
            assert!(error.starts_with(&format!(
                "{path} does not match the admitted release bytes for source source"
            )));
            assert!(error.contains("git add alone is insufficient"));
            assert_eq!(
                fs::read_to_string(fixture.repo.root.join(path)).unwrap(),
                edited
            );
            assert!(!fixture.repo.exec.state.borrow().calls.iter().any(|call| {
                matches!(
                    call[0].as_str(),
                    "make" | "cargo" | "write" | "stream-write"
                )
            }));
        }
        // Substitute committing the notes into the selected source. No real
        // commit, staging or release effects are performed by this fixture.
        fixture
            .repo
            .exec
            .state
            .borrow_mut()
            .git
            .source
            .insert(path.to_owned(), edited);
        fixture.repo.preflight(&fixture.selection).unwrap();
    }

    for notes in [
        "##\t[0.13.0]  \n\nChanges.\n\n## [0.12.3] - 2026-10-04\nHistory",
        "## [0.13.0] - Unreleased\n\nChanges.\n\n## [0.12.3] - 2026-10-04\nHistory",
        "## [0.13.0]",
        "## [0.12.3]\n\nUndated published history",
    ] {
        let fixture = Fixture::new();
        for path in ["CHANGELOG.md", "docs/changelog/0.13.md"] {
            fs::write(fixture.repo.root.join(path), notes).unwrap();
            fixture
                .repo
                .exec
                .state
                .borrow_mut()
                .git
                .source
                .insert(path.to_owned(), notes.to_owned());
        }
        fixture.repo.preflight(&fixture.selection).unwrap();
        fixture.release();
        fixture.repo.publish(true).unwrap();
        for path in ["CHANGELOG.md", "docs/changelog/0.13.md"] {
            let finalized = fs::read_to_string(fixture.repo.root.join(path)).unwrap();
            assert!(finalized.starts_with("## [0.13.0] - 2026-10-05\n"));
            if let Some(history) = notes.find("## [0.12.3]") {
                assert!(finalized.ends_with(&notes[history..]));
            }
        }
    }

    for conflict in [
        "##\t[0.13.1]  \nDifferent candidate\n",
        "## [0.13.0]\n\n##\t[0.13.0]\t-\t2026-10-04  \nAlready released\n",
    ] {
        let fixture = Fixture::new();
        fs::write(fixture.repo.root.join("CHANGELOG.md"), conflict).unwrap();
        fixture
            .repo
            .exec
            .state
            .borrow_mut()
            .git
            .source
            .insert("CHANGELOG.md".to_owned(), conflict.to_owned());
        assert!(fixture.repo.preflight(&fixture.selection).is_err());
        assert!(fixture.prepare().is_err());
        fixture.assert_original();
    }
}

#[test]
fn failed_shared_note_finalization_never_installs_partial_output() {
    let fixture = Fixture::new();
    let finalizer = fixture.repo.root.join(CHANGELOG_FINALIZER);
    fs::write(&finalizer, "BEGIN { print \"partial output\"; exit 23 }").unwrap();
    let error = fixture.prepare().unwrap_err();
    let Some(ToolError::Execution(failure)) = error.downcast_ref::<ToolError>() else {
        panic!("expected the original typed finalizer failure: {error}");
    };
    assert_eq!(failure.evidence.status.unwrap().code(), Some(23));
    assert_eq!(failure.evidence.stdout, b"partial output\n");
    fixture.assert_original();
    fs::remove_file(finalizer).unwrap();
    assert!(fixture.prepare().is_err());
    fixture.assert_original();
}

#[test]
fn preparation_stage_commit_qualification_and_publish_preserve_exact_evidence() {
    let fixture = Fixture::new();
    fixture.release();
    let prepared: PackageEvidence = serde_json::from_slice(
        &fs::read(fixture.repo.receipt_path("0.13.0", Prepared).unwrap()).unwrap(),
    )
    .unwrap();
    let final_evidence = fixture
        .repo
        .verify_evidence(&fixture.selection, "release", Final, Working)
        .unwrap();
    assert_ne!(prepared.package_sha256, final_evidence.package_sha256);
    assert!(
        fixture
            .repo
            .retained_package(&prepared.package_sha256)
            .unwrap()
            .is_file()
    );
    assert!(
        fixture
            .repo
            .retained_package(&final_evidence.package_sha256)
            .unwrap()
            .is_file()
    );
    assert_eq!(final_evidence.package_head, "release");
    fixture
        .repo
        .tagged_check(&fixture.selection, "release")
        .unwrap();
    assert!(
        fixture
            .repo
            .root
            .join("configured output/release-validation/0.13.0-prepared.json")
            .is_file()
    );
    fixture
        .repo
        .push_check(&fixture.selection, "release")
        .unwrap();
    fixture.repo.publish(true).unwrap();
    let state = fixture.repo.exec.state.borrow();
    assert!(
        !state.calls.iter().any(|call| call[0] == "git"
            && matches!(call[1].as_str(), "add" | "commit" | "tag" | "push"))
    );
    assert!(state.calls.iter().any(|call| call
        == &[
            "cargo",
            "+1.99.0",
            "publish",
            "--locked",
            "--registry",
            "crates-io",
            "--package",
            "ic-memory",
            "--no-verify",
            "--dry-run"
        ]));
    assert!(!state.calls.iter().any(|call| {
        call.iter()
            .any(|arg| arg == "generate-lockfile" || arg == "clean")
    }));
}

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
fn validation_refuses_make_modes_before_gates_and_preserves_evidence() {
    let fixture = Fixture::new();
    fixture.repo.verify(&fixture.selection).unwrap();
    let receipt = fixture.repo.validation_path("0.13.0").unwrap();
    let previous = fs::read(&receipt).unwrap();
    fixture.fail_build(BuildFault::MakeExecution);
    fixture.repo.exec.state.borrow_mut().calls.clear();
    assert!(fixture.repo.verify(&fixture.selection).is_err());
    assert_eq!(fs::read(&receipt).unwrap(), previous);
    assert_eq!(
        fixture.repo.exec.state.borrow().calls,
        vec![vec!["bash", "scripts/ci/check-make-execution.sh"]]
    );
    assert!(!receipt.parent().unwrap().join("attempts").exists());
    fixture.assert_original();
}

#[test]
fn validation_binds_saved_intent_and_retains_previous_attempts() {
    let fixture = Fixture::new();
    assert!(fixture.repo.prepare_selected(&fixture.selection).is_err());
    fixture.assert_original();
    fixture.repo.verify(&fixture.selection).unwrap();
    let receipt = fixture.repo.validation_path("0.13.0").unwrap();
    let first = fs::read(&receipt).unwrap();
    for field in [
        "kind", "version", "previous", "source", "date", "remote", "branch",
    ] {
        let mut selection = serde_json::to_value(&fixture.selection).unwrap();
        selection[field] = serde_json::json!("changed");
        let selection = serde_json::from_value(selection).unwrap();
        assert!(
            fixture.repo.prepare_selected(&selection).is_err(),
            "{field}"
        );
        fixture.assert_original();
        assert_eq!(fs::read(&receipt).unwrap(), first);
    }
    fixture.fail_build(BuildFault::Validate);
    assert!(fixture.repo.verify(&fixture.selection).is_err());
    assert_eq!(fs::read(&receipt).unwrap(), first);
    fixture.repo.exec.state.borrow_mut().clear_faults();
    fixture.repo.verify(&fixture.selection).unwrap();
    let attempts = receipt.parent().unwrap().join("attempts");
    let entries: Vec<_> = fs::read_dir(attempts).unwrap().collect();
    assert_eq!(entries.len(), 1);
    assert_eq!(
        fs::read(entries[0].as_ref().unwrap().path()).unwrap(),
        first
    );
    fixture.assert_original();
}

#[test]
fn validation_lock_cannot_be_rebound_to_an_uncommitted_graph() {
    let fixture = Fixture::new();
    fixture.repo.verify(&fixture.selection).unwrap();
    let path = fixture.repo.validation_path("0.13.0").unwrap();
    let mut evidence: ValidationEvidence =
        serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    let changed = LOCK.replace(
        "name = \"other\"\nversion = \"0.12.3\"",
        "name = \"other\"\nversion = \"0.12.4\"",
    );
    evidence.lock = changed.as_bytes().to_vec();
    fs::write(&path, serde_json::to_vec(&evidence).unwrap()).unwrap();
    fs::write(fixture.repo.root.join("Cargo.lock"), &changed).unwrap();
    assert!(fixture.repo.prepare_selected(&fixture.selection).is_err());
    assert_eq!(fixture.repo.version(None).unwrap(), "0.12.3");
    assert_eq!(
        fs::read_to_string(fixture.repo.root.join("Cargo.lock")).unwrap(),
        changed
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
fn all_release_kinds_finalize_root_and_detail_notes_at_saved_date() {
    for (kind, version, detail) in [
        ("patch", "0.12.4", "docs/changelog/0.12.md"),
        ("minor", "0.13.0", "docs/changelog/0.13.md"),
        ("major", "1.0.0", "docs/changelog/1.0.md"),
    ] {
        let mut fixture = Fixture::new();
        fixture.selection.kind = kind.to_owned();
        fixture.selection.version = version.to_owned();
        for path in ["CHANGELOG.md", detail] {
            let text = format!("# Notes\n\n## [{version}]\n\nChanges.\n\n## 0.12.3\n\nHistory.\n");
            fs::write(fixture.repo.root.join(path), &text).unwrap();
            fixture
                .repo
                .exec
                .state
                .borrow_mut()
                .git
                .source
                .insert(path.to_owned(), text);
        }
        fixture.prepare().unwrap();
        assert_eq!(fixture.repo.version(None).unwrap(), version);
        let files = fixture.repo.surfaces(&fixture.selection).unwrap();
        assert_eq!(
            files.keys().map(String::as_str).collect::<Vec<_>>(),
            [
                "CHANGELOG.md",
                "Cargo.lock",
                "Cargo.toml",
                &package_manifest("ic-logging-stable"),
                PACKAGE_README,
                &package_manifest("ic-metrics-stable"),
                &package_manifest("ic-storage"),
                detail
            ],
        );
        for path in ["CHANGELOG.md", detail] {
            let text = fs::read_to_string(fixture.repo.root.join(path)).unwrap();
            assert!(text.contains(&format!("## [{version}] - 2026-10-05")));
            assert!(text.ends_with("## 0.12.3\n\nHistory.\n"));
        }
        assert!(
            fs::read_to_string(fixture.repo.root.join("Cargo.lock"))
                .unwrap()
                .contains("name = \"other\"\nversion = \"0.12.3\"")
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

#[test]
fn archive_copy_refuses_a_directory_and_preserves_unpublished_state() {
    let fixture = Fixture::new();
    let package = fixture.repo.package_path("ic-memory", "0.13.0").unwrap();
    fs::create_dir_all(package.parent().unwrap()).unwrap();
    fs::write(&package, b"archive").unwrap();
    let digest = Sha256Digest::compute(b"archive").to_string();
    let retained = fixture.repo.retained_package(&digest).unwrap();
    fs::create_dir_all(retained.parent().unwrap()).unwrap();
    let foreign = retained.with_extension("crate.tmp");
    fs::write(&foreign, b"unrelated evidence").unwrap();
    fixture.repo.exec.files.borrow_mut().archive_change = Some(package);
    fixture.fail_files(FileFault::ArchiveRead);

    let error = fixture
        .repo
        .record_package("ic-memory", "0.13.0")
        .unwrap_err();
    let ic_host_fs::durable::NamedWriteError::Producer {
        source: actual,
        cleanup_error: None,
    } = error
        .downcast_ref::<ic_host_fs::durable::NamedWriteError<ic_host_artifacts::artifact::CopyError>>()
        .unwrap()
    else {
        panic!("archive read failure must retain its producer error before publication");
    };
    assert!(matches!(
        actual,
        ic_host_artifacts::artifact::CopyError::Input(
            ic_host_artifacts::artifact::ArtifactError::NotRegularFile
        )
    ));
    assert!(!retained.exists());
    assert_eq!(fs::read(&foreign).unwrap(), b"unrelated evidence");
    assert_eq!(fs::read_dir(retained.parent().unwrap()).unwrap().count(), 1);
    assert!(
        !fixture
            .repo
            .receipt_path("0.13.0", Prepared)
            .unwrap()
            .exists()
    );
}

#[test]
fn changed_archive_source_stops_before_publication_or_receipt() {
    let fixture = Fixture::new();
    fixture.repo.verify(&fixture.selection).unwrap();
    let validation = fixture.repo.validation_path("0.13.0").unwrap();
    let validation_bytes = fs::read(&validation).unwrap();
    let package = fixture.repo.package_path("ic-memory", "0.13.0").unwrap();
    fixture.repo.exec.files.borrow_mut().archive_change = Some(package.clone());
    assert!(fixture.repo.prepare_selected(&fixture.selection).is_err());
    fixture.assert_original();
    let digest = Sha256Digest::compute(b"prepared-archive").to_string();
    assert!(!fixture.repo.retained_package(&digest).unwrap().exists());
    assert!(
        !fixture
            .repo
            .receipt_path("0.13.0", Prepared)
            .unwrap()
            .exists()
    );
    assert_eq!(fs::read(&validation).unwrap(), validation_bytes);
    assert_eq!(
        fs::read(&package).unwrap(),
        b"changed after digest admission"
    );
    fixture.repo.prepare_selected(&fixture.selection).unwrap();
    assert_eq!(
        fixture
            .repo
            .prepared(&fixture.selection)
            .unwrap()
            .package_sha256,
        digest
    );
}

#[test]
fn source_admission_reports_actual_status_and_preserves_git_failures() {
    let fixture = Fixture::new();
    let checkout = fixture.repo.root.join("status-checkout");
    Processes
        .capture(
            &fixture.repo.root,
            "git",
            &[
                "clone",
                "--quiet",
                "--shared",
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../..")
                    .to_str()
                    .unwrap(),
                checkout.to_str().unwrap(),
            ],
        )
        .unwrap();
    let repo = Repository {
        root: checkout,
        exec: Processes,
    };
    // Discard stat-cache entries without changing the tree, so a plain status
    // observation would refresh the index even though the checkout is clean.
    repo.git(&["read-tree", "HEAD"]).unwrap();
    repo.git(&["checkout-index", "--all", "--force"]).unwrap();
    let clean_index = fs::read(repo.root.join(".git/index")).unwrap();
    repo.clean().unwrap();
    assert_eq!(fs::read(repo.root.join(".git/index")).unwrap(), clean_index);
    let lock = format!(
        "{}\n# staged fixture\n",
        repo.text("Cargo.lock", None).unwrap()
    );
    fs::write(repo.root.join("Cargo.lock"), &lock).unwrap();
    repo.git(&["add", "--force", "--", "Cargo.lock"]).unwrap();
    let manifest = format!(
        "{}\n# working fixture\n",
        repo.text("Cargo.toml", None).unwrap()
    );
    fs::write(repo.root.join("Cargo.toml"), &manifest).unwrap();
    let untracked = "untracked\nfile";
    fs::write(repo.root.join(untracked), b"untracked fixture").unwrap();
    let tree = repo.git(&["write-tree"]).unwrap();
    let status = repo
        .output(
            "git",
            &[
                "--no-optional-locks",
                "status",
                "--porcelain",
                "--untracked-files=all",
            ],
        )
        .unwrap();
    assert!(status.contains("M  Cargo.lock"));
    assert!(status.contains(" M Cargo.toml"));
    assert!(status.contains("?? \"untracked\\nfile\""));
    let dirty_index = fs::read(repo.root.join(".git/index")).unwrap();
    let error = repo.clean().unwrap_err().to_string();
    assert!(error.ends_with(status.trim_end()));
    assert_eq!(fs::read(repo.root.join(".git/index")).unwrap(), dirty_index);
    assert_eq!(repo.git(&["write-tree"]).unwrap(), tree);
    assert_eq!(repo.text("Cargo.lock", None).unwrap(), lock);
    assert_eq!(repo.text("Cargo.toml", None).unwrap(), manifest);
    assert_eq!(
        fs::read(repo.root.join(untracked)).unwrap(),
        b"untracked fixture"
    );

    fixture.fail_git(GitFault::Status);
    assert_eq!(
        fixture.repo.clean().unwrap_err().to_string(),
        "fixture Git status observation failure"
    );
    assert_eq!(
        fixture.repo.exec.state.borrow().calls,
        [vec![
            "git",
            "--no-optional-locks",
            "status",
            "--porcelain",
            "--untracked-files=all"
        ]]
    );
    assert_eq!(fixture.repo.version(None).unwrap(), "0.12.3");
}

#[test]
fn release_admission_rejects_whitespace_only_paths() {
    let fixture = Fixture::new();
    let repo = Repository {
        root: fixture.repo.root.clone(),
        exec: Processes,
    };
    fs::create_dir_all(repo.root.join("scripts/ci")).unwrap();
    for path in [
        "scripts/ci/next-release-version.sh",
        "scripts/ci/validate-release-inputs.pl",
        "scripts/ci/StableVersion.pm",
    ] {
        fs::copy(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .join(path),
            repo.root.join(path),
        )
        .unwrap();
    }
    fs::write(repo.root.join(" "), b"original tracked bytes\n").unwrap();
    repo.git(&["init", "--quiet"]).unwrap();
    repo.git(&["add", "--", "."]).unwrap();
    let selection = ReleaseSelection {
        source: repo.git(&["write-tree"]).unwrap(),
        ..fixture.selection.clone()
    };
    let surfaces = repo.surfaces(&selection).unwrap();
    repo.check_surfaces(&selection, &repo.surfaces(&selection).unwrap(), None, true)
        .unwrap();
    repo.check_index(&selection, &surfaces).unwrap();

    fs::write(repo.root.join("  "), b"retained untracked bytes\n").unwrap();
    assert_eq!(
        repo.check_surfaces(&selection, &repo.surfaces(&selection).unwrap(), None, true)
            .unwrap_err()
            .to_string(),
        "release candidate has untracked files"
    );
    assert_eq!(repo.git(&["write-tree"]).unwrap(), selection.source);
    assert_eq!(
        fs::read(repo.root.join("  ")).unwrap(),
        b"retained untracked bytes\n"
    );
    fs::remove_file(repo.root.join("  ")).unwrap();

    fs::write(repo.root.join(" "), b"changed tracked bytes\n").unwrap();
    assert_eq!(
        repo.check_surfaces(&selection, &repo.surfaces(&selection).unwrap(), None, true)
            .unwrap_err()
            .to_string(),
        "release candidate contains changes outside its version surfaces"
    );
    assert_eq!(repo.git(&["write-tree"]).unwrap(), selection.source);
    assert_eq!(
        fs::read(repo.root.join(" ")).unwrap(),
        b"changed tracked bytes\n"
    );

    repo.git(&["add", "--", " "]).unwrap();
    let staged = repo.git(&["write-tree"]).unwrap();
    fs::write(repo.root.join(" "), b"original tracked bytes\n").unwrap();
    repo.check_surfaces(&selection, &repo.surfaces(&selection).unwrap(), None, true)
        .unwrap();
    assert_eq!(
        repo.check_index(&selection, &surfaces)
            .unwrap_err()
            .to_string(),
        "index contains unrelated release changes"
    );
    assert_eq!(repo.git(&["write-tree"]).unwrap(), staged);
    assert_eq!(
        fs::read(repo.root.join(" ")).unwrap(),
        b"original tracked bytes\n"
    );
}

#[test]
fn dirty_sources_and_changed_validation_inputs_prevent_version_edits() {
    let fixture = Fixture::new();
    fs::write(fixture.repo.root.join(PACKAGE_README), "unfinished").unwrap();
    assert!(fixture.prepare().is_err());
    assert_eq!(
        fs::read_to_string(fixture.repo.root.join("Cargo.toml")).unwrap(),
        MANIFEST
    );
    let fixture = Fixture::new();
    fixture.fail_build(BuildFault::ValidationLock);
    assert!(fixture.prepare().is_err());
    assert_eq!(
        fs::read_to_string(fixture.repo.root.join("Cargo.toml")).unwrap(),
        MANIFEST
    );
}

#[test]
fn lockfile_changes_during_post_validation_remote_inspection_stop_before_mutation() {
    let fixture = Fixture::new();
    fixture.fail_git(GitFault::RemoteLock);
    assert!(fixture.prepare().is_err());
    assert_eq!(fixture.repo.version(None).unwrap(), "0.12.3");
    for (path, text) in &fixture.repo.exec.state.borrow().git.source {
        if path == "Cargo.lock" {
            continue;
        }
        assert_eq!(
            fs::read_to_string(fixture.repo.root.join(path)).unwrap(),
            *text
        );
    }
    // Preserve the separately changed lock selection; it is not our version edit.
    assert_eq!(
        fs::read_to_string(fixture.repo.root.join("Cargo.lock")).unwrap(),
        LOCK.replace(
            "name = \"other\"\nversion = \"0.12.3\"",
            "name = \"other\"\nversion = \"0.12.4\""
        )
    );
    assert!(
        !fixture
            .repo
            .receipt_path("0.13.0", Prepared)
            .unwrap()
            .exists()
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
}

#[test]
fn changed_missing_and_wrong_source_receipts_never_dispatch_publication() {
    for changed in [
        "lock",
        "package",
        "receipt",
        "compiler",
        "source",
        "commands",
        "tag",
        "configuration",
        "retained",
    ] {
        let fixture = Fixture::new();
        fixture.release();
        let receipt = fixture.repo.receipt_path("0.13.0", Final).unwrap();
        match changed {
            "lock" => {
                fs::remove_file(fixture.repo.root.join("Cargo.lock")).unwrap();
            }
            "package" => {
                fs::write(
                    fixture.repo.package_path("ic-memory", "0.13.0").unwrap(),
                    "changed artifact",
                )
                .unwrap();
            }
            "receipt" => {
                fs::remove_file(receipt).unwrap();
            }
            "retained" => {
                let evidence: PackageEvidence =
                    serde_json::from_slice(&fs::read(&receipt).unwrap()).unwrap();
                fs::remove_file(
                    fixture
                        .repo
                        .retained_package(&evidence.package_sha256)
                        .unwrap(),
                )
                .unwrap();
            }
            "tag" => {
                fixture.repo.exec.state.borrow_mut().git.tag = false;
            }
            field => {
                let mut evidence: serde_json::Value =
                    serde_json::from_slice(&fs::read(&receipt).unwrap()).unwrap();
                if field == "configuration" {
                    evidence["validation"][field] =
                        serde_json::json!({"env:RUSTFLAGS": "--cfg unqualified"});
                } else if field == "commands" {
                    evidence[field] = serde_json::json!([]);
                } else if field == "compiler" {
                    evidence["validation"]["identities"][1] = serde_json::json!("changed");
                } else {
                    evidence["validation"]["selection"][field] = serde_json::json!("changed");
                }
                fs::write(receipt, serde_json::to_vec(&evidence).unwrap()).unwrap();
            }
        }
        if changed == "tag" {
            assert!(
                fixture
                    .repo
                    .tagged_check(&fixture.selection, "release")
                    .is_err()
            );
        }
        assert!(fixture.repo.publish(true).is_err());
        assert!(
            !fixture
                .repo
                .exec
                .state
                .borrow()
                .calls
                .iter()
                .any(|call| is_cargo_call(call, "publish"))
        );
    }
}

#[test]
fn publication_rechecks_tag_identity_without_replaying_effects_or_replacing_evidence() {
    for change in [
        GitFault::PublishBeforeTag,
        GitFault::PublishTagMissing,
        GitFault::PublishTagCommit,
        GitFault::PublishTagObject,
    ] {
        let fixture = Fixture::new();
        fixture.release();
        let receipt = fixture.repo.receipt_path("0.13.0", Final).unwrap();
        let before = fs::read(&receipt).unwrap();
        fixture.fail_git(change.clone());
        assert!(fixture.repo.publish(true).is_err(), "accepted {change:?}");
        assert_eq!(fs::read(&receipt).unwrap(), before);
        assert_eq!(
            fixture
                .repo
                .exec
                .state
                .borrow()
                .calls
                .iter()
                .filter(|call| { is_cargo_call(call, "publish") })
                .count(),
            usize::from(change != GitFault::PublishBeforeTag)
        );
    }
}

#[test]
fn invalid_prepared_artifacts_stop_final_qualification_without_replacing_evidence() {
    for changed in ["head", "missing", "corrupted"] {
        let fixture = Fixture::new();
        fixture.release();
        let prepared_path = fixture.repo.receipt_path("0.13.0", Prepared).unwrap();
        let mut prepared: PackageEvidence =
            serde_json::from_slice(&fs::read(&prepared_path).unwrap()).unwrap();
        let retained = fixture
            .repo
            .retained_package(&prepared.package_sha256)
            .unwrap();
        match changed {
            "head" => {
                prepared.package_head = "unvalidated-head".to_owned();
                fs::write(&prepared_path, serde_json::to_vec(&prepared).unwrap()).unwrap();
            }
            "missing" => fs::remove_file(&retained).unwrap(),
            "corrupted" => fs::write(&retained, "corrupted prepared archive").unwrap(),
            _ => unreachable!(),
        }
        let final_path = fixture.repo.receipt_path("0.13.0", Final).unwrap();
        let final_bytes = fs::read(&final_path).unwrap();
        let final_evidence: PackageEvidence = serde_json::from_slice(&final_bytes).unwrap();
        let final_archive = fixture
            .repo
            .retained_package(&final_evidence.package_sha256)
            .unwrap();
        let archive_bytes = fs::read(&final_archive).unwrap();
        let working_archive = fixture.repo.package_path("ic-memory", "0.13.0").unwrap();
        let working_bytes = fs::read(&working_archive).unwrap();
        let prepared_bytes = fs::read(&prepared_path).unwrap();
        let call_start = fixture.repo.exec.state.borrow().calls.len();

        assert!(fixture.repo.qualify_release().is_err(), "{changed}");
        assert_eq!(fs::read(&final_path).unwrap(), final_bytes);
        assert_eq!(fs::read(&final_archive).unwrap(), archive_bytes);
        assert_eq!(fs::read(&working_archive).unwrap(), working_bytes);
        assert_eq!(fs::read(&prepared_path).unwrap(), prepared_bytes);
        assert!(
            !fixture.repo.exec.state.borrow().calls[call_start..]
                .iter()
                .any(|call| is_package_call(call))
        );
    }
}

#[test]
fn final_package_failure_is_retryable_without_a_second_commit() {
    let fixture = Fixture::new();
    fixture.prepare().unwrap();
    fixture.stage().unwrap();
    fixture.fail_build(BuildFault::Package);
    assert!(fixture.commit().is_err());
    assert!(fixture.repo.exec.state.borrow().git.release.is_some());
    assert!(!fixture.repo.exec.state.borrow().git.tag);
    assert!(!fixture.repo.receipt_path("0.13.0", Final).unwrap().exists());
    fixture.repo.exec.state.borrow_mut().clear_faults();
    fixture.commit().unwrap();
    assert_eq!(
        fixture
            .repo
            .exec
            .state
            .borrow()
            .calls
            .iter()
            .filter(|call| call.get(1).is_some_and(|arg| arg == "commit"))
            .count(),
        1
    );
}

#[test]
fn head_change_during_final_packaging_preserves_prepared_evidence() {
    let fixture = Fixture::new();
    fixture.prepare().unwrap();
    fixture.stage().unwrap();
    let path = fixture.repo.receipt_path("0.13.0", Prepared).unwrap();
    let prepared_bytes = fs::read(&path).unwrap();
    let prepared: PackageEvidence = serde_json::from_slice(&prepared_bytes).unwrap();
    let archive = fixture
        .repo
        .retained_package(&prepared.package_sha256)
        .unwrap();
    let archive_bytes = fs::read(&archive).unwrap();
    fixture.fail_build(BuildFault::PackageHead);
    assert!(fixture.commit().is_err());
    assert!(fixture.repo.exec.state.borrow().git.descendant.is_some());
    assert!(!fixture.repo.exec.state.borrow().git.tag);
    assert!(!fixture.repo.receipt_path("0.13.0", Final).unwrap().exists());
    assert_eq!(fs::read(&path).unwrap(), prepared_bytes);
    assert_eq!(fs::read(&archive).unwrap(), archive_bytes);
}

#[test]
fn selected_older_release_reuses_exact_receipts_and_archives_after_a_fix() {
    let fixture = Fixture::new();
    fixture.release();
    let prepared_path = fixture.repo.receipt_path("0.13.0", Prepared).unwrap();
    let final_path = fixture.repo.receipt_path("0.13.0", Final).unwrap();
    let prepared_bytes = fs::read(&prepared_path).unwrap();
    let final_bytes = fs::read(&final_path).unwrap();
    let evidence: PackageEvidence = serde_json::from_slice(&final_bytes).unwrap();
    let archive = fixture
        .repo
        .retained_package(&evidence.package_sha256)
        .unwrap();
    let archive_bytes = fs::read(&archive).unwrap();
    let mut descendant = {
        let state = fixture.repo.exec.state.borrow();
        state.git.release.clone().unwrap()
    };
    descendant.insert(
        "source.rs".to_owned(),
        "// committed callback fix\n".to_owned(),
    );
    for (path, text) in &descendant {
        fs::write(fixture.repo.root.join(path), text).unwrap();
    }
    fixture.repo.exec.state.borrow_mut().git.descendant = Some(descendant);
    fs::write(
        fixture.repo.package_path("ic-memory", "0.13.0").unwrap(),
        "newer working archive",
    )
    .unwrap();
    let calls = fixture.repo.exec.state.borrow().calls.len();
    fixture
        .repo
        .selected_committed_check(&fixture.selection, "release")
        .unwrap();
    fixture
        .repo
        .tagged_check(&fixture.selection, "release")
        .unwrap();
    fixture
        .repo
        .push_check(&fixture.selection, "release")
        .unwrap();
    assert!(fixture.repo.release_commit().is_err());
    assert!(
        fixture
            .repo
            .selected_committed_check(&fixture.selection, "fix")
            .is_err()
    );
    assert_eq!(fs::read(&prepared_path).unwrap(), prepared_bytes);
    assert_eq!(fs::read(&final_path).unwrap(), final_bytes);
    assert_eq!(fs::read(&archive).unwrap(), archive_bytes);
    assert!(
        !fixture.repo.exec.state.borrow().calls[calls..]
            .iter()
            .any(|call| {
                call[0] == "make"
                    || is_package_call(call)
                    || is_cargo_call(call, "publish")
                    || is_cargo_call(call, "update")
            })
    );

    for conflict in [GitFault::History, GitFault::Parent, GitFault::TagCommit] {
        fixture.fail_git(conflict.clone());
        assert!(
            fixture
                .repo
                .push_check(&fixture.selection, "release")
                .is_err(),
            "{conflict:?}"
        );
        assert_eq!(fs::read(&final_path).unwrap(), final_bytes);
    }
    fixture.repo.exec.state.borrow_mut().clear_faults();
    fs::write(&archive, "corrupted retained archive").unwrap();
    assert!(
        fixture
            .repo
            .push_check(&fixture.selection, "release")
            .is_err()
    );
    fs::write(&archive, archive_bytes).unwrap();
    fs::remove_file(&final_path).unwrap();
    let calls = fixture.repo.exec.state.borrow().calls.len();
    assert!(
        fixture
            .repo
            .selected_committed_check(&fixture.selection, "release")
            .is_err()
    );
    assert!(!final_path.exists());
    assert!(
        !fixture.repo.exec.state.borrow().calls[calls..]
            .iter()
            .any(|call| is_package_call(call))
    );
}

#[test]
fn real_index_rejects_hidden_staging_and_checks_exact_prepared_bytes() {
    let fixture = Fixture::new();
    let checkout = fixture.repo.root.join("index-checkout");
    Processes
        .capture(
            &fixture.repo.root,
            "git",
            &[
                "clone",
                "--quiet",
                "--shared",
                "--no-checkout",
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../..")
                    .to_str()
                    .unwrap(),
                checkout.to_str().unwrap(),
            ],
        )
        .unwrap();
    let repo = Repository {
        root: checkout,
        exec: Processes,
    };
    let mut selection = fixture.selection.clone();
    selection.source = repo.git(&["rev-parse", "HEAD"]).unwrap();
    repo.git(&["read-tree", "HEAD"]).unwrap();
    // Give this real index fixture a tracked lock in an uncommitted tree object.
    // Existing repository history is reused; no commit or tag is created.
    fs::write(repo.root.join("Cargo.lock"), LOCK).unwrap();
    repo.git(&["add", "--force", "--", "Cargo.lock"]).unwrap();
    selection.source = repo.git(&["write-tree"]).unwrap();
    repo.git(&["read-tree", &selection.source]).unwrap();
    let original = repo.text("README.md", Some("HEAD")).unwrap();
    let expected = format!("{original}\nPrepared release fixture.\n");
    let locked = Repository::<Processes>::replace_lock_version(LOCK, "0.12.3", "0.13.0").unwrap();
    let surfaces = BTreeMap::from([
        (
            "README.md".to_owned(),
            ReleaseSurface {
                original: original.clone(),
                expected: expected.clone(),
            },
        ),
        (
            "Cargo.lock".to_owned(),
            ReleaseSurface {
                original: LOCK.to_owned(),
                expected: locked.clone(),
            },
        ),
    ]);
    repo.check_index(&selection, &surfaces).unwrap();

    fs::write(repo.root.join("README.md"), &original).unwrap();
    repo.git(&["update-index", "--chmod=+x", "README.md"])
        .unwrap();
    assert!(repo.check_index(&selection, &surfaces).is_err());
    repo.git(&["read-tree", &selection.source]).unwrap();

    fs::write(repo.root.join("unrelated.rs"), "unrelated index change").unwrap();
    repo.git(&["add", "--", "unrelated.rs"]).unwrap();
    fs::remove_file(repo.root.join("unrelated.rs")).unwrap();
    assert!(repo.check_index(&selection, &surfaces).is_err());
    repo.git(&["read-tree", &selection.source]).unwrap();

    for (staged, accepted) in [
        ("arbitrary staged metadata".to_owned(), false),
        (expected, true),
        (format!("{original} "), false),
    ] {
        fs::write(repo.root.join("README.md"), staged).unwrap();
        repo.git(&["add", "--", "README.md"]).unwrap();
        fs::write(repo.root.join("README.md"), &original).unwrap();
        assert_eq!(repo.check_index(&selection, &surfaces).is_ok(), accepted);
        repo.git(&["read-tree", &selection.source]).unwrap();
    }
    for (staged, accepted) in [
        (locked.clone(), true),
        (
            locked.replace(
                "name = \"other\"\nversion = \"0.12.3\"",
                "name = \"other\"\nversion = \"0.12.4\"",
            ),
            false,
        ),
        (format!("{locked}# arbitrary staged text\n"), false),
    ] {
        fs::write(repo.root.join("Cargo.lock"), staged).unwrap();
        repo.git(&["add", "--force", "--", "Cargo.lock"]).unwrap();
        fs::write(repo.root.join("Cargo.lock"), LOCK).unwrap();
        assert_eq!(repo.check_index(&selection, &surfaces).is_ok(), accepted);
        repo.git(&["read-tree", &selection.source]).unwrap();
    }
}

#[test]
fn wasm_metadata_and_budgets_use_only_the_selected_artifact_directory() {
    let fixture = Fixture::new();
    let target = fixture.repo.target().unwrap();
    let artifacts = target.join("wasm32-unknown-unknown/wasm-size/examples");
    fs::create_dir_all(&artifacts).unwrap();
    for (probe, budget) in PROBES {
        let path = artifacts.join(format!("wasm_{probe}_size_probe.wasm"));
        let file = fs::File::create(path).unwrap();
        file.set_len(budget).unwrap();
    }
    fixture.repo.wasm_size().unwrap();
    let calls = fixture.repo.exec.state.borrow().calls.clone();
    let build = calls
        .iter()
        .find(|call| is_cargo_call(call, "build"))
        .unwrap();
    assert_eq!(
        build
            .iter()
            .filter(|arg| arg.as_str() == "--example")
            .count(),
        4
    );
    let path = artifacts.join("wasm_core_size_probe.wasm");
    fs::File::create(&path).unwrap().set_len(260_001).unwrap();
    assert!(check_wasm_artifacts(&target).is_err());
    fs::remove_file(path).unwrap();
    assert!(check_wasm_artifacts(&target).is_err());
    fixture.fail_build(BuildFault::RelativeMetadata);
    assert!(fixture.repo.target().is_err());
    fixture.fail_build(BuildFault::Metadata);
    assert!(fixture.repo.target().is_err());
}

#[test]
fn patch_preparation_keeps_head_and_rejects_unvalidated_source_or_staging() {
    let mut fixture = Fixture::new();
    let path = "CHANGELOG.md";
    let notes = fixture.repo.exec.state.borrow().git.source[path].replace("[0.13.0]", "[0.12.4]");
    fs::write(fixture.repo.root.join(path), &notes).unwrap();
    fixture
        .repo
        .exec
        .state
        .borrow_mut()
        .git
        .source
        .insert(path.to_owned(), notes);
    fixture.selection.kind = "patch".to_owned();
    fixture.selection.version = "0.12.4".to_owned();
    let detail = "docs/changelog/0.12.md";
    let notes = "# Detail\n\n## [0.12.4]\n\nFixture changes.\n";
    fs::write(fixture.repo.root.join(detail), notes).unwrap();
    fixture
        .repo
        .exec
        .state
        .borrow_mut()
        .git
        .source
        .insert(detail.to_owned(), notes.to_owned());
    fixture.prepare().unwrap();
    assert!(fixture.repo.exec.state.borrow().git.release.is_none());
    assert_eq!(fixture.repo.version(None).unwrap(), "0.12.4");
    assert!(!fixture.repo.exec.state.borrow().calls.iter().any(|call| {
        call.get(1)
            .is_some_and(|arg| matches!(arg.as_str(), "commit" | "tag" | "push"))
    }));
    let fixture = Fixture::new();
    fixture.prepare().unwrap();
    assert!(fixture.commit().is_err()); // Expected edits are not staged.
    fixture.stage().unwrap();
    fs::write(
        fixture.repo.root.join("source.rs"),
        "// unvalidated source\n",
    )
    .unwrap();
    assert!(fixture.commit().is_err());
    assert!(fixture.repo.exec.state.borrow().git.release.is_none());
    let fixture = Fixture::new();
    fixture.fail_git(GitFault::RemoteAhead);
    assert!(fixture.prepare().is_err());
    fixture.assert_original();
}

#[test]
fn coordinated_publication_resumes_only_matching_packages_in_dependency_order() {
    let fixture = Fixture::new();
    fixture.release();
    fixture.fail_registry(RegistryFault::StorageUpload);
    assert!(fixture.repo.publish(false).is_err());
    assert_eq!(fixture.repo.exec.state.borrow().registry.published.len(), 1);
    assert!(
        fixture
            .repo
            .exec
            .state
            .borrow()
            .registry
            .published
            .contains_key("ic-memory")
    );
    fixture.repo.exec.state.borrow_mut().clear_faults();
    fixture.repo.publish(false).unwrap();
    fixture.repo.publish(false).unwrap();
    let state = fixture.repo.exec.state.borrow();
    let uploads: Vec<_> = state
        .calls
        .iter()
        .filter(|call| is_cargo_call(call, "publish"))
        .map(|call| call[call.iter().position(|arg| arg == "--package").unwrap() + 1].as_str())
        .collect();
    assert_eq!(
        uploads,
        [
            "ic-memory",
            "ic-storage",
            "ic-storage",
            "ic-metrics-stable",
            "ic-logging-stable"
        ]
    );
    assert_eq!(state.registry.published.len(), packages().len());
    let requests: Vec<_> = state
        .calls
        .iter()
        .filter(|call| call[0] == "curl")
        .collect();
    assert_ne!(requests, [] as [&Vec<String>; 0]);
    for request in requests {
        assert!(request.windows(2).any(|args| {
            args[0] == "--user-agent"
                && args[1]
                    == "ic-persistence-release (https://github.com/dragginzgame/ic-persistence)"
        }));
    }
}

#[test]
fn storage_archive_and_manifest_are_bound_to_the_coordinated_release() {
    for (package, path) in [
        ("ic-storage", package_manifest("ic-storage")),
        ("ic-metrics-stable", package_manifest("ic-metrics-stable")),
        ("ic-logging-stable", package_manifest("ic-logging-stable")),
    ] {
        let fixture = Fixture::new();
        fixture.release();
        assert_eq!(
            toml::from_str::<toml::Value>(
                &fs::read_to_string(fixture.repo.root.join(path)).unwrap()
            )
            .unwrap()["package"]["version"]["workspace"]
                .as_bool(),
            Some(true)
        );
        let manifest: toml::Value =
            toml::from_str(&fs::read_to_string(fixture.repo.root.join("Cargo.toml")).unwrap())
                .unwrap();
        assert_eq!(
            manifest["workspace"]["dependencies"]["ic-memory"]["version"].as_str(),
            Some("0.13.0")
        );
        assert_eq!(
            manifest["workspace"]["dependencies"]["ic-storage"]["version"].as_str(),
            Some("0.13.0")
        );
        fs::write(
            fixture.repo.package_path(package, "0.13.0").unwrap(),
            b"foreign archive",
        )
        .unwrap();
        assert!(fixture.repo.publish(true).is_err());
    }
}

mod publication;

#[test]
fn process_capture_rejects_oversized_output_instead_of_returning_truncation() {
    let root = std::env::current_dir().unwrap();
    assert_eq!(
        Processes.execute(&root, "sh", &["-c", "exit 0"]).unwrap(),
        ()
    );
    assert!(Processes.execute(&root, "sh", &["-c", "exit 23"]).is_err());
    assert_eq!(
        Processes
            .capture(&root, "sh", &["-c", "printf complete"])
            .unwrap(),
        "complete"
    );
    for command in [
        "dd if=/dev/zero bs=1048576 count=9 2>/dev/null",
        "dd if=/dev/zero bs=1048576 count=1 1>&2 2>/dev/null",
    ] {
        let error = Processes
            .capture(&root, "sh", &["-c", command])
            .unwrap_err();
        assert!(matches!(
            error.downcast_ref::<ToolError>(),
            Some(ToolError::Execution(error))
                if matches!(error.failure, ic_host_process::tool::ExecutionFailure::OutputLimit { .. })
        ));
    }
}
