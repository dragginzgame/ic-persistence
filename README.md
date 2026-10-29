# IC Persistence

An unpublished [`ic-logging-stable`](crates/ic-logging-stable/README.md) adapter
adds explicit bounded logging checkpoints alongside Metrics Stable. Its candidate
implementation requires Logging 0.17.2's validated restore contract; registry and
installed-runtime qualification remain pending. Heap Logging selects no stable
store automatically.

Allocation governance and governed stable collections for Internet Computer
canisters, maintained in one repository and published as three crates.

| Crate | Responsibility | Guide |
| --- | --- | --- |
| `ic-memory` | Allocation identities, admission, runtime bootstrap and exclusive leases | [Memory](crates/ic-memory/README.md) |
| `ic-storage` | Governed collection adapters over Memory leases | [Storage](crates/ic-storage/README.md) |
| `ic-metrics-stable` | Validated metric snapshots in governed Storage maps | [Metrics Stable](crates/ic-metrics-stable/README.md) |

Storage depends on Memory. There is no umbrella crate. Upstream
`ic-stable-structures` retains collection algorithms and persisted formats;
applications retain lifecycle and recovery policy. The stable metric adapter
owns its V1 records here; allocation-free arithmetic remains in IC Metrics.
See [ownership](docs/ownership.md) and [safety](SAFETY.md).

The root Cargo workspace owns all three packages, one dependency catalog and one
lockfile, with Rust 1.91 as the minimum. Run Make and Cargo commands here.
The independent [installed runtime runner](testing/runtime-qualification/README.md)
retains its host-only lockfile. Archive consumer qualification uses isolated,
generated workspaces so a workspace path dependency cannot mask missing package
contents or a second Memory identity.

Recovery mutation smoke cases run with the library tests. Run the longer,
reproducible campaign explicitly with
`cargo test --locked --offline -p ic-memory --lib recovery_mutation_campaign -- --ignored --nocapture --test-threads=1`.
Set `IC_MEMORY_FUZZ_SEED` to a decimal `u64` to replay another seed; each campaign
performs 10,000 mutations plus corpus truncation cases. This is bounded mutation
testing, not coverage-guided fuzzing.

[Setup](docs/host-support.md) · [Releasing](RELEASING.md) ·
[Documentation](docs/README.md) · [Migration map](docs/migration/ic-persistence.md) ·
[Current handoff](docs/status/current.md) · [Contributing](rules/contributions.md)
