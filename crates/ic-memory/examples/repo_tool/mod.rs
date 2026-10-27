//! Maintainer release workflow and portable artifact qualification.
//!
//! Source identity, selected dependencies, compiler identities and the verified
//! package travel together in one receipt. Publication checks that evidence
//! before dispatch; it never resolves a replacement lockfile.

use clap::{Parser, Subcommand};
use ic_build::cargo::{
    environment::{
        ConfigurationLimits, configuration_directories, is_build_variable, is_compiler_override,
        observe_configurations,
    },
    lock_projection::{WorkspaceVersionProjection, verify_workspace_release_projection},
    observation::{CargoMetadataSpec, decode_metadata},
    release_metadata::{
        DependencyVersionProjection, render_workspace_lock, render_workspace_manifest,
    },
    selection::CargoFeatureSelection,
};
use ic_host_artifacts::artifact::Sha256Digest;
use ic_host_fs::durable::{PublicationMode, WriteOptions};
use ic_host_fs::read::hash_file;
use ic_host_process::tool::{
    CompletedExecution, OutputLimit, OutputLimits, ToolError, capture_command,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    env,
    error::Error,
    fs,
    io::{BufWriter, Write as _},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Instant, SystemTime, UNIX_EPOCH},
};

mod metadata;
mod packages;
use packages::{package_manifest, packages};
mod publication;
mod qualification;
use ArchiveLocation::{Retained, Working};
use ReceiptStage::{Final, Prepared};

#[cfg(test)]
mod tests;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

// Consumer admission budgets for complete Cargo configuration observation.
const CONFIGURATION_LIMITS: ConfigurationLimits = ConfigurationLimits {
    file_bytes: 1024 * 1024,
    total_bytes: 4 * 1024 * 1024,
    files: 64,
    depth: 16,
};

///
/// Arguments
///
/// Parse the maintainer command before observing repository state or effects.
///

#[derive(Debug, Parser)]
#[command(
    name = "repo-tool",
    about = "IC Persistence release and qualification tooling"
)]
struct Arguments {
    #[command(subcommand)]
    command: ToolCommand,
}

///
/// ToolCommand
///
/// Maintainer operations with command-specific arguments.
///

#[derive(Debug, Subcommand)]
enum ToolCommand {
    /// Print the selected package version.
    Version,
    /// Print the package's minimum supported Rust version.
    Msrv,
    /// Print the selected development toolchain.
    Toolchain,
    /// Print Cargo's target directory.
    Target,
    /// Require clean repository source.
    EnsureClean,
    /// Build the selected Wasm probes offline and check their size budgets.
    WasmSize,
    /// Admit source and saved release intent before validation.
    ReleasePreflight,
    /// Validate and retain release evidence.
    ReleaseVerify,
    /// Prepare the saved release's metadata transaction.
    ReleasePrepareVersion,
    /// Check prepared metadata against saved intent.
    ReleasePreparedCheck,
    /// Print selected release paths as NUL-delimited records.
    ReleaseFiles,
    /// Admit the selected index before committing.
    ReleaseCommitCheck,
    /// Verify the selected committed release.
    ReleaseCommittedCheck,
    /// Verify the selected release tag.
    ReleaseTaggedCheck,
    /// Admit the selected release before pushing.
    ReleasePushCheck,
    /// Qualify the release's retained source and package.
    QualifyRelease,
    /// Publish the qualified package using saved release evidence.
    Publish {
        /// Check publication without uploading the package.
        #[arg(long)]
        dry_run: bool,
    },
}

///
/// PreparationError
///
/// Original preparation failure together with failed metadata restorations.
///
/// Successful restorations and retained evidence remain available for recovery.
///

#[derive(Debug)]
struct PreparationError {
    source: Box<dyn Error>,
    rollback_errors: Vec<(String, Box<dyn Error>)>,
}

impl std::fmt::Display for PreparationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.source)?;
        for (path, error) in &self.rollback_errors {
            write!(formatter, "; could not restore {path}: {error}")?;
        }
        Ok(())
    }
}

impl Error for PreparationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.source.as_ref())
    }
}

