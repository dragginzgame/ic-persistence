# Current handoff

Delivery of the pending Shared Tooling and Metrics adoptions is authorized.
Release/publish Memory and Storage 0.60.1 first, followed by Metrics Stable 0.61.0.
Logging Stable remains 0.60.0. Evidence and temporary files for this delivery stay
under `target/delivery/shared-metrics024/`; each group uses the standard full gate,
its own retained receipts/archives, GitHub annotated tag and crates.io publication.

The working tree on released `aa35b60` now selects published Metrics 0.24.0.
Metrics Stable preserves default native metadata and shares Wire 0.13 with its
durable codec. The old native-only Wire/Derive 0.12 copies leave the root lock.
Only Metrics’ identity changes and those two old packages are removed; every
other selected package identity/checksum is preserved. The independent runtime
workspace has no Metrics dependency and its lock remains unchanged.

Metrics Stable has 0.61.0 pending notes for its exchanged arithmetic types.
Hosts must align their Metrics requirement. Arithmetic, serialized current ICMW
rows and restoration rules are unchanged; no reset or migration is required for
this dependency adoption. The package guide distinguishes this from the earlier
host-owned retirement of older formats. All four package versions remain 0.60.0;
delivery is now authorized to crates.io and GitHub. Release core 0.60.1 first,
then Metrics Stable 0.61.0; Logging Stable remains 0.60.0.

Evidence under `target/adoption/metrics024/` passes: 29 library tests, one
integration test, two doctests, strict affected Clippy, Rust 1.91, Wasm tests
compilation, production graph/pins, formatting and links. A disposable native
consumer compiles all four metric families against Wire 0.13’s schema trait,
retaining selected external identities. Fixed-row and malformed-state tests
qualify stored-byte/restoration preservation. Full release and installed runtime
matrices are not rerun for this dependency-only batch.

Pending Shared adoption is preserved at commit
`143a9abb9ae81feae9400ff3844b03b1ddfdd75a`, carrying the 0.27.22 rules while its
committed VERSION remains 0.27.21. Canonical export/source verification retains
78 roots and 132 payloads. The complete root member catalog uses contiguous group
blocks, group-owned aliases and separate changelog comments. Lint levels are
unchanged. Earlier Shared evidence remains under `target/adoption/shared02720/`,
`target/adoption/shared02721/` and `target/adoption/shared02722/`. The comment-only
0.27.22 stage’s metadata equality record predates this deliberate Metrics change.
Core’s pending tooling patch remains 0.60.1. No cleanup is performed; explicit
Cargo cleanup still owns the whole workspace target, including retained evidence.

The previous delivery remains complete: all four crates 0.60.0 are on crates.io
with matching qualified checksums and three GitHub Releases. Delivery evidence
remains under `target/delivery/wire013-tooling075/`. Wire 0.13.0, Testkit Native/CLI
0.75.0, Build/Host/editor 0.73.1 and PocketIC 16.1.0 remain selected. Library
Platform 0.15.2, Types 0.15.1 and Logging 0.23.2 remain unchanged; the runtime
workspace retains Platform 0.15.1. Original installed proof stays at
`target/qualification/runtime/attempt.6EwNrA/` without relabelling it as new work.
