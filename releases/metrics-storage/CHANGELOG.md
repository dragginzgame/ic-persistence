# Metrics Storage release notes

Current releases use the Storage naming. [Earlier published history](https://github.com/dragginzgame/ic-storage/blob/23deb81c48d58558ac3b0ce36bf266ddc25dca91/releases/metrics-stable/CHANGELOG.md)
retains its original package and release identities.

## [0.66.0] - Unreleased

### ⚠️ Breaking

- Expose Wire 0.15 codec errors while preserving checkpoint bytes and budgets.
  Native Metrics schema alignment awaits its separately published supplier.
- Adopt fallible Storage map operations and propagate collection admission errors
  through `StoreError::Storage`; host map constructors now return `Result`.

- Rename `ic-metrics-stable` to `ic-metrics-storage`. Update Cargo dependencies and Rust
  imports; the new package is a separate crates.io identity with no compatibility
  facade. [Adoption](../../docs/migration/storage-naming.md).

- Adopt Storage's owned allocator and map format. Hosts must use fresh/reset
  whole backing and rebuild against its owned traits and namespace admission;
  adapter Wire schemas remain unchanged.

- Consume allocation and runtime types from the consolidated `ic-storage` crate;
  hosts must replace their `ic-memory` imports. [Details](0.66.md).

- Adopt Metrics 0.26.0 for public measurement and restoration-error types. Hosts
  exchanging these values must align their Metrics dependency. Arithmetic,
  snapshots, limits and the Rust 1.99.1 support floor are unchanged. [Details](0.66.md).

