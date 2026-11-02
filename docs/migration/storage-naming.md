# Storage naming adoption

The repository is renamed from `dragginzgame/ic-persistence` to
`dragginzgame/ic-storage`. The core package stays `ic-storage`. The adapters move
to new crates.io identities; no forwarding crate or Cargo alias is provided.

| Previous | Current | Rust import |
| --- | --- | --- |
| `ic-storage` | `ic-storage` | `ic_storage` |
| `ic-metrics-stable` | `ic-metrics-storage` | `ic_metrics_storage` |
| `ic-logging-stable` | `ic-logging-storage` | `ic_logging_storage` |

Update workspace catalogs, member declarations and Rust imports together. Align
all users with the same core and adapter versions to preserve Rust type identity.
The external engines remain `ic-metrics` and `ic-logging`; their names do not
change. This source rename does not publish the new package names.

Release automation selects `RELEASE_GROUP=storage`, `metrics-storage` or
`logging-storage`. Current source lives under the matching `crates/ic-*` paths;
pending ledgers and detailed notes live under `releases/<group>/`.

Published histories remain in `releases/persistence/`, `releases/metrics-stable/`
and `releases/logging-stable/`, with their original versions and package names.
Previously saved release attempts and receipts keep their original source/group
identity and must not be resumed under the new policy. Numeric manifest versions
are unchanged by the rename; pending minor releases include the existing hard cut.

The rename itself adds no stored-format change. The combined pending Storage
release still requires fresh/reset whole backing for the owned allocator and
collection formats. See [the allocation contract](../../crates/ic-storage/ALLOCATION.md).

Update existing remotes to `https://github.com/dragginzgame/ic-storage.git`.
The active local checkout directory may retain its previous name; directory names
are not Cargo package or release identities.
