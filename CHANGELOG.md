# Changelog

## [0.1.0]

- Establish the repository with the Dragginzgame engineering baseline, pinned
  developer tooling, Linux/macOS CI and standard release commands.
- Add the Rust workspace and library scaffold, upstream ownership guidance,
  Cargo release metadata, formatting hooks and guarded crates.io publication.
- Add an independent stable-memory B-tree with custom key ordering, lazy
  bidirectional scans, bounded large values, reusable pages and fallible
  corruption/capacity handling. Its new format requires fresh storage; IcyDB
  adoption and canister recovery qualification remain separate.
