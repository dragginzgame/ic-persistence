# IC Storage

IC Storage is a new Dragginzgame repository. It currently contains the shared
engineering baseline, a Rust library scaffold, developer setup, CI and release infrastructure.
Storage functionality and public APIs have not been implemented.

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
qualify the Rust scaffold. `make ci` runs the complete scaffold validation. CI selects Ubuntu 24.04 x86_64
and macOS 15 arm64. Native hosted validation is pending the initial push;
local Linux checks do not establish macOS qualification.

## Releases

`make version` reads the root Cargo workspace version. The initial `0.0.0` is unpublished, with an
undated `0.1.0` entry in [CHANGELOG.md](CHANGELOG.md).
After the initial scaffold is committed and pushed, an authorized
`make release-minor` prepares the initial release. Subsequent releases use
`make release-patch`, `make release-minor` or `make release-major` according to
the change. These commands validate, update metadata, commit, push and tag
through the [shared release contract](docs/releases.md), targeting `origin/main`.
Cargo.toml, Cargo.lock and the changelog are updated together.

`make package` builds a local crate archive. On a clean, committed source tree
at its annotated release tag, `make publish-check` performs a crates.io dry run
and `make publish` uploads `ic-storage` using the caller's Cargo credentials.
Publication attempts retain evidence under `target/publication`; inspect and
reconcile an interrupted attempt before retrying. Publication remains separate
from release/tag delivery. The current crate is an empty scaffold.

## License

[MIT](LICENSE), copyright 2026 Toko Studios Ltd.
