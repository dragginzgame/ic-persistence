# IC Storage

IC Storage exposes governed stable collections through `ic_storage::adapter`.
All public collections require IC Memory runtime handles; memory extraction is
not exposed. Governed region views support retained trees sharing an allocation
with application frames. Collections retain the upstream stable structures' persisted formats.
Read the [package guide](crates/ic-storage/README.md) for the supported API and
allocation ownership boundary.

The independent B-tree and comparison benchmark have been removed. Their design
and performance investigation is retained as [historical research](docs/archive/private-btree/README.md).

## Development

The reviewed [Shared Tooling snapshot](.shared-tooling.snapshot) pins version
0.23.0 at commit `0b351cd2ba2bcd8927b783a5fec89afc8013f708`.
Read [AGENTS.md](AGENTS.md) for local contributor instructions and
[the setup guide](docs/local-setup.md) for Linux/macOS prerequisites.
Rust 1.99.1 is the development toolchain. The Rust 2024 library lives in
`crates/ic-storage`, inherits root workspace metadata, and supports Rust 1.91.0.
See [upstream ownership](docs/upstreams.md) for dependency responsibilities.

```sh
make install-tools
make tools-check
make install-hooks
make help
```

Common setup prepares host and Cargo tools. IC executables are optional product
selections. Make selects the repository's tools automatically. For direct shell use:

```sh
export PATH="$PWD/.tools/host/bin:$PWD/.tools/rust/common/bin:$PATH"
```

Focused checks include `make shared-tooling-check`, `make check-doc-links`,
`make check-license`, `make check-pins` and `make check-storage-boundary`.
Rust checks are `make fmt-check`, `make check`, `make clippy`, `make docs-check`,
`make test`, `make wasm-check` and `make msrv`. Use the shared validation cadence
for development; `make ci` is the complete release gate. Linux and macOS are
configured in CI. Historical engine benchmarks do not qualify a future storage
implementation.

Authorized releases use `make release-patch`, `make release-minor` or
`make release-major`; `make release-resume VERSION=x.y.z` resumes the exact
recorded candidate. Publication uses `make publish-check` and `make publish`
from clean source at an annotated release tag. See [the release contract](docs/releases.md).

Licensed under MIT; see LICENSE.
