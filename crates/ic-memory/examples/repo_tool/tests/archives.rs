//! Archives release contracts.

use super::*;
use crate::repo_tool::wasm::{PROBES, check_wasm_artifacts};

#[test]
fn release_version_requires_explicit_group_member_identity() {
    let fixture = Fixture::new();
    assert_eq!(fixture.repo.version(None).unwrap(), "0.12.3");
    for name in packages() {
        for manifest in [
            String::new(),
            "[workspace]\n".to_owned(),
            format!("[package]\nname = {name:?}\n"),
            "[package]\nname = \"another-package\"\nversion.workspace = true\n".to_owned(),
            format!("[package]\nname = {name:?}\nversion.workspace = true\n"),
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
        fixture.repo.prepared(&fixture.selection).unwrap().archives["ic-memory"],
        digest
    );
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
fn storage_archive_and_manifest_are_bound_to_the_coordinated_release() {
    let (package, path) = ("ic-storage", package_manifest("ic-storage"));
    let fixture = Fixture::new();
    fixture.release();
    assert_eq!(
        toml::from_str::<toml::Value>(&fs::read_to_string(fixture.repo.root.join(path)).unwrap())
            .unwrap()["package"]["version"]
            .as_str(),
        Some("0.13.0")
    );
    let manifest: toml::Value =
        toml::from_str(&fs::read_to_string(fixture.repo.root.join("Cargo.toml")).unwrap()).unwrap();
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

#[test]
fn coordinated_archive_receipt_uses_one_admitted_target_observation() {
    let fixture = Fixture::new();
    fixture.release();
    let previous = fixture
        .repo
        .verify_evidence(&fixture.selection, "release", Final, Working)
        .unwrap();
    let expected = serde_json::to_value(&previous).unwrap();
    fixture.repo.exec.state.borrow_mut().calls.clear();
    let recorded = PackageEvidence::record(
        &fixture.repo,
        previous.validation,
        previous.package_head,
        previous.lock_sha256,
        previous.commands,
    )
    .unwrap();
    assert_eq!(serde_json::to_value(recorded).unwrap(), expected);
    assert_eq!(
        fixture
            .repo
            .exec
            .state
            .borrow()
            .calls
            .iter()
            .filter(|call| call
                .first()
                .is_some_and(|program| program == "cargo-metadata"))
            .count(),
        1
    );
}
