//! Artifacts responsibilities for the repository adapter.
use super::*;

impl<E: Execute> Repository<E> {
    pub(super) fn target(&self) -> Result<PathBuf> {
        let target = self.exec.cargo_target(&self.root)?;
        require(
            target.is_absolute(),
            "Cargo target_directory must be absolute",
        )?;
        Ok(target)
    }

    pub(super) fn artifact_paths(&self) -> Result<ArtifactPaths> {
        Ok(ArtifactPaths::new(self.target()?))
    }

    pub(super) fn receipt_path(&self, version: &str, stage: ReceiptStage) -> Result<PathBuf> {
        self.version_parts(version)?;
        Ok(self.artifact_paths()?.receipt(version, stage))
    }

    #[cfg(test)]
    pub(super) fn package_path(&self, package: &str, version: &str) -> Result<PathBuf> {
        self.version_parts(version)?;
        Ok(self.artifact_paths()?.package(package, version))
    }

    pub(super) fn sha256(path: &Path) -> Result<String> {
        // Preserve the existing file-size contract; hashing uses constant memory.
        Ok(hash_file(path, u64::MAX)?.sha256.to_string())
    }

    #[cfg(test)]
    pub(super) fn retained_package(&self, digest: &str) -> Result<PathBuf> {
        let digest: Sha256Digest = digest.parse()?;
        Ok(self.artifact_paths()?.retained(digest))
    }

    #[cfg(test)]
    pub(super) fn record_package(&self, name: &str, version: &str) -> Result<String> {
        self.version_parts(version)?;
        self.record_package_in(&self.artifact_paths()?, name, version)
    }

    pub(super) fn record_package_in(
        &self,
        paths: &ArtifactPaths,
        name: &str,
        version: &str,
    ) -> Result<String> {
        let package = paths.package(name, version);
        let digest = hash_file(&package, u64::MAX)?.sha256;
        let retained = paths.retained(digest);
        if !retained.exists() {
            self.exec.copy_verified(&package, &retained, digest)?;
        }
        require(
            hash_file(&retained, u64::MAX)?.sha256 == digest,
            "retained package differs from its digest",
        )?;
        Ok(digest.to_string())
    }
}
