# ic-logging-stable changelog

Independent releases begin after the coordinated 0.58.1 release. Earlier history
is retained once in the [shared historical ledger](../history.md#0581---2026-10-31).

## [0.60.4] - 2026-11-01

### 🔧 Changed

- Qualify the governed logging adapter and canister fixture with Platform 0.16.0.
  Checkpoint bytes, limits, retained-data handling and restoration are unchanged.
  Retain the selected Logging 0.23.5 patch and consume the compatible core supplier.

## [0.60.3] - 2026-11-01

### 🧹 Cleanup

- Validate checkpoint field limits and accumulate retained text in one traversal,
  preserving error precedence, encoded bytes and canonical restoration.

### ⬆️ Dependencies

- Retain the selected Logging 0.23.4 patch and consume the compatible core
  0.60.5 supplier; existing public behavior and durable formats are unchanged.

## [0.60.2] - 2026-11-01

### ⬆️ Dependencies

- Adopt the compatible core 0.60.4 supplier and require Wire 0.13.2 for bounded logging checkpoints. Checkpoint
  bytes, admission and restoration remain unchanged; no data reset is needed.

## [0.60.1] - 2026-10-31

### ⬆️ Dependencies

- Adopt Logging 0.23.3 and the compatible Platform 0.15.3 core supplier.
  Logging semantics, checkpoint bytes and restoration remain unchanged.

## [0.60.0] - 2026-10-31

### ⚠️ Breaking

- Adopt IC Wire 0.13.0 for durable codecs. Consumers exchanging exposed Wire
  errors must align that dependency identity. Encoded bytes, bounds and admission
  remain unchanged; no retained-data reset or migration is required.

## [0.59.0] - 2026-10-31

### ⚠️ Breaking

Adopt Wire 0.12.1 and Logging 0.23.1. Consumers exchanging
exposed Wire errors must align those dependency identities.
Stored bytes, bounds and checkpoint admission are unchanged; no reset or retained
data migration is required.

### ⬆️ Dependencies

Own this adapter's version and release independently of Memory/Storage and the
other stable adapter ([#144 — Separate core persistence and stable adapters into independent release groups](https://github.com/dragginzgame/ic-persistence/issues/144)).
Numeric versions retain the last coordinated 0.58.1 identity until a separately
authorized release. Core requirements change only when compatibility requires it.

Logging constructs retained messages from already-admitted parts. Text accounting,
truncation, eviction, sequence watermarks and checkpoint restoration are unchanged.
The existing discriminator-3 text-budget contract and host retirement policy remain.

The Shared symmetric-layout adoption places this canonical ledger under
`releases/` and requires explicit group selection for release, resume and
publication. Literal package versions retain 0.58.1; the former crate ledger is
only a navigation pointer. Earlier shared history is preserved once, not copied.
