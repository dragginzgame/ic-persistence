//! Manifest paths shared by consumer adapters.

pub(super) fn package_manifest(name: &str) -> String {
    format!("crates/{name}/Cargo.toml")
}

#[cfg(test)]
pub(super) const fn packages() -> [&'static str; 4] {
    [
        "ic-memory",
        "ic-storage",
        "ic-metrics-stable",
        "ic-logging-stable",
    ]
}
