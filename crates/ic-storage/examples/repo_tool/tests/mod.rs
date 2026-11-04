//! Release effects are substituted; native Git checks use disposable indexes.
//! No fixture commits, tags, pushes or live publication occur.

use super::*;
use ic_host_process::tool::{
    CompletedExecution, OutputLimit, OutputLimits, ToolError, capture_command,
};
use std::{cell::RefCell, process::Command};

const PACKAGE_README: &str = "crates/ic-storage/README.md";

const fn packages() -> [&'static str; 3] {
    ["ic-storage", "ic-metrics-storage", "ic-logging-storage"]
}

mod build;
mod files;
mod git;
mod registry;
use build::{BuildEffects, BuildFault};
use files::{FileEffects, FileFault};
use git::{GitFault, GitState};
use registry::{RegistryEffects, RegistryFault};

mod fixtures;
use fixtures::*;

mod archives;
mod configuration;
mod preparation;
mod process_io;
mod publication;
mod recovery;
mod source_admission;

mod groups;

mod validation;

// Private fixture helpers; excluded from production compilation.
impl<E: Execute> Repository<E> {
    pub(super) fn package_path(&self, package: &str, version: &str) -> Result<PathBuf> {
        self.version_parts(version)?;
        Ok(self.artifact_paths()?.package(package, version))
    }

    pub(super) fn retained_package(&self, digest: &str) -> Result<PathBuf> {
        let digest: Sha256Digest = digest.parse()?;
        Ok(self.artifact_paths()?.retained(digest))
    }

    pub(super) fn record_package(&self, name: &str, version: &str) -> Result<String> {
        self.version_parts(version)?;
        self.record_package_in(&self.artifact_paths()?, name, version)
    }
}

// Private fixture helpers; excluded from production compilation.
impl<E: Execute> Repository<E> {
    pub(super) fn project_catalog(
        &self,
        text: &str,
        previous: &str,
        version: &str,
    ) -> Result<String> {
        self.project_group_catalog(&self.release_group(None)?, text, previous, version)
    }

    pub(super) fn replace_lock_version(
        &self,
        text: &str,
        previous: &str,
        version: &str,
    ) -> Result<String> {
        Self::replace_group_lock_version(&self.release_group(None)?, text, previous, version)
    }
}
