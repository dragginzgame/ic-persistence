# IC Logging Stable

Unpublished workspace scaffold for a durable adapter to
[`ic-logging`](https://github.com/dragginzgame/ic-observability/tree/main/crates/ic-logging),
owned by IC Persistence. It inherits the workspace version, Rust 1.91 minimum,
edition and license. It currently defines no public API, persisted format or
logging behavior and adds no dependencies.

## Ownership and upstream handoff

IC Observability owns the logging engine: input admission, sequence/time rules,
UTF-8 truncation, filtering, paging and retention decisions. Its restoration or
storage contract must expose sufficient validated state to resume the same log
without repeating sequence IDs or forgetting the last timestamp after pruning
every entry. Persistence must reuse that contract rather than implement a second
logging engine or access private Observability state.

IC Persistence will own bounded durable encoding, corruption admission, governed
Storage integration and reopen/upgrade qualification. Production handles must
come from the host's committed `ic-memory` runtime. Keep durable logging separate
from `ic-metrics-stable`; neither adapter should depend on the other.

The host retains allocation declarations, namespace identity, clocks, retention
scheduling, endpoints, authorization and lifecycle. Diagnostic history does not
establish that an external effect completed.

Before implementing append/reopen/prune/page operations, select the published
Observability handoff and its validity rules, define retained-byte and entry
bounds, and specify how pruning reuses storage. No stable row layout, codec,
cursor restoration or reset policy is frozen by this scaffold. The upstream
dependency will be added to the root catalog when that contract is ready.

## Development and publication

```sh
cargo check -p ic-logging-stable --locked --offline
```

The crate participates in workspace formatting and compilation. Release version
projection includes its local lock record, but `publish = false` keeps it out of
coordinated archive receipts and publication. Enabling publication requires the
implemented adapter, focused qualification and release-roster integration.
