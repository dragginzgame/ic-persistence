# Current handoff

The repository adopts Shared Tooling 0.15.4 through the root snapshot.
A virtual Cargo workspace owns `crates/ic-storage`, an empty Rust 2024 library
with a qualified Rust 1.85.0 floor. Storage behavior remains to be implemented.
Shared Tooling and IC Host Tooling ownership is described in ../upstreams.md.

Make provides tool setup, formatting/hooks, Rust and governance checks, standard
Cargo release commands and guarded crates.io publish/publish-check commands.
The root Cargo version is 0.0.0; the pending changelog is 0.1.0.
No release or package upload has occurred. Hosted Linux/macOS CI qualification
requires the initial source push.
