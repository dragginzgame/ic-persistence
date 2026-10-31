# ic-logging-stable changelog

Independent releases begin after the coordinated 0.58.1 release. Earlier history
is retained once in the [shared historical ledger](../history.md#0581---2026-10-31).

## [0.60.1] - Unreleased

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
