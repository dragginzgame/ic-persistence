//! Consumer-owned coordinated publication order, also read by shell adapters.

pub(super) fn packages() -> [&'static str; 3] {
    serde_json::from_str(include_str!("packages.json")).expect("checked-in package roster")
}

// The unpublished logging adapter inherits the release version, but has no
// archive or publication authority until its implementation is qualified.
pub(super) fn workspace_packages() -> [&'static str; 4] {
    let [memory, storage, metrics] = packages();
    let [logging]: [&str; 1] = serde_json::from_str(include_str!("unpublished-packages.json"))
        .expect("checked-in unpublished package roster");
    [memory, storage, metrics, logging]
}

pub(super) fn package_manifest(name: &str) -> String {
    format!("crates/{name}/Cargo.toml")
}
