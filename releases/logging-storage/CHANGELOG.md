# Logging Storage release notes

Current releases use the Storage naming. [Earlier published history](../logging-stable/CHANGELOG.md)
retains its original package and release identities.

## [0.64.0] - Unreleased

### ⚠️ Breaking

- Adopt fallible Storage map operations and propagate collection admission errors
  through `StoreError::Storage`; host map constructors now return `Result`.

- Rename `ic-logging-stable` to `ic-logging-storage`. Update Cargo dependencies and Rust
  imports; the new package is a separate crates.io identity with no compatibility
  facade. [Adoption](../../docs/migration/storage-naming.md).

- Adopt Storage's owned allocator and map format. Hosts must use fresh/reset
  whole backing and rebuild against its owned traits and namespace admission;
  adapter Wire schemas remain unchanged.

- Consume allocation and runtime types from the consolidated `ic-storage` crate;
  hosts must replace their `ic-memory` imports. [Details](0.64.md).

- Adopt Logging 0.24.0 for public log, limit and restoration-error types. Hosts
  exchanging these values must align their Logging dependency. Checkpoint bytes,
  retention, limits and the Rust 1.99.1 support floor are unchanged. [Details](0.64.md).
