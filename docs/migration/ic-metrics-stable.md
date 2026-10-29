# IC Metrics Stable import

IC Persistence now owns `crates/ic-metrics-stable`, imported from IC Metrics
[`92ff804125fb07370fad4cb24cc620f52122ac36`](https://github.com/dragginzgame/ic-metrics/tree/92ff804125fb07370fad4cb24cc620f52122ac36/crates/ic-metrics-stable).
The complete Rust implementation, native regression tests and original MIT
license are copied unchanged. The original repository retains its Git history,
release notes and frozen measurement receipts with their original identities.
They are historical evidence, not qualification of the relocated graph.

## Ownership and compatibility

- IC Metrics owns allocation-free `no_std` arithmetic and releases
  `ic-metrics` independently on its 0.17 line, with standard Serde and native Wire
  support. It has no persistence dependency.
- IC Persistence owns the adapter's V1 record codec, numeric/continuity admission,
  governed map integration, tests, packaging and publication. Memory owns allocation
  governance; Storage owns the public collection and its retained map format.
- Hosts retain allocation keys, bootstrap, attribution, synchronization, lifecycle,
  write cadence and resets. No hooks, singleton, allocation manager or new storage
  format is introduced by this import.

The crate name, Rust module paths, API, row keys, V1 bytes and B-tree format are
unchanged. The package now inherits Persistence's 0.50 workspace version and uses
its local Memory and Storage packages. The 0.50.17 import used published
`ic-metrics = "0.15"`; Persistence 0.51 adopted `ic-metrics = "0.16"`, and the
pending 0.54 release adopts `ic-metrics = "0.17"`.
Select the adapter, Memory and Storage from the same Persistence release line.
Hosts using the adapter's public arithmetic types must select the same Metrics
minor line: each pre-1.0 Metrics minor has a different Rust type identity. Updating these
requirements is a manifest migration, not a retained-data conversion or a reason
to reset storage.
No maintained sibling consumer was found in the reviewed workspace manifests.

## Cutover and verification

Integrate and publish the Persistence addition before retiring adapter publication
from IC Metrics. The source retirement is a separate linked PR. Existing published
0.15 archives remain available; neither PR publishes or changes release versions.
Future consumers adopt an actually published Persistence version, not a pending
branch. No source path into a sibling checkout is required.

Workspace CI includes the imported native regression tests and README doctests,
strict native/Wasm Clippy, Rust 1.91 compilation and production graph admission.
Archive qualification extracts all three hashed packages and checks that the
consumer resolves exactly those Memory, Storage and Metrics Stable identities.
It saves and reopens a metric counter while retaining allocation lease checks,
on the development compiler and MSRV, and checks the same consumer for Wasm.
Release metadata/recovery fixtures cover all three inherited versions; publication
qualifies all archives and observes every registry record before any upload,
then publishes Memory, Storage and Metrics Stable in dependency order.

Run the focused checks recorded in the PR. A native reopen does not qualify live
IC upgrades, instruction costs, deployment, macOS or consumer attribution.
