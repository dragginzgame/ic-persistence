//! Process io release contracts.

use super::*;

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
            "repo_tool::tests::process_io::package_retention_refuses_a_replaced_fifo_without_a_writer",
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
            archives: BTreeMap::from([
                ("ic-memory".to_owned(), "memory".to_owned()),
                ("ic-storage".to_owned(), "storage".to_owned()),
            ]),
            commands: vec![vec!["package".to_owned()]],
        };
        assert_eq!(
            serde_json::to_value(&evidence).unwrap(),
            serde_json::json!({
                "validation": serde_json::to_value(&validation).unwrap(),
                "package_head": head,
                "lock_sha256": "lock",
                "archives": {"ic-memory":"memory", "ic-storage":"storage"},
                "commands": [["package"]],
            })
        );
        assert_eq!(
            evidence.archives().collect::<Vec<_>>(),
            [("ic-memory", "memory"), ("ic-storage", "storage"),]
        );
        let mut unknown = serde_json::to_value(&evidence).unwrap();
        unknown["unknown"] = serde_json::json!(true);
        assert!(serde_json::from_value::<PackageEvidence>(unknown).is_err());
        let mut incomplete = serde_json::to_value(&evidence).unwrap();
        incomplete.as_object_mut().unwrap().remove("archives");
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
