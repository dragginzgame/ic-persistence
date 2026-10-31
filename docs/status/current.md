# Current handoff

Delivery of Wire 0.13.0 and Testkit Native/CLI 0.75.0 is authorized to crates.io
and GitHub. Build/Host and the selected Build editor remain 0.73.1. Release all
three approved groups at 0.60.0 in supplier-first order: persistence,
metrics-stable, logging-stable. No stored bytes or admission changes are needed.
The incoming Platform/Types and Logging lock selections remain preserved.

Current delivery evidence is retained under `target/delivery/wire013-tooling075/`;
its temporary directories remain in this repository to avoid external `/tmp`
cleanup. Each group runs the configured full gate, retains its own receipts and
archives, and publishes separately from its qualified release checkout.

Metrics 0.23.2 retains native-only Wire 0.12 metadata by default. Persistence
codecs and production Wasm select Wire 0.13; this does not claim host metadata
schema alignment. Stable-structures 0.7.2 and PocketIC 16.1.0 remain unchanged.

The preceding adoption evidence remains under `target/adoption/wire-tooling22/`:
245 library tests, strict Clippy, Rust 1.91, Wasm checks, runtime workspace check,
graph/pins/formatting, all three group tests, exact file plans and six interrupted
metadata states passed. Earlier `/tmp` records removed by host cleanup are not
claimed as retained. Original installed runtime evidence remains at
`target/qualification/runtime/attempt.6EwNrA/`, without relabelling it as new proof.

Previous delivery published Memory/Storage 0.59.0, Metrics Stable 0.59.1 and
Logging Stable 0.59.0 with matching registry checksums and GitHub Releases. Metrics
Stable 0.59.0 remains published; 0.59.1 corrected qualification of packaged
supplier checksums. Preserve its original tag and receipts. Shared remains
committed 0.27.19 at `54fc49e12f439dbe18d232f622a6d78e61773353`.
