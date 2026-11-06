# Selected measurements and ownership

Experimental evidence follows the [shared retention contract](../../rules/engineering-details.md#changelogs-and-artifact-preservation):
one current summary and at most two small data files per topic, 100 KiB combined,
within the 5 MiB repository budget. These are limits, not storage targets.

| Topic | Summary | Selected data |
| --- | --- | --- |
| Storage/ICSS type matrix | [Current coverage/results](../audits/recurring/benchmark-full-catalogue-2026-11-05.md) | [565 selected rows](storage-comparison-selected.csv), [compact identities](storage-comparison-identities.json) |
| Consumer memory sizing | [Declaration inventory and historical controls](../local-memory-audit.md) | [Consumer declarations](consumer-memory-inventory.json), [selected historical controls](storage-memory-study.json) |
| Current engine reset | [Compatibility, safety and selected baseline](../audits/recurring/icss-engine-baseline-2026-11-05.md) | Inline selected table |
| Historical engine experiments | [Decisions and limits](../audits/recurring/storage-icss-hotspots-2026-11-04.md) | Inline selected table |
| IC stable-read charging | [Mechanism and selected calibration](../audits/recurring/vector-stable-read-charges-2026-11-05.md) | Inline selected tables |

The 2026-11-06 cleanup retires bulk CSV/JSON exports, repeated run metadata and
patches for removed collection engines. It does not create another permanent
archive. Existing target run directories are untouched and transient; hashes or
summaries alone cannot verify raw artifacts after explicit cargo clean removes
them. New comparison output uses `target/evidence/storage-comparison/`.

Small runtime/ledger/allocation/compiler/Wasm qualification tables remain linked
from their historical release/support records. The `canic162/` qualification
subdirectory, release receipts, active recovery state, source-provenance records,
tool caches and pending code are outside this experimental-export retirement.
Do not treat historical measurements as qualification of current collection
formats or an optimum for all applications.
