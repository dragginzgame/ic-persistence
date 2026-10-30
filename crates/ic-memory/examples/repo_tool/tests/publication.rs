//! Publication release contracts.

use super::*;

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
        assert!(!fixture.repo.exec.state.borrow().has_call(|call| {
            call.first().is_some_and(|arg| arg == "cargo") && is_cargo_call(call, "publish")
        }));
    }
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
                .has_call(|call| is_cargo_call(call, "publish"))
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
fn logging_upload_rejection_and_lost_response_resume_without_replaying_dependencies() {
    for (fault, count_after_failure, logging_uploads) in [
        (RegistryFault::LoggingUpload, 3, 2),
        (RegistryFault::LoggingUploadResponseLost, 4, 1),
    ] {
        let fixture = Fixture::new();
        fixture.release();
        fixture.fail_registry(fault);
        assert!(fixture.repo.publish(false).is_err());
        assert_eq!(
            fixture.repo.exec.state.borrow().registry.published.len(),
            count_after_failure
        );
        fixture.repo.exec.state.borrow_mut().clear_faults();
        fixture.repo.publish(false).unwrap();
        fixture.repo.publish(false).unwrap();
        let state = fixture.repo.exec.state.borrow();
        assert_eq!(state.registry.published.len(), packages().len());
        for name in packages() {
            let uploads = state
                .calls
                .iter()
                .filter(|call| {
                    is_cargo_call(call, "publish")
                        && call.windows(2).any(|pair| pair == ["--package", name])
                })
                .count();
            assert_eq!(
                uploads,
                if name == "ic-logging-stable" {
                    logging_uploads
                } else {
                    1
                }
            );
        }
    }
}

#[test]
fn missing_or_changed_logging_archive_refuses_before_any_registry_observation() {
    for missing in [true, false] {
        let fixture = Fixture::new();
        fixture.release();
        let archive = fixture
            .repo
            .package_path("ic-logging-stable", &fixture.selection.version)
            .unwrap();
        if missing {
            fs::remove_file(archive).unwrap();
        } else {
            fs::write(archive, b"foreign logging archive").unwrap();
        }
        let before = fixture.repo.exec.state.borrow().calls.len();
        assert!(fixture.repo.publish(false).is_err());
        let state = fixture.repo.exec.state.borrow();
        assert!(
            state.calls[before..]
                .iter()
                .all(|call| call[0] != "curl" && !is_cargo_call(call, "publish"))
        );
        assert!(state.registry.published.is_empty());
    }
}

#[test]
fn publication_reconciles_a_lost_upload_response_without_reuploading_memory() {
    let fixture = Fixture::new();
    fixture.release();
    fixture.fail_registry(RegistryFault::UploadResponseLost);
    assert!(fixture.repo.publish(false).is_err());
    fixture.repo.exec.state.borrow_mut().clear_faults();
    fixture.repo.publish(false).unwrap();
    assert_eq!(
        fixture.repo.exec.state.borrow().registry.published.len(),
        packages().len()
    );
    assert_eq!(
        fixture
            .repo
            .exec
            .state
            .borrow()
            .calls
            .iter()
            .filter(|call| is_cargo_call(call, "publish"))
            .count(),
        packages().len()
    );
}

#[test]
fn registry_conflicts_or_unknown_status_refuse_before_any_upload() {
    for package in packages().into_iter().skip(1) {
        for failure in [
            RegistryFault::RegistryForbidden,
            RegistryFault::RegistryError,
            RegistryFault::RegistryYanked,
            RegistryFault::Checksum,
        ] {
            let fixture = Fixture::new();
            fixture.release();
            let evidence = fixture
                .repo
                .verify_evidence(&fixture.selection, "release", Final, Working)
                .unwrap();
            fixture
                .repo
                .exec
                .state
                .borrow_mut()
                .registry
                .published
                .insert(
                    package.to_owned(),
                    if failure == RegistryFault::Checksum {
                        "wrong".to_owned()
                    } else {
                        evidence
                            .archives()
                            .find(|(name, _)| *name == package)
                            .unwrap()
                            .1
                            .to_owned()
                    },
                );
            fixture.fail_registry(failure.clone());
            let error = fixture.repo.publish(false).unwrap_err().to_string();
            if let Some(status) = match failure {
                RegistryFault::RegistryForbidden => Some("403"),
                RegistryFault::RegistryError => Some("503"),
                _ => None,
            } {
                assert!(error.contains(&format!("HTTP {status}")));
                assert!(error.contains(&format!("ic-memory {}", fixture.selection.version)));
            }
            assert!(
                !fixture
                    .repo
                    .exec
                    .state
                    .borrow()
                    .has_call(|call| is_cargo_call(call, "publish"))
            );
        }
    }
}
