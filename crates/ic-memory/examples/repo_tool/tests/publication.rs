use super::*;

#[test]
fn publication_reconciles_a_lost_upload_response_without_reuploading_memory() {
    let fixture = Fixture::new();
    fixture.release();
    fixture.fail("upload-response-lost");
    assert!(fixture.repo.publish(false).is_err());
    fixture.repo.exec.state.borrow_mut().fail = None;
    fixture.repo.publish(false).unwrap();
    assert_eq!(fixture.repo.exec.state.borrow().published.len(), 2);
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
        2
    );
}

#[test]
fn registry_conflicts_or_unknown_status_refuse_before_any_upload() {
    for failure in [
        "registry-forbidden",
        "registry-error",
        "registry-yanked",
        "checksum",
    ] {
        let fixture = Fixture::new();
        fixture.release();
        let evidence = fixture
            .repo
            .verify_evidence(&fixture.selection, "release", false, true)
            .unwrap();
        fixture.repo.exec.state.borrow_mut().published.insert(
            "ic-storage".to_owned(),
            if failure == "checksum" {
                "wrong".to_owned()
            } else {
                evidence.storage_sha256
            },
        );
        fixture.fail(failure);
        let error = fixture.repo.publish(false).unwrap_err().to_string();
        if let Some(status) = match failure {
            "registry-forbidden" => Some("403"),
            "registry-error" => Some("503"),
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
