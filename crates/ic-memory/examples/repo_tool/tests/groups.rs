//! Actual consumer release phases with substituted Git, gates and uploads.
use super::*;

#[test]
fn every_group_preserves_unselected_metadata_and_recovers_preparation_and_upload() {
    for name in ["persistence", "metrics-stable", "logging-stable"] {
        let fixture = Fixture::for_group(name);
        let group = fixture.repo.release_group(None).unwrap();
        let source = fixture.repo.exec.state.borrow().git.source.clone();
        let failure_path = package_manifest(&group.primary);
        fixture.fail_files(FileFault::WriteAfter(failure_path.into()));
        assert!(fixture.prepare().is_err());
        for (path, original) in &source {
            assert_eq!(
                fs::read_to_string(fixture.repo.root.join(path)).unwrap(),
                *original,
                "{name}: rollback {path}"
            );
        }
        fixture.prepare().unwrap();
        let files = fixture.repo.release_files(None).unwrap();
        let changed = fixture
            .repo
            .exec
            .state
            .borrow()
            .git
            .source
            .iter()
            .filter_map(|(path, original)| {
                (fs::read_to_string(fixture.repo.root.join(path)).unwrap() != *original)
                    .then_some(path.clone())
            })
            .collect::<Vec<_>>();
        assert!(changed.iter().all(|path| files.contains(path)));
        assert_eq!(fixture.repo.version(None).unwrap(), "0.13.0");
        for package in packages() {
            assert_eq!(
                BuildEffects::package_version(&fixture.repo.root, package).unwrap(),
                if group.members.iter().any(|p| p == package) {
                    "0.13.0"
                } else {
                    "0.12.3"
                }
            );
            let owner = fixture
                .repo
                .release_groups(None)
                .unwrap()
                .into_iter()
                .find(|g| g.members.iter().any(|p| p == package))
                .unwrap();
            if owner.name != name {
                assert_eq!(
                    fs::read_to_string(fixture.repo.root.join(&owner.changelog)).unwrap(),
                    source[&owner.changelog]
                );
                assert_eq!(
                    fs::read_to_string(fixture.repo.root.join(package_manifest(package))).unwrap(),
                    source[&package_manifest(package)]
                );
            }
        }
        if name != "persistence" {
            assert_eq!(fixture.repo.text("Cargo.toml", None).unwrap(), MANIFEST);
        }
        let receipt = fixture.package_evidence(Prepared);
        assert_eq!(receipt.validation.selection.group, name);
        assert_eq!(receipt.archives.len(), group.members.len());
        assert!(
            group
                .members
                .iter()
                .all(|p| receipt.archives.contains_key(p))
        );
        assert!(
            fixture
                .repo
                .receipt_path("0.13.0", Prepared)
                .unwrap()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with(name)
        );
        fixture.stage().unwrap();
        fixture.commit().unwrap();
        fixture.fail_registry(RegistryFault::UploadResponseLost);
        assert!(fixture.repo.publish(false).is_err());
        fixture.repo.exec.state.borrow_mut().registry.fail = None;
        fixture.repo.publish(false).unwrap();
        fixture.repo.publish(false).unwrap();
        assert_eq!(
            fixture.repo.exec.state.borrow().registry.published.len(),
            group.members.len()
        );
    }
}

#[test]
fn compatible_core_release_preserves_adapter_requirements_and_identity() {
    let fixture = Fixture::new();
    let result = fixture
        .repo
        .project_catalog(MANIFEST, "0.12.3", "0.12.4")
        .unwrap();
    let after: toml::Value = toml::from_str(&result).unwrap();
    assert!(after["workspace"]["package"].get("version").is_none());
    assert_eq!(
        after["workspace"]["dependencies"]["ic-memory"]["version"].as_str(),
        Some("0.12.3")
    );
    let breaking = fixture
        .repo
        .project_catalog(MANIFEST, "0.12.3", "0.13.0")
        .unwrap();
    let after: toml::Value = toml::from_str(&breaking).unwrap();
    assert_eq!(
        after["workspace"]["dependencies"]["ic-memory"]["version"].as_str(),
        Some("0.13.0")
    );
    for group in ["metrics-stable", "logging-stable"] {
        let adapter = Fixture::for_group(group);
        assert_eq!(
            adapter
                .repo
                .project_catalog(MANIFEST, "0.12.3", "0.13.0")
                .unwrap(),
            MANIFEST
        );
    }
}

#[test]
fn group_receipts_cannot_be_replayed_under_another_selection() {
    let mut fixture = Fixture::new();
    fixture.prepare().unwrap();
    let receipt = fixture.repo.receipt_path("0.13.0", Prepared).unwrap();
    fixture.repo.group = "metrics-stable".to_owned();
    let foreign = fixture.repo.receipt_path("0.13.0", Prepared).unwrap();
    fs::write(&foreign, fs::read(&receipt).unwrap()).unwrap();
    fixture.selection.group = "metrics-stable".to_owned();
    assert!(fixture.repo.prepared(&fixture.selection).is_err());
    assert_eq!(fs::read(&receipt).unwrap(), fs::read(&foreign).unwrap());
}
