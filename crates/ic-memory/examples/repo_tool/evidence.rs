//! Receipt schemas and coordinated archive identities; JSON shapes are stable.
use super::*;

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
pub(super) struct ValidationEvidence {
    pub(super) selection: ReleaseSelection,
    pub(super) lock: Vec<u8>,
    pub(super) identities: [String; 3],
    pub(super) configuration: BTreeMap<String, String>,
    pub(super) command: Vec<String>,
}

///
/// PackageEvidence
///
/// Qualified package bound to the saved validation and current root lockfile.
///

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PackageEvidence {
    pub(super) validation: ValidationEvidence,
    pub(super) package_head: String,
    pub(super) lock_sha256: String,
    pub(super) package_sha256: String,
    pub(super) storage_sha256: String,
    pub(super) metrics_sha256: String,
    pub(super) logging_sha256: String,
    pub(super) commands: Vec<Vec<String>>,
}

// These names describe existing receipt/observation choices without changing JSON.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ReceiptStage {
    Prepared,
    Final,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ArchiveLocation {
    Working,
    Retained,
}

impl PackageEvidence {
    // Keep the fixed receipt-field mapping beside its read projection. Callers
    // carry complete evidence rather than positional arrays of package digests.
    pub(super) fn record<E: Execute>(
        repo: &Repository<E>,
        validation: ValidationEvidence,
        package_head: String,
        lock_sha256: String,
        commands: Vec<Vec<String>>,
    ) -> Result<Self> {
        let [memory, storage, metrics, logging] = packages();
        let version = &validation.selection.version;
        repo.version_parts(version)?;
        let paths = repo.artifact_paths()?;
        let package_sha256 = repo.record_package_in(&paths, memory, version)?;
        let storage_sha256 = repo.record_package_in(&paths, storage, version)?;
        let metrics_sha256 = repo.record_package_in(&paths, metrics, version)?;
        let logging_sha256 = repo.record_package_in(&paths, logging, version)?;
        Ok(Self {
            validation,
            package_head,
            lock_sha256,
            package_sha256,
            storage_sha256,
            metrics_sha256,
            logging_sha256,
            commands,
        })
    }

    pub(super) fn archives(&self) -> impl Iterator<Item = (&'static str, &str)> {
        packages().into_iter().zip([
            self.package_sha256.as_str(),
            self.storage_sha256.as_str(),
            self.metrics_sha256.as_str(),
            self.logging_sha256.as_str(),
        ])
    }
}
