# Changelog

## [0.1.3]

- Compare upstream storage with IcyDB's declared key/value bounds across seven
  synthetic index, journal and row profiles, including read/write bytes, calls
  and reserved capacity. Add regular bounded-profile mutation/reopen coverage.
- Record how upstream bound metadata changes the earlier capacity comparison;
  actual IcyDB codecs and canister instruction costs remain unqualified.

## [0.1.2] - 2026-10-16

- Add direct differential tests against ic-stable-structures 0.7.2 and a heap
  model, including seeded mutation campaigns, semantic key ordering, range
  bounds, large payloads, reopen cycles and rejection of upstream-format bytes.
- Compare native read/write traffic and capacity with upstream trees, including
  bounded index types; record both small-entry costs and large-payload benefits.
- Reuse the located node and read the previous overflow value once during
  replacement, halving read traffic in the native index/journal/row fixtures.
  The public API, persisted format and staged-write failure contract are unchanged.
- Add an opt-in stable-memory traffic qualification for gets, replacements,
  forward/reverse key scans and removals. Native byte counts do not establish
  canister instruction costs or production IcyDB performance.

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