const PACKAGE_README: &str = "crates/ic-memory/README.md";
const CHANGELOG_FINALIZER: &str = "scripts/ci/finalize-release-changelog.awk";
const RELEASE_INPUT_ADMISSION: &str = "scripts/ci/validate-release-inputs.pl";
const TAG_CHECKER: &str = "scripts/ci/check-release-tag.sh";
const REPLACE_FILE_OPTIONS: WriteOptions = WriteOptions {
    mode: PublicationMode::Replace,
    permissions: 0o666,
};
const PROBES: [(&str, u64); 4] = [
    ("core", 260_000),
    ("diagnostics", 315_000),
    ("admission", 264_000),
    ("runtime_integration", 270_000),
];

///
/// ReleaseSelection
///
/// Exact release intent supplied by the common runner.
///
/// Adapters consume this selection without choosing another increment or target.
///

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct ReleaseSelection {
    kind: String,
    previous: String,
    source: String,
    version: String,
    date: String,
    remote: String,
    branch: String,
}

/// One inspected commit and its source-bound release intent.
#[derive(Debug, Eq, PartialEq)]
struct InspectedRelease {
    commit: String,
    selection: ReleaseSelection,
}

/// Immutable source and candidate bytes for one release-owned path.
struct ReleaseSurface {
    original: String,
    expected: String,
}

///
/// ValidationEvidence
///
/// Successful full-gate inputs, saved before release metadata may change.
///
/// The original selected lock bytes permit exact root-version refresh checks
/// and recovery after an interrupted refresh without selecting dependencies.
///

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ValidationEvidence {
    selection: ReleaseSelection,
    lock: Vec<u8>,
    identities: [String; 3],
    configuration: BTreeMap<String, String>,
    command: Vec<String>,
}

///
/// PackageEvidence
///
/// Qualified package bound to the saved validation and current root lockfile.
///

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct PackageEvidence {
    validation: ValidationEvidence,
    package_head: String,
    lock_sha256: String,
    package_sha256: String,
    storage_sha256: String,
    metrics_sha256: String,
    commands: Vec<Vec<String>>,
}

// These names describe existing receipt/observation choices without changing JSON.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ReceiptStage {
    Prepared,
    Final,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ArchiveLocation {
    Working,
    Retained,
}

impl PackageEvidence {
    // Keep the fixed receipt-field mapping beside its read projection. Callers
    // carry complete evidence rather than positional arrays of package digests.
    fn record<E: Execute>(
        repo: &Repository<E>,
        validation: ValidationEvidence,
        package_head: String,
        lock_sha256: String,
        commands: Vec<Vec<String>>,
    ) -> Result<Self> {
        let [memory, storage, metrics] = packages();
        let version = &validation.selection.version;
        let package_sha256 = repo.record_package(memory, version)?;
        let storage_sha256 = repo.record_package(storage, version)?;
        let metrics_sha256 = repo.record_package(metrics, version)?;
        Ok(Self {
            validation,
            package_head,
            lock_sha256,
            package_sha256,
            storage_sha256,
            metrics_sha256,
            commands,
        })
    }

    fn archives(&self) -> impl Iterator<Item = (&'static str, &str)> {
        packages().into_iter().zip([
            self.package_sha256.as_str(),
            self.storage_sha256.as_str(),
            self.metrics_sha256.as_str(),
        ])
    }
}

///
/// Repository
///
/// Explicit working directory for external commands and release files.
///
/// Command execution is owned here so tests can substitute external effects
/// without creating Git commits, tags, pushes or package publications.
///

struct Repository<E> {
    root: PathBuf,
    exec: E,
}

///
/// Execute
///
/// Process and file-publication boundaries for real effects and test substitutes.
///

trait Execute {
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

struct Processes;

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

fn require(condition: bool, message: &str) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(message.into())
    }
}

impl ReleaseSelection {
    fn from_env<E: Execute>(repo: &Repository<E>) -> Result<Self> {
        let selection = Self {
            kind: env::var("RELEASE_KIND")?,
            previous: env::var("RELEASE_PREVIOUS")?,
            version: env::var("RELEASE_VERSION")?,
            date: env::var("RELEASE_DATE")?,
            source: env::var("RELEASE_SOURCE")?,
            remote: env::var("RELEASE_REMOTE")?,
            branch: env::var("RELEASE_BRANCH")?,
        };
        require(
            matches!(selection.kind.as_str(), "patch" | "minor" | "major"),
            "invalid release kind",
        )?;
        repo.version_parts(&selection.previous)?;
        repo.version_parts(&selection.version)?;
        repo.validate_date(&selection.date)?;
        require(
            valid_commit_identity(&selection.source),
            "invalid source commit identity",
        )?;
        require(
            !selection.remote.is_empty()
                && selection
                    .remote
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || b"._-".contains(&byte)),
            "invalid selected remote",
        )?;
        require(
            !selection.branch.is_empty()
                && !selection.branch.bytes().any(|byte| byte.is_ascii_control()),
            "invalid selected branch",
        )?;
        Ok(selection)
    }
}

