use super::*;

#[test]
fn observation_timeout_preserves_partial_output_and_typed_failure() {
    let mut command = Command::new("sh");
    command.args(["-c", "printf ready; printf detail >&2; exec sleep 30"]);
    let error = capture_complete(
        "sh",
        &mut command,
        OutputLimits {
            timeout: Some(Duration::from_secs(1)),
            ..observation_limits(1024, 1024)
        },
    )
    .unwrap_err();
    let ToolError::Execution(failure) = error.downcast_ref::<ToolError>().unwrap() else {
        panic!("expected retained execution evidence");
    };
    assert!(matches!(
        failure.failure,
        ic_host_process::tool::ExecutionFailure::TimedOut
    ));
    assert_eq!(failure.evidence.stdout, b"ready");
    assert_eq!(failure.evidence.stderr, b"detail");
}
