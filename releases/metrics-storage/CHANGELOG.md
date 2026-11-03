# Metrics Storage release notes

Current releases use the Storage naming. [Earlier published history](https://github.com/dragginzgame/ic-storage/blob/23deb81c48d58558ac3b0ce36bf266ddc25dca91/releases/metrics-stable/CHANGELOG.md)
retains its original package and release identities.

## [0.67.0]

### ⚠️ Breaking

- Adopt Wire 0.16 codec errors and Metrics 0.28 measurement/restoration types.
  Align direct dependencies when exchanging those values. Services own report
  DTO schemas; stored rows, arithmetic and validated restoration remain unchanged ([#154 — Remove Metrics-native schema coupling](https://github.com/dragginzgame/ic-storage/issues/154)).
  [Details](0.67.md).

## [0.66.0] - 2026-11-02

### ⚠️ Breaking

- Expose Wire 0.15 codec errors while preserving checkpoint bytes and budgets.
  Align native metric schemas with the same Wire identity through Metrics 0.27.
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

- Adopt Metrics 0.27.0 for public measurement and restoration-error types. Hosts
  exchanging these values must align their Metrics dependency. Arithmetic,
  snapshots, limits and the Rust 1.99.1 support floor are unchanged. [Details](0.66.md).

