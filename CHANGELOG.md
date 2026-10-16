# Changelog

## [0.1.1] - 2026-10-16

- Synchronize the local Cargo lockfile identity with the current 0.1.0 manifest
  so locked release preflight can run without changing dependency selections.
- Replacing a stored value reuses its overflow pages and leaves tree topology
  unchanged. Same-size or shrinking updates no longer need a second payload's
  worth of capacity; failed growth still preserves the existing data.
- Qualify semantic encoded-key ordering through multi-level tree mutations,
  mixed inline/overflow payloads, bounded bidirectional scans and reopening.

## [0.1.0]

- Establish the repository with the Dragginzgame engineering baseline, pinned
  developer tooling, Linux/macOS CI and standard release commands.
- Add the Rust workspace and library scaffold, upstream ownership guidance,
  Cargo release metadata, formatting hooks and guarded crates.io publication.
- Add an independent stable-memory B-tree with custom key ordering, lazy
  bidirectional scans, bounded large values, reusable pages and fallible
  corruption/capacity handling. Its new format requires fresh storage; IcyDB
  adoption and canister recovery qualification remain separate.
