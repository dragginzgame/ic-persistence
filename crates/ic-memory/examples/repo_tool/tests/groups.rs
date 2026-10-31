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
        let catalog: toml::Value =
            toml::from_str(&fixture.repo.text("Cargo.toml", None).unwrap()).unwrap();
        for package in packages() {
            let expected = if group.members.iter().any(|p| p == package) {
                "0.13.0"
            } else {
                "0.12.3"
            };
            assert_eq!(
                BuildEffects::package_version(&fixture.repo.root, package).unwrap(),
                expected
            );
            assert_eq!(
                catalog["workspace"]["dependencies"][package]["version"].as_str(),
                Some(expected)
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
fn core_releases_align_local_requirements_and_preserve_adapter_identity() {
    let fixture = Fixture::new();
    let result = fixture
        .repo
        .project_catalog(MANIFEST, "0.12.3", "0.12.4")
        .unwrap();
    let after: toml::Value = toml::from_str(&result).unwrap();
    assert!(after["workspace"]["package"].get("version").is_none());
    assert_eq!(
        after["workspace"]["dependencies"]["ic-memory"]["version"].as_str(),
        Some("0.12.4")
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
    for prefix in ["", "^", "~", "=", "^  "] {
        let source = MANIFEST.replace(
            "version = \"0.12.3\"",
            &format!("version = \"{prefix}0.12.3\""),
        );
        for version in ["0.12.4", "0.13.0"] {
            let result = fixture
                .repo
                .project_catalog(&source, "0.12.3", version)
                .unwrap();
            let result: toml::Value = toml::from_str(&result).unwrap();
            for name in ["ic-memory", "ic-storage"] {
                assert_eq!(
                    result["workspace"]["dependencies"][name]["version"].as_str(),
                    Some(format!("{prefix}{version}").as_str())
                );
            }
            assert_eq!(
                result["workspace"]["dependencies"]["other"].as_str(),
                Some("0.12.3")
            );
        }
    }
    for stale in ["0.12", "0.12.2"] {
        let source = MANIFEST.replace("version = \"0.12.3\"", &format!("version = \"{stale}\""));
        assert!(
            fixture
                .repo
                .project_catalog(&source, "0.12.3", "0.12.4")
                .is_err()
        );
    }
    for group in ["metrics-stable", "logging-stable"] {
        let adapter = Fixture::for_group(group);
        let projected = adapter
            .repo
            .project_catalog(MANIFEST, "0.12.3", "0.13.0")
            .unwrap();
        let projected: toml::Value = toml::from_str(&projected).unwrap();
        for name in [
            "ic-memory",
            "ic-storage",
            "ic-metrics-stable",
            "ic-logging-stable",
        ] {
            let expected = if name == format!("ic-{group}") {
                "0.13.0"
            } else {
                "0.12.3"
            };
            assert_eq!(
                projected["workspace"]["dependencies"][name]["version"].as_str(),
                Some(expected)
            );
        }
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
