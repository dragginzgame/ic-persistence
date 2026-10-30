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
    release_metadata::{
        DependencyVersionProjection, render_workspace_lock, render_workspace_manifest,
    },
};
use ic_host_artifacts::artifact::Sha256Digest;
use ic_host_fs::durable::{PublicationMode, WriteOptions};
use ic_host_fs::read::hash_file;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    env,
    error::Error,
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

mod artifact_paths;
use artifact_paths::ArtifactPaths;
mod artifacts;
mod metadata;
mod packages;
mod release;
mod source;
use source::SurfaceState;
mod wasm;
use packages::{package_manifest, packages};
mod processes;
use processes::{Execute, Processes};
mod publication;
mod qualification;
use qualification::LockState;
mod evidence;
mod preparation;
use ArchiveLocation::{Retained, Working};
use ReceiptStage::{Final, Prepared};
use evidence::{ArchiveLocation, PackageEvidence, ReceiptStage, ValidationEvidence};

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

const PACKAGE_README: &str = "crates/ic-memory/README.md";
const CHANGELOG_FINALIZER: &str = "scripts/ci/finalize-release-changelog.awk";
const RELEASE_INPUT_ADMISSION: &str = "scripts/ci/validate-release-inputs.pl";
const TAG_CHECKER: &str = "scripts/ci/check-release-tag.sh";
const REPLACE_FILE_OPTIONS: WriteOptions = WriteOptions {
    mode: PublicationMode::Replace,
    permissions: 0o666,
};

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

impl<E: Execute> Repository<E> {
    fn run(&self, program: &str, args: &[&str]) -> Result<()> {
        self.exec.execute(&self.root, program, args)
    }

    fn output(&self, program: &str, args: &[&str]) -> Result<String> {
        self.exec.capture(&self.root, program, args)
    }

    fn git(&self, args: &[&str]) -> Result<String> {
        Ok(self.output("git", args)?.trim().to_owned())
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

    fn identities(&self) -> Result<[String; 3]> {
        Ok([
            self.output("cargo", &[&format!("+{}", self.toolchain()?), "--version"])?,
            self.output("rustc", &[&format!("+{}", self.toolchain()?), "-Vv"])?,
            self.output("rustc", &[&format!("+{}", self.msrv()?), "-Vv"])?,
        ])
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
