//! Dependency-free interoperability fixture; product adapters deserialize the JSON.
use std::{
    env,
    io::{self, Write},
    process::{Command, ExitCode},
};

fn main() -> ExitCode {
    let output = Command::new("perl")
        .args(env::args_os().skip(1))
        .output()
        .expect("start shared evidence verifier");
    io::stderr()
        .write_all(&output.stderr)
        .expect("write diagnostics");
    if !output.status.success() {
        assert!(
            output.stdout.is_empty(),
            "failure must not return a partial proof"
        );
        return ExitCode::FAILURE;
    }
    assert!(output.stdout.starts_with(b"{"));
    assert!(output.stdout.ends_with(b"}\n"));
    io::stdout()
        .write_all(&output.stdout)
        .expect("write canonical result");
    ExitCode::SUCCESS
}
