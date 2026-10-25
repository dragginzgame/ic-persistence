use super::*;

#[test]
fn publication_reconciles_a_lost_upload_response_without_reuploading_memory() {
    let fixture = Fixture::new();
    fixture.release();
    fixture.fail(Fault::UploadResponseLost);
    assert!(fixture.repo.publish(false).is_err());
    fixture.repo.exec.state.borrow_mut().fail = None;
    fixture.repo.publish(false).unwrap();
    assert_eq!(fixture.repo.exec.state.borrow().published.len(), 3);
    assert_eq!(
        fixture
            .repo
            .exec
            .state
            .borrow()
            .calls
            .iter()
            .filter(|call| call.get(2).is_some_and(|arg| arg == "publish"))
            .count(),
        3
    );
}

#[test]
fn registry_conflicts_or_unknown_status_refuse_before_any_upload() {
    for package in ["ic-storage", "ic-metrics-stable"] {
        for failure in [
            Fault::RegistryForbidden,
            Fault::RegistryError,
            Fault::RegistryYanked,
            Fault::Checksum,
        ] {
            let fixture = Fixture::new();
            fixture.release();
            let evidence = fixture
                .repo
                .verify_evidence(&fixture.selection, "release", Final, Working)
                .unwrap();
            fixture.repo.exec.state.borrow_mut().published.insert(
                package.to_owned(),
                if failure == Fault::Checksum {
                    "wrong".to_owned()
                } else if package == "ic-storage" {
                    evidence.storage_sha256
                } else {
                    evidence.metrics_sha256
                },
            );
            fixture.fail(failure.clone());
            let error = fixture.repo.publish(false).unwrap_err().to_string();
            if let Some(status) = match failure {
                Fault::RegistryForbidden => Some("403"),
                Fault::RegistryError => Some("503"),
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
                    .calls
                    .iter()
                    .any(|call| call.get(2).is_some_and(|arg| arg == "publish"))
            );
        }
    }
}
