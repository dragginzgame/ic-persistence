# ic-metrics-stable changelog

Independent releases begin after the coordinated 0.58.1 release. Earlier history
is retained once in the [shared historical ledger](../history.md#0581---2026-10-31).

## [0.64.0] - 2026-11-01

### ⚠️ Breaking

- Adopt Metrics 0.25.0 for public measurement and restoration-error types.
  Hosts exchanging these values with Metrics Stable must align their Metrics
  dependency. Native schemas now share Wire 0.14 with the adapter, removing the
  former Wire 0.13 dependency. Snapshot bytes, bounds, arithmetic and checkpoint
  continuity remain unchanged; no reset is needed. [Details](0.64.md).

### ⬆️ Dependencies

- Qualify snapshot encoding with locked Wire 0.14.1, preserving its Wire 0.14
  identity, stored rows and admission budgets.

## [0.63.0] - 2026-11-01

### ⚠️ Breaking

- Adopt Wire 0.14.0 for governed metric snapshots and public codec errors, with
  the matching core supplier identity. Hosts constructing or matching Wire
  payloads must align their dependency. Record bytes, bounds, arithmetic and
  checkpoint continuity remain unchanged. Metrics' native schema support retains
  its own required Wire 0.13 lane until the supplier adopts Wire 0.14.

## [0.62.1] - 2026-11-01

### ⬆️ Dependencies

- Qualify governed metric snapshots with locked Wire 0.13.5. Record bytes,
  budgets, arithmetic admission and checkpoint continuity remain unchanged.
- Adopt the core supplier's Platform 0.17.0 boundary, preserving public metric
  identities, stored records and validation.

## [0.62.0] - 2026-11-01

### ⚠️ Breaking

- Align the next core supplier identity for typed ledger decode errors. Metric
  records, arithmetic admission and checkpoint continuity remain unchanged.
- Hosts must pass parsed stable keys and explicitly select Storage creation,
  retained loading or deliberate reset. Valid checkpoint bytes remain unchanged.
  [Migration](../../docs/migration/0.61-public-api.md).

### ⬆️ Dependencies

- Require published Metrics 0.24.5, preserving current checkpoint bytes and
  canonical state admission.

## [0.61.5] - 2026-11-01

### ⬆️ Dependencies

- Require Wire 0.13.3 for governed metric snapshots. Existing record bytes,
  limits, arithmetic validation and checkpoint continuity remain unchanged.

## [0.61.4] - 2026-11-01

### 🔧 Changed

- Qualify the governed snapshot adapter with Platform 0.16.0. Public metric
  identities, record bytes, arithmetic admission and checkpoint continuity are unchanged.
  Retain the selected Metrics 0.24.4 patch and consume the compatible core supplier.

## [0.61.3] - 2026-11-01

### 🧹 Cleanup

- Iterate finite and overflow histogram buckets together for continuity checks,
  preserving exact-addition and sticky-overflow admission without sentinel indexing.

### ⬆️ Dependencies

- Retain the selected Metrics 0.24.3 patch and consume the compatible core
  0.60.5 supplier; existing public behavior and durable formats are unchanged.

## [0.61.2] - 2026-11-01

### ⬆️ Dependencies

- Adopt the compatible core 0.60.4 supplier and require Wire 0.13.2 for durable metric snapshots. Public types,
  encoded rows, bounds and restoration remain unchanged; no data reset is needed.

## [0.61.1] - 2026-10-31

### ⬆️ Dependencies

- Adopt Metrics 0.24.1 and the compatible Platform 0.15.3 core supplier.
  Numeric behavior, Wire 0.13 metadata and durable rows are unchanged.

## [0.61.0] - 2026-10-31

### ⚠️ Breaking

- Adopt Metrics 0.24.0 so native metadata shares Wire 0.13 with the durable
  codec. Hosts exchanging arithmetic values must align their Metrics dependency.
  Arithmetic, encoded rows and restoration rules are unchanged; existing current
  rows need no reset or migration. The older native Wire provider is removed.

## [0.60.0] - 2026-10-31

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
