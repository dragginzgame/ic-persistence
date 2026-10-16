# IC Storage

IC Storage implements an independent stable-memory B-tree for Internet Computer
applications, shaped by IcyDB's canonical-store requirements. It supports custom
key order, large values, lazy bidirectional scans and reusable pages.

Read [the package guide](crates/ic-storage/README.md) for the API and persistence
contract, and [the IcyDB boundary](docs/design/icydb-btree.md) for consumer evidence.
The tree has not yet been integrated into IcyDB or qualified in a live canister.

## Development

The reviewed [Shared Tooling snapshot](.shared-tooling.snapshot) pins version
0.15.4 at commit `879eb2b021522252a67e908dbc217a0f4efb2a6f`.
Read [AGENTS.md](AGENTS.md) for local contributor instructions and
[the setup guide](docs/local-setup.md) for Linux/macOS prerequisites.
Rust 1.99.0 is the development toolchain. The Rust 2024 library lives in
`crates/ic-storage`, inherits root workspace metadata, and supports Rust 1.85.0.
See [upstream ownership](docs/upstreams.md) for Shared Tooling and IC Host Tooling.

```sh
make install-tools
make tools-check
make install-hooks
make help
```

Make selects the repository's tools automatically. For direct shell use:

```sh
export PATH="$PWD/.tools/host/bin:$PWD/.tools/ic/bin:$PWD/.tools/rust/common/bin:$PATH"
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

`make version` reads the root Cargo workspace version. The pending `0.1.2`
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
