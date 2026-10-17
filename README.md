# IC Storage

IC Storage exposes governed stable collections through `ic_storage::adapter`.
All public collections consume IC Memory runtime handles; memory extraction is
not exposed. They retain the upstream stable structures' persisted formats.

The independent page-based B-tree remains private to qualification tests and the
unpublished benchmark. No downstream API or feature exposes it, and no Canic or
IcyDB store is switched to its format. See [the private format contract](docs/design/experimental-btree.md)
and [the IcyDB requirements](docs/design/icydb-btree.md) for its implementation
and eventual adoption criteria. Read [the package guide](crates/ic-storage/README.md)
for the supported adapter and allocation ownership boundary.

## Development

The reviewed [Shared Tooling snapshot](.shared-tooling.snapshot) pins version
0.23.0 at commit `0b351cd2ba2bcd8927b783a5fec89afc8013f708`.
Read [AGENTS.md](AGENTS.md) for local contributor instructions and
[the setup guide](docs/local-setup.md) for Linux/macOS prerequisites.
Rust 1.99.0 is the development toolchain. The Rust 2024 library lives in
`crates/ic-storage`, inherits root workspace metadata, and supports Rust 1.91.0.
See [upstream ownership](docs/upstreams.md) for Shared Tooling and IC Host Tooling.

The non-published `ic-storage-bench` package uses Rust 1.99 and IC Testkit for
canister instruction measurements; it does not raise the storage library's floor.
`make bench-run POCKET_IC_BIN=/absolute/path/to/pocket-ic` builds the release Wasm
and runs fresh local PocketIC instances. Supply an already-admitted 16.x server;
the command does not provision one. See the [measurement contract and results](docs/design/icydb-btree.md#canister-instruction-baseline)
before interpreting the CSV output. `make msrv` checks the public library;
workspace checks and `make wasm-check` also cover benchmark compilation.

For variable-size canonical-store workloads, build the runner with
`cargo build -p ic-storage-bench --bin ic-storage-bench --locked`, then pass
`workloads` as its third argument after the server and Wasm paths. The
[workload investigation](docs/design/canonical-workloads.md) documents the exact
command, model checks, copy trace, measured results and experimental limitations.

```sh
make install-tools
make tools-check
make install-hooks
make help
```

Common setup prepares host and Cargo tools. IC executable installation is optional;
the benchmark takes an explicitly supplied PocketIC server rather than selecting
one from the shared IC tool bundle.

Make selects the repository's tools automatically. For direct shell use:

```sh
export PATH="$PWD/.tools/host/bin:$PWD/.tools/rust/common/bin:$PATH"
```

Focused checks include `make shared-tooling-check`, `make check-doc-links`,
`make check-license`, `make check-pins` and `make check-release-commands`.
`make fmt-check`, `make check`, `make clippy`, `make docs-check` and `make msrv`
qualify the Rust library. `make test` runs storage tests and documentation
examples; `make wasm-check` checks the canister compilation target.
`make ci` runs the complete repository validation. CI selects Ubuntu 24.04 x86_64
and macOS 15 arm64. Native hosted validation is pending the initial push;
local Linux checks do not establish macOS qualification.

## Releases

`make version` reads the root Cargo workspace version. The pending `0.5.0`
changes are recorded in [CHANGELOG.md](CHANGELOG.md). Authorized releases use
`make release-patch`, `make release-minor` or `make release-major` according to
the change. These commands validate, update metadata, commit, push and tag
through the [shared release contract](docs/releases.md), targeting `origin/main`.
Cargo.toml, Cargo.lock and the changelog are updated together.

`make package` builds a local crate archive. On a clean, committed source tree
at its annotated release tag, `make publish-check` performs a crates.io dry run
and `make publish` uploads `ic-storage` using the caller's Cargo credentials.
Publication attempts retain evidence under `target/publication`; inspect and
reconcile an interrupted attempt before retrying. Publication remains separate
from release/tag delivery.

## License

[MIT](LICENSE), copyright 2026 Toko Studios Ltd.