fn valid_commit_identity(commit: &str) -> bool {
    matches!(commit.len(), 40 | 64)
        && commit
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn release_commit_from_env() -> Result<String> {
    let commit = env::var("RELEASE_COMMIT")?;
    require(
        valid_commit_identity(&commit),
        "invalid selected release commit",
    )?;
    Ok(commit)
}

fn toml_string(text: &str, section: &str, field: &str) -> Result<String> {
    let value: toml::Value = toml::from_str(text)?;
    section
        .split('.')
        .try_fold(&value, |value, key| value.get(key))
        .and_then(|value| value.get(field))
        .and_then(toml::Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| format!("missing {section}.{field}").into())
}

fn check_lock_update(before: &[u8], after: &str, previous: &str, version: &str) -> Result<()> {
    let before = std::str::from_utf8(before)?;
    verify_workspace_release_projection(
        before,
        after,
        &packages().map(|package| WorkspaceVersionProjection {
            package,
            previous,
            next: version,
        }),
    )?;
    Ok(())
}

fn check_wasm_artifacts(target: &Path) -> Result<()> {
    let artifacts = target.join("wasm32-unknown-unknown/wasm-size/examples");
    for (probe, budget) in PROBES {
        let path = artifacts.join(format!("wasm_{probe}_size_probe.wasm"));
        let metadata = fs::metadata(&path)?;
        require(metadata.is_file(), "Wasm artifact is not a regular file")?;
        let bytes = metadata.len();
        println!("{probe} raw Wasm: {bytes} bytes (budget: {budget} bytes)");
        require(
            bytes <= budget,
            &format!("{} exceeds its raw Wasm budget", path.display()),
        )?;
    }
    Ok(())
}

impl<E: Execute> Repository<E> {
    fn wasm_size(&self) -> Result<()> {
        let mut command = vec![
            "cargo".to_owned(),
            format!("+{}", self.toolchain()?),
            "build".to_owned(),
            "--locked".to_owned(),
            "--offline".to_owned(),
            "--profile".to_owned(),
            "wasm-size".to_owned(),
            "--target".to_owned(),
            "wasm32-unknown-unknown".to_owned(),
        ];
        for (probe, _) in PROBES {
            command.extend([
                "--example".to_owned(),
                format!("wasm-{}-size-probe", probe.replace('_', "-")),
            ]);
        }
        self.run_command(&command)?;
        check_wasm_artifacts(&self.target()?)
    }

    fn run(&self, program: &str, args: &[&str]) -> Result<()> {
        self.exec.execute(&self.root, program, args)
    }

    fn output(&self, program: &str, args: &[&str]) -> Result<String> {
        self.exec.capture(&self.root, program, args)
    }

    fn git(&self, args: &[&str]) -> Result<String> {
        Ok(self.output("git", args)?.trim().to_owned())
    }

    fn text(&self, path: &str, revision: Option<&str>) -> Result<String> {
        match revision {
            Some(revision) => self.output("git", &["show", &format!("{revision}:{path}")]),
            None => Ok(fs::read_to_string(self.root.join(path))?),
        }
    }

    fn version(&self, revision: Option<&str>) -> Result<String> {
        let text = self.text("Cargo.toml", revision)?;
        self.member_release_manifest(revision, "ic-memory")?;
        let version = toml_string(&text, "workspace.package", "version")?;
        self.version_parts(&version)?;
        Ok(version)
    }

    fn toolchain(&self) -> Result<String> {
        toml_string(
            &self.text("rust-toolchain.toml", None)?,
            "toolchain",
            "channel",
        )
    }

    fn msrv(&self) -> Result<String> {
        toml_string(
            &self.text("Cargo.toml", None)?,
            "workspace.package",
            "rust-version",
        )
    }

    fn clean(&self) -> Result<()> {
        let status = self.output(
            "git",
            &[
                "--no-optional-locks",
                "status",
                "--porcelain",
                "--untracked-files=all",
            ],
        )?;
        require(
            status.trim().is_empty(),
            &format!(
                "working tree/index must be clean; review these Git status entries:\n{}",
                status.trim_end()
            ),
        )
    }

    fn target(&self) -> Result<PathBuf> {
        let target = self.exec.cargo_target(&self.root)?;
        require(
            target.is_absolute(),
            "Cargo target_directory must be absolute",
        )?;
        Ok(target)
    }

    fn receipt_path(&self, version: &str, stage: ReceiptStage) -> Result<PathBuf> {
        self.version_parts(version)?;
        let suffix = if stage == Prepared { "-prepared" } else { "" };
        Ok(self
            .target()?
            .join("release-validation")
            .join(format!("{version}{suffix}.json")))
    }

    fn package_path(&self, package: &str, version: &str) -> Result<PathBuf> {
        self.version_parts(version)?;
        Ok(self
            .target()?
            .join("package")
            .join(format!("{package}-{version}.crate")))
    }

    fn sha256(path: &Path) -> Result<String> {
        // Preserve the existing file-size contract; hashing uses constant memory.
        Ok(hash_file(path, u64::MAX)?.sha256.to_string())
    }

    fn identities(&self) -> Result<[String; 3]> {
        Ok([
            self.output("cargo", &[&format!("+{}", self.toolchain()?), "--version"])?,
            self.output("rustc", &[&format!("+{}", self.toolchain()?), "-Vv"])?,
            self.output("rustc", &[&format!("+{}", self.msrv()?), "-Vv"])?,
        ])
    }

    fn retained_package(&self, digest: &str) -> Result<PathBuf> {
        let digest: Sha256Digest = digest.parse()?;
        Ok(self
            .target()?
            .join("release-validation/artifacts")
            .join(format!("{digest}.crate")))
    }

    fn record_package(&self, name: &str, version: &str) -> Result<String> {
        let package = self.package_path(name, version)?;
        let digest = Self::sha256(&package)?;
        let retained = self.retained_package(&digest)?;
        if !retained.exists() {
            self.exec
                .copy_verified(&package, &retained, digest.parse()?)?;
        }
        require(
            Self::sha256(&retained)? == digest,
            "retained package differs from its digest",
        )?;
        Ok(digest)
    }

    fn configuration(&self) -> Result<BTreeMap<String, String>> {
        let mut configuration = BTreeMap::new();
        for (name, value) in env::vars_os() {
            let name = name.to_str().ok_or("non-UTF-8 environment variable name")?;
            // The receipts identify the declared toolchains. Cargo compiler
            // replacements or wrappers would execute outside those identities.
            require(
                !is_compiler_override(name),
                &format!("unset {name} before release qualification; use the declared toolchains"),
            )?;
            if is_build_variable(name) {
                configuration.insert(
                    format!("env:{name}"),
                    value
                        .into_string()
                        .map_err(|_| "non-UTF-8 build environment")?,
                );
            }
        }
        let cargo_home = env::var_os("CARGO_HOME");
        let home = env::var_os("HOME");
        require(
            cargo_home.is_some() || home.is_some(),
            "set CARGO_HOME or HOME",
        )?;
        // Keep conservative custody of both root names, even when Cargo shadows
        // one. Build owns discovery, include traversal and same-read raw hashes.
        let roots: Vec<_> =
            configuration_directories(&self.root, cargo_home.as_deref(), home.as_deref())?
                .iter()
                .flat_map(|directory| [directory.join("config"), directory.join("config.toml")])
                .filter(|path| path.exists())
                .collect();
        for file in observe_configurations(&roots, CONFIGURATION_LIMITS)? {
            if let Some(field) = file.compiler_overrides.first() {
                return Err(format!(
                    "remove build.{field} from {} before release qualification; use the declared toolchains",
                    file.path.display()
                ).into());
            }
            // Contents may contain secrets; receipts retain only lookup labels
            // and the hash of the exact bytes parsed by the observer.
            configuration.insert(
                format!("config:{}", file.path.display()),
                file.sha256.to_string(),
            );
        }
        Ok(configuration)
    }

    fn check_selection(&self, selection: &ReleaseSelection) -> Result<()> {
        self.version_parts(&selection.previous)?;
        self.version_parts(&selection.version)?;
        self.validate_date(&selection.date)?;
        require(
            self.output(
                "bash",
                &[
                    "scripts/ci/next-release-version.sh",
                    &selection.previous,
                    &selection.kind,
                ],
            )?
            .trim()
                == selection.version,
            "candidate differs from the common version selection",
        )?;
        require(
            self.version(Some(&selection.source))? == selection.previous,
            "saved source version differs from the release selection",
        )
    }

    fn member_release_manifest(&self, revision: Option<&str>, name: &str) -> Result<String> {
        let text = self.text(&package_manifest(name), revision)?;
        let manifest: toml::Value = toml::from_str(&text)?;
        let package = manifest
            .get("package")
            .ok_or("missing Persistence package")?;
        require(
            package.get("name").and_then(toml::Value::as_str) == Some(name),
            "unexpected Persistence package",
        )?;
        require(
            package
                .get("version")
                .and_then(|version| version.get("workspace"))
                .and_then(toml::Value::as_bool)
                == Some(true),
            "expected inherited Persistence version",
        )?;
        Ok(text)
    }

    fn surfaces(&self, selection: &ReleaseSelection) -> Result<BTreeMap<String, ReleaseSurface>> {
        self.check_selection(selection)?;
        let source = &selection.source;
        let previous = &selection.previous;
        let version = &selection.version;
        let [major, minor, _] = self.version_parts(version)?;
        let detail = format!("docs/changelog/{major}.{minor}.md");
        let finalizer = self.root.join(CHANGELOG_FINALIZER);
        let mut surfaces = BTreeMap::new();
        for path in [
            "Cargo.lock",
            "Cargo.toml",
            PACKAGE_README,
            "CHANGELOG.md",
            &detail,
        ] {
            let original = self.text(path, Some(source))?;
            let expected = match path {
                "Cargo.lock" => Self::replace_lock_version(&original, previous, version)?,
                "Cargo.toml" => self.replace_workspace_version(&original, previous, version)?,
                PACKAGE_README => original.clone(),
                _ => self.release_changelog(
                    &finalizer,
                    &original,
                    previous,
                    version,
                    &selection.date,
                )?,
            };
            surfaces.insert(path.to_owned(), ReleaseSurface { original, expected });
        }
        for name in packages().into_iter().skip(1) {
            let original = self.member_release_manifest(Some(source), name)?;
            surfaces.insert(
                package_manifest(name),
                ReleaseSurface {
                    expected: original.clone(),
                    original,
                },
            );
        }
        Ok(surfaces)
    }

    fn check_surfaces(
        &self,
        selection: &ReleaseSelection,
        surfaces: &BTreeMap<String, ReleaseSurface>,
        revision: Option<&str>,
        partial: bool,
    ) -> Result<()> {
        let mut expected_changes = Vec::new();
        for (path, surface) in surfaces {
            let actual = self.text(path, revision)?;
            require(
                actual == surface.expected || partial && actual == surface.original,
                &format!(
                    "{path} does not match the admitted release bytes for source {}. \
                     For a new release, commit intended source edits first; git add alone is insufficient. \
                     For recovery, preserve the saved plan and reconcile the conflicting file",
                    selection.source
                ),
            )?;
            if actual != surface.original {
                expected_changes.push(path.clone());
            }
        }
        let mut args = vec!["diff", "--name-only", &selection.source];
        if let Some(revision) = revision {
            args.push(revision);
        }
        args.push("--");
        require(
            self.output("git", &args)?
                .lines()
                .eq(expected_changes.iter().map(String::as_str)),
            "release candidate contains changes outside its version surfaces",
        )?;
        require(
            self.output("git", &["ls-files", "--others", "--exclude-standard"])?
                .is_empty(),
            "release candidate has untracked files",
        )
    }

    fn check_index(
        &self,
        selection: &ReleaseSelection,
        surfaces: &BTreeMap<String, ReleaseSurface>,
    ) -> Result<()> {
        for path in self
            .output(
                "git",
                &["diff", "--cached", "--name-only", &selection.source, "--"],
            )?
            .lines()
        {
            let expected = surfaces
                .get(path)
                .ok_or("index contains unrelated release changes")?;
            require(
                self.git(&[
                    "diff",
                    "--cached",
                    "--summary",
                    &selection.source,
                    "--",
                    path,
                ])?
                .is_empty(),
                "staged release metadata changes file mode or identity",
            )?;
            let actual = self.output("git", &["show", &format!(":{path}")])?;
            require(
                actual == expected.expected || actual == expected.original,
                "staged release metadata differs from saved intent",
            )?;
        }
        Ok(())
    }
    fn local_tag(&self, version: &str) -> Result<Option<String>> {
        let refs = self.git(&[
            "for-each-ref",
            "--format=%(objectname) %(refname)",
            "refs/tags",
        ])?;
        let name = format!("refs/tags/v{version}");
        Ok(refs
            .lines()
            .filter_map(|line| line.split_once(' '))
            .find(|(_, reference)| *reference == name)
            .map(|(hash, _)| hash.to_owned()))
    }

    fn remote_ready(&self, selection: &ReleaseSelection) -> Result<()> {
        require(
            self.git(&["symbolic-ref", "--quiet", "--short", "HEAD"])? == selection.branch,
            "selected release branch is not checked out",
        )?;
        let remote_ref = format!("refs/remotes/{}/{}", selection.remote, selection.branch);
        self.git(&[
            "fetch",
            "--quiet",
            "--no-tags",
            &selection.remote,
            &format!("+refs/heads/{}:{remote_ref}", selection.branch),
        ])?;
        self.git(&["merge-base", "--is-ancestor", &remote_ref, "HEAD"])?;
        Ok(())
    }
    fn commit_check(&self, selection: &ReleaseSelection) -> Result<()> {
        let validation = self.preparation_validation(selection)?;
        let surfaces = self.surfaces(selection)?;
        self.qualify_prepared(validation, &surfaces)?;
        self.check_index(selection, &surfaces)?;
        require(
            self.git(&["diff", "--name-only"])?.is_empty(),
            "release metadata has unstaged changes",
        )?;
        require(
            self.git(&["diff", "--cached", "--name-only"])?
                == self.git(&["diff", "--name-only", &selection.source])?,
            "stage exactly the release metadata",
        )
    }

    fn inspect_release_commit(&self, commit: &str) -> Result<InspectedRelease> {
        self.clean()?;
        let version = self.version(Some(commit))?;
        let prepared: PackageEvidence =
            serde_json::from_slice(&fs::read(self.receipt_path(&version, Prepared)?)?)?;
        let selection = prepared.validation.selection;
        require(
            selection.version == version,
            "release metadata differs from saved intent",
        )?;
        require(
            self.git(&["log", "-1", "--format=%P", commit])? == selection.source,
            "release parent differs from validated source",
        )?;
        require(
            self.git(&["log", "-1", "--format=%s", commit])? == format!("Release {version}"),
            "selected commit is not the release commit",
        )?;
        self.git(&["merge-base", "--is-ancestor", commit, "HEAD"])?;
        self.check_surfaces(&selection, &self.surfaces(&selection)?, Some(commit), false)?;
        Ok(InspectedRelease {
            commit: commit.to_owned(),
            selection,
        })
    }

    fn release_commit(&self) -> Result<InspectedRelease> {
        let release = self.inspect_release_commit(&self.git(&["rev-parse", "HEAD"])?)?;
        self.verify_evidence(&release.selection, &release.commit, Final, Working)?;
        Ok(release)
    }

    fn qualify_release(&self) -> Result<()> {
        self.qualify_release_at(&self.git(&["rev-parse", "HEAD"])?)
    }

    fn qualify_release_at(&self, commit: &str) -> Result<()> {
        let release = self.inspect_release_commit(commit)?;
        let selection = &release.selection;
        let version = &selection.version;
        let head = &release.commit;
        let mut evidence =
            self.verify_evidence(selection, &selection.source, Prepared, Retained)?;
        let current = self.git(&["rev-parse", "HEAD"])? == commit;
        if self.receipt_path(version, Final)?.exists() {
            // Recovery verifies the retained archive for the selected commit.
            // Cargo's working archive may belong to the newer source at HEAD.
            self.verify_evidence(
                selection,
                head,
                Final,
                if current { Working } else { Retained },
            )?;
            return Ok(());
        }
        require(
            current,
            "selected older release has no final qualification receipt; qualify that exact commit before recovery",
        )?;
        let command = self.final_package_command()?;
        self.run_command(&command)?;
        require(
            self.git(&["rev-parse", "HEAD"])? == commit
                && self.inspect_release_commit(commit)? == release,
            "release changed during final qualification",
        )?;
        self.verify_inputs(&evidence.validation, true)?;
        require(
            Self::sha256(&self.root.join("Cargo.lock"))? == evidence.lock_sha256,
            "final packaging changed selected dependencies",
        )?;
        evidence.commands.push(command);
        let evidence = PackageEvidence::record(
            self,
            evidence.validation,
            head.clone(),
            evidence.lock_sha256,
            evidence.commands,
        )?;
        self.write_evidence(&evidence, Final)
    }

    fn selected_committed_check(&self, selection: &ReleaseSelection, commit: &str) -> Result<()> {
        require(
            self.inspect_release_commit(commit)?.selection == *selection,
            "commit identifies another release intent",
        )?;
        self.qualify_release_at(commit)
    }

    fn tagged_check(&self, selection: &ReleaseSelection, commit: &str) -> Result<InspectedRelease> {
        let release = self.inspect_release_commit(commit)?;
        require(
            release.selection == *selection,
            "tagged release identifies another intent",
        )?;
        self.verify_evidence(
            selection,
            commit,
            Final,
            if self.git(&["rev-parse", "HEAD"])? == commit {
                Working
            } else {
                Retained
            },
        )?;
        self.check_tag(&release.selection.version, &release.commit)?;
        Ok(release)
    }

    fn push_check(&self, selection: &ReleaseSelection, commit: &str) -> Result<()> {
        let release = self.tagged_check(selection, commit)?;
        self.remote_ready(selection)?;
        require(
            self.tagged_check(selection, commit)? == release,
            "release changed during remote inspection",
        )?;
        Ok(())
    }
    fn check_tag(&self, version: &str, head: &str) -> Result<String> {
        let tag = self.local_tag(version)?.ok_or("release tag is missing")?;
        self.output("bash", &[TAG_CHECKER, head, version])?;
        require(
            self.local_tag(version)?.as_deref() == Some(tag.as_str()),
            "release tag changed during admission",
        )?;
        Ok(tag)
    }
}

#[expect(
    clippy::redundant_pub_crate,
    reason = "only the binary entrypoint calls this module"
)]
pub(super) fn main() -> Result<()> {
    let arguments = Arguments::parse();
    let repo = Repository {
        root: env::current_dir()?,
        exec: Processes,
    };
    match arguments.command {
        ToolCommand::Version => println!("{}", repo.version(None)?),
        ToolCommand::Msrv => println!("{}", repo.msrv()?),
        ToolCommand::Toolchain => println!("{}", repo.toolchain()?),
        ToolCommand::Target => println!("{}", repo.target()?.display()),
        ToolCommand::EnsureClean => repo.clean()?,
        ToolCommand::WasmSize => repo.wasm_size()?,
        ToolCommand::ReleasePreflight => repo.preflight(&ReleaseSelection::from_env(&repo)?)?,
        ToolCommand::ReleaseVerify => repo.verify(&ReleaseSelection::from_env(&repo)?)?,
        ToolCommand::ReleasePrepareVersion => {
            repo.prepare_selected(&ReleaseSelection::from_env(&repo)?)?;
        }
        ToolCommand::ReleasePreparedCheck => {
            repo.prepared(&ReleaseSelection::from_env(&repo)?)?;
        }
        ToolCommand::ReleaseFiles => {
            for path in repo.surfaces(&ReleaseSelection::from_env(&repo)?)?.keys() {
                print!("{path}\0");
            }
        }
        ToolCommand::ReleaseCommitCheck => {
            repo.commit_check(&ReleaseSelection::from_env(&repo)?)?;
        }
        ToolCommand::ReleaseCommittedCheck => {
            repo.selected_committed_check(
                &ReleaseSelection::from_env(&repo)?,
                &release_commit_from_env()?,
            )?;
        }
        ToolCommand::ReleaseTaggedCheck => {
            repo.tagged_check(
                &ReleaseSelection::from_env(&repo)?,
                &release_commit_from_env()?,
            )?;
        }
        ToolCommand::ReleasePushCheck => {
            repo.push_check(
                &ReleaseSelection::from_env(&repo)?,
                &release_commit_from_env()?,
            )?;
        }
        ToolCommand::QualifyRelease => repo.qualify_release()?,
        ToolCommand::Publish { dry_run } => {
            // This policy must reject an invalid environment value even when
            // --dry-run was explicitly supplied; it is not a CLI flag default.
            let flag = match env::var("PUBLISH_DRY_RUN") {
                Ok(flag) => flag,
                Err(env::VarError::NotPresent) => "0".to_owned(),
                Err(error) => return Err(error.into()),
            };
            require(flag == "0" || flag == "1", "PUBLISH_DRY_RUN must be 0 or 1")?;
            repo.publish(dry_run || flag == "1")?;
        }
    }
    Ok(())
}
