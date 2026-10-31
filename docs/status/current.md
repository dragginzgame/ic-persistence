# Current handoff

Delivery of the pending upstream refresh, issue #144 release groups and symmetric
Cargo/changelog layout is authorized to crates.io and GitHub. Metrics now selects
0.23.2; Logging remains 0.23.1. Shared selects committed 0.27.19 at
`54fc49e12f439dbe18d232f622a6d78e61773353`. The canonical exporter preserves
132 payloads and 78 selected roots. Latest evidence is retained under
`/tmp/ic-persistence-delivery21/`.

All three groups have 0.59.0 pending notes for the previously adopted Wire/Metrics
identity changes. Delivery runs supplier-first: persistence (Memory/Storage),
metrics-stable, logging-stable. Group version sources, ledgers, tags and receipts
are explicit; each release runs the complete configured gate and publishes its
qualified archive set. Root history remains preserved once.

Earlier evidence remains under `/tmp/ic-persistence-refresh19/`,
`/tmp/ic-persistence-groups144/` and `/tmp/ic-persistence-layout20/`. Installed
Platform/Wire/Logging qualification remains at
`target/qualification/runtime/attempt.6EwNrA/`; the Metrics patch changes only the
Metrics package identity in the library lock and does not change that independent
runtime graph. Metrics numeric and transport contracts are unchanged. Retain
all prior logs, receipts, archives and failed attempts.
