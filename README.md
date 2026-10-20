# IC Persistence

Allocation governance and governed stable collections for Internet Computer
canisters, maintained in one repository and published as two crates.

| Crate | Responsibility | Guide |
| --- | --- | --- |
| `ic-memory` | Allocation identities, admission, runtime bootstrap and exclusive leases | [Memory](crates/ic-memory/README.md) |
| `ic-storage` | Governed collection adapters over Memory leases | [Storage](crates/ic-storage/README.md) |

Storage depends on Memory. There is no umbrella crate. Upstream
`ic-stable-structures` retains collection algorithms and persisted formats;
applications retain lifecycle, codec and recovery policy. Metrics remains outside
this repository. See [ownership](docs/ownership.md) and [safety](SAFETY.md).

The root Cargo workspace owns both packages, one dependency catalog and one
lockfile, with Rust 1.91 as the minimum. Run Make and Cargo commands here.
The independent [installed runtime runner](testing/runtime-qualification/README.md)
retains its host-only lockfile. Archive consumer qualification uses isolated,
generated workspaces so a workspace path dependency cannot mask missing package
contents or a second Memory identity.

[Setup](docs/host-support.md) · [Releasing](RELEASING.md) ·
[Documentation](docs/README.md) · [Migration map](docs/migration/ic-persistence.md) ·
[Current handoff](docs/status/current.md) · [Contributing](rules/contributions.md)
