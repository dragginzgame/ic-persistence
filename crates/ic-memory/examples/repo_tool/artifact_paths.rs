//! Paths projected from one admitted Cargo target-directory observation.
use super::*;

pub(super) struct ArtifactPaths {
    target: PathBuf,
}

impl ArtifactPaths {
    pub(super) const fn new(target: PathBuf) -> Self {
        Self { target }
    }

    pub(super) fn package(&self, package: &str, version: &str) -> PathBuf {
        self.target
            .join("package")
            .join(format!("{package}-{version}.crate"))
    }

    pub(super) fn retained(&self, digest: Sha256Digest) -> PathBuf {
        self.target
            .join("release-validation/artifacts")
            .join(format!("{digest}.crate"))
    }
    pub(super) fn publication(&self) -> PathBuf {
        self.target.join("release-validation/publication")
    }
}
