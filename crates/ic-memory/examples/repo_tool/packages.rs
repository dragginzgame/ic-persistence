//! Consumer-owned coordinated publication order, also read by shell adapters.

pub(super) fn packages() -> [&'static str; 3] {
    serde_json::from_str(include_str!("packages.json")).expect("checked-in package roster")
}

pub(super) fn package_manifest(name: &str) -> String {
    format!("crates/{name}/Cargo.toml")
}
