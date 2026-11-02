//! Manifest paths shared by consumer adapters.

pub(super) fn package_manifest(name: &str) -> String {
    format!("crates/{name}/Cargo.toml")
}

#[cfg(test)]
pub(super) const fn packages() -> [&'static str; 3] {
    ["ic-storage", "ic-metrics-storage", "ic-logging-storage"]
}
