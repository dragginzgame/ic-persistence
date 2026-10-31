//! Group-bound receipts and selected archive identities.
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
    pub(super) archives: BTreeMap<String, String>,
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
        let version = &validation.selection.version;
        repo.version_parts(version)?;
        let paths = repo.artifact_paths()?;
        let archives = repo
            .selected_packages()?
            .into_iter()
            .map(|name| {
                let digest = repo.record_package_in(&paths, &name, version)?;
                Ok((name, digest))
            })
            .collect::<Result<_>>()?;
        Ok(Self {
            validation,
            package_head,
            lock_sha256,
            archives,
            commands,
        })
    }

    pub(super) fn archives(&self) -> impl Iterator<Item = (&str, &str)> {
        self.archives
            .iter()
            .map(|(name, digest)| (name.as_str(), digest.as_str()))
    }
}
