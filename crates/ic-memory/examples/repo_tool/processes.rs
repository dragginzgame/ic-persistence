//! Native process and durable-file boundaries for release orchestration.

use super::{REPLACE_FILE_OPTIONS, Result, require};
use ic_build::cargo::{
    observation::{CargoMetadataSpec, decode_metadata},
    selection::CargoFeatureSelection,
};
use ic_host_artifacts::artifact::Sha256Digest;
use ic_host_process::tool::{
    CompletedExecution, OutputLimit, OutputLimits, ToolError, capture_command,
};
use serde::Serialize;
use std::{
    io::{BufWriter, Write as _},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::Instant,
};

///
/// Execute
///
/// Process and file-publication boundaries for real effects and test substitutes.
///

pub(super) trait Execute {
    fn execute(&self, root: &Path, program: &str, args: &[&str]) -> Result<()>;

    fn capture(&self, root: &Path, program: &str, args: &[&str]) -> Result<String>;

    fn cargo_target(&self, root: &Path) -> Result<PathBuf>;

    fn write_bytes(&self, path: &Path, bytes: &[u8]) -> Result<()> {
        Ok(ic_host_fs::durable::write_bytes(
            path,
            REPLACE_FILE_OPTIONS,
            bytes,
        )?)
    }

    fn write_json(&self, path: &Path, value: &impl Serialize) -> Result<()> {
        // Keep receipt encoding local; the shared engine owns staging and
        // publication, including original producer and cleanup failures.
        Ok(ic_host_fs::durable::write_with(
            path,
            REPLACE_FILE_OPTIONS,
            |file| {
                let mut writer = BufWriter::new(file);
                serde_json::to_writer_pretty(&mut writer, value)?;
                // Flush the fixed-size buffer before the engine may publish.
                writer.flush().map_err(serde_json::Error::io)
            },
        )?)
    }

    fn copy_verified(&self, source: &Path, path: &Path, expected: Sha256Digest) -> Result<()> {
        ic_host_fs::durable::copy_verified(source, path, expected, u64::MAX, REPLACE_FILE_OPTIONS)?;
        Ok(())
    }
}

///
/// Processes
///
/// Native process execution, preserving argument boundaries and failure status.
///

pub(super) struct Processes;

impl Execute for Processes {
    fn cargo_target(&self, root: &Path) -> Result<PathBuf> {
        let mut command = CargoMetadataSpec {
            manifest_path: &root.join("Cargo.toml"),
            include_dependencies: false,
            filter_platform: None,
            locked: true,
            offline: true,
            features: &CargoFeatureSelection::default(),
        }
        .command(Command::new("cargo"))?;
        let output = capture_complete(
            "cargo",
            &mut command,
            OutputLimits {
                stdout: OutputLimit::Terminate(4 * 1024 * 1024),
                stderr: OutputLimit::Terminate(64 * 1024),
                timeout: None,
            },
        )?;
        let metadata = decode_metadata(&output.stdout)?;
        Ok(metadata.target_directory.into_std_path_buf())
    }

    fn capture(&self, root: &Path, program: &str, args: &[&str]) -> Result<String> {
        let mut command = Command::new(program);
        command.current_dir(root).args(args);
        // Keep successful output complete, but reject oversized responses.
        // Host owns fair pipe draining, failure evidence and child cleanup.
        let output = capture_complete(
            program,
            &mut command,
            OutputLimits {
                stdout: OutputLimit::Terminate(8 * 1024 * 1024),
                stderr: OutputLimit::Terminate(256 * 1024),
                timeout: None,
            },
        )?;
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
        Ok(String::from_utf8(output.stdout)?)
    }

    fn execute(&self, root: &Path, program: &str, args: &[&str]) -> Result<()> {
        let mut command = Command::new(program);
        command.current_dir(root).args(args);
        let phase = args
            .iter()
            .find(|arg| !arg.starts_with('-') && !arg.starts_with('+'))
            .copied()
            .unwrap_or("");
        let started = Instant::now();
        eprintln!("repo-tool: starting {program} {phase}");
        let status = command
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status();
        eprintln!(
            "repo-tool: {program} {phase} finished in {:.3}s ({})",
            started.elapsed().as_secs_f64(),
            match &status {
                Ok(status) => status.to_string(),
                Err(error) => format!("spawn/wait failed: {error}"),
            }
        );
        let status = status?;
        require(
            status.success(),
            &format!("{program} {args:?} failed ({status})"),
        )?;
        Ok(())
    }
}

fn capture_complete(
    program: &str,
    command: &mut Command,
    limits: OutputLimits,
) -> Result<CompletedExecution> {
    Ok(capture_command(command, limits)
        .and_then(CompletedExecution::require_success)
        .and_then(CompletedExecution::require_complete)
        .inspect_err(|error| report_tool_error(program, error))?)
}

fn report_tool_error(program: &str, error: &ToolError) {
    // Host formatting excludes bytes; preserve stderr before returning its cause.
    if let ToolError::Execution(failure) = error {
        if let Some(status) = failure.evidence.status {
            eprintln!("{program} failed ({status})");
        }
        eprint!("{}", String::from_utf8_lossy(&failure.evidence.stderr));
    }
}
