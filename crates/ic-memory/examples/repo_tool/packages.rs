//! Consumer-owned coordinated publication order, also read by shell adapters.

use std::sync::LazyLock;

static PACKAGES: LazyLock<[&'static str; 4]> = LazyLock::new(|| {
    serde_json::from_str(include_str!("packages.json")).expect("checked-in package roster")
});

pub(super) fn packages() -> [&'static str; 4] {
    *PACKAGES
}

pub(super) fn package_manifest(name: &str) -> String {
    format!("crates/{name}/Cargo.toml")
}
