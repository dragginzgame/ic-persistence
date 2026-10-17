# IC Storage agent instructions

Apply the [Dragginzgame baseline](DRAGGINZGAME.md). Its exact source revision
and exported files are recorded in [.shared-tooling.snapshot](.shared-tooling.snapshot).
This file is the repository-local overlay; do not edit snapshot-owned files.
The dependency-boundary helper is part of the root snapshot and uses local policy
in `ci/cargo-dependency-boundaries.json`.

Read [the current handoff](docs/status/current.md) at session start.
IC Storage owns the governed collection adapter. Read the package README before
changing storage behavior. Upstream owns collection formats; IC Memory owns
allocation. IcyDB owns application codecs and recovery. The independent B-tree
and its benchmark are retired; their bytes are incompatible with upstream
collection formats. Historical research lives under
[the archive](docs/archive/private-btree/README.md).
The virtual root workspace owns the `ic-storage` library under `crates/`.
Keep shared metadata and dependency selections at the root; see
[upstream ownership](docs/upstreams.md) before adding host helpers.

Application collections consume IC Memory runtime handles through IC Storage.
Only the `ic-storage` infrastructure package may depend directly on
`ic-stable-structures`; consumers must use the adapter. Do not expose unrestricted
memory/manager constructors. `ic-storage::adapter` owns the supported collection
and codec surface; IC Memory owns allocation authority. Both infrastructure
packages may use the substrate privately.
`make check-storage-boundary` checks workspace dependency declarations.

Use the shared [validation cadence](DRAGGINZGAME.md#validation-and-evidence).
Focused setup checks are `make shared-tooling-check`, `make check-doc-links`,
`make check-license`, `make check-pins` and `make check-release-commands`.
Rust checks are `make fmt-check`, `make check`, `make clippy`,
`make docs-check`, `make test`, `make wasm-check` and `make msrv`. `make ci` is the complete repository gate. Shared Tooling owns tests of the exported helpers.

Common setup and release commands come from `make/consumer.mk`.
The pending implementation batch is `0.5.2`; release metadata remains maintainer-owned.
Releases use the local Cargo adapter and shared metadata helpers and direct delivery to
`origin/main`. The shared contribution and release authorization rules apply.
`make publish-check` and `make publish` require clean source at an annotated
release tag. Publication uses the shared attempt runner and crates.io.
Host scope and current qualification limits are documented in [README.md](README.md).
