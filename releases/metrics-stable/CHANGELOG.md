# ic-metrics-stable changelog

Independent releases begin after the coordinated 0.58.1 release. Earlier history
is retained once in the [shared historical ledger](../history.md#0581---2026-10-31).

## [0.60.0] - Unreleased

### ⚠️ Breaking

- Adopt IC Wire 0.13.0 for durable codecs. Consumers exchanging exposed Wire
  errors must align that dependency identity. Encoded bytes, bounds and admission
  remain unchanged; no retained-data reset or migration is required.

## [0.59.1] - 2026-10-31

### 🐛 Fixed

- Qualify the selected group's exact publication archive after workspace-wide
  integration packaging. Published core dependencies retain their registry
  checksums in the adapter's packaged lockfile. Metrics 0.59.0's post-upload
  refusal exposed regenerated local supplier checksums; source and manifests
  matched, but its archive differed from the retained qualification. Preserve
  that release and evidence; this patch binds qualification to publication bytes.

## [0.59.0] - 2026-10-31

### ⚠️ Breaking

Adopt Wire 0.12.1 and Metrics 0.23.2. Consumers exchanging
exposed Wire errors or Metrics values must align those dependency identities.
Stored bytes, bounds and checkpoint admission are unchanged; no reset or retained
data migration is required.

### ⬆️ Dependencies

Own this adapter's version and release independently of Memory/Storage and the
other stable adapter ([#144 — Separate core persistence and stable adapters into independent release groups](https://github.com/dragginzgame/ic-persistence/issues/144)).
Numeric versions retain the last coordinated 0.58.1 identity until a separately
authorized release. Core requirements change only when compatibility requires it.

Metrics' default-enabled `native-wire` feature preserves native schema support
and now shares Wire 0.12 with this adapter. Disabling Metrics defaults requires
explicit `native-wire` when metadata is needed. Wasm arithmetic/Serde and durable
snapshot formats are unchanged; the older Wire metadata provider is removed. The upstream release-group adoption
preserves arithmetic, logging semantics and encoded bytes.

The Shared symmetric-layout adoption places this canonical ledger under
`releases/` and requires explicit group selection for release, resume and
publication. Literal package versions retain 0.58.1; the former crate ledger is
only a navigation pointer. Earlier shared history is preserved once, not copied.

Adopt Metrics 0.23.2, preserving its numeric and transport contracts. Shared
Tooling 0.27.19 records delegated delivery without changing release gates.
