# IC Storage agent instructions

Apply the [Dragginzgame baseline](DRAGGINZGAME.md). Its exact source revision
and exported files are recorded in [.shared-tooling.snapshot](.shared-tooling.snapshot).
This file is the repository-local overlay; do not edit snapshot-owned files.

Read [the current handoff](docs/status/current.md) at session start.
IC Storage is currently a repository scaffold with no product implementation.
Do not infer storage formats, canister identities or deployment targets from its name.
The virtual root workspace owns the `ic-storage` library under `crates/`.
Keep shared metadata and dependency selections at the root; see
[upstream ownership](docs/upstreams.md) before adding host helpers.

Use the shared [validation cadence](DRAGGINZGAME.md#validation-and-evidence).
Focused setup checks are `make shared-tooling-check`, `make check-doc-links`,
`make check-license`, `make check-pins` and `make check-release-commands`.
Rust checks are `make fmt-check`, `make check`, `make clippy`,
`make docs-check` and `make msrv`. `make ci` is the complete scaffold gate; extend it when
implementation is added. Shared Tooling owns tests of the exported helpers.

Common setup and release commands come from `make/consumer.mk`.
The initial version is `0.0.0`; `0.1.0` is the pending initial release.
Releases use the local Cargo adapter and shared metadata helpers and direct delivery to
`origin/main`. The shared contribution and release authorization rules apply.
`make publish-check` and `make publish` require clean source at an annotated
release tag. Publication uses the shared attempt runner and crates.io.
Host scope and current qualification limits are documented in [README.md](README.md).
