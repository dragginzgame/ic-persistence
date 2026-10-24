# Implementation ownership

IC Persistence maintains three published crates. IC Memory owns durable allocation governance for one backing memory: permanent
key-to-ID bindings, host-wide placement, admission before commitment, protected
ledger recovery and exclusive checked handles. It does not own application data,
collection semantics, component lifecycle or deployment policy.

## Public entry points

The root [workspace catalog](../Cargo.toml) owns dependency and package selection;
[the library package](../crates/ic-memory/Cargo.toml) inherits it. All production
Rust exports are at [`ic_memory::...`](../crates/ic-memory/src/lib.rs); implementation
modules are private. There is no implicit library singleton or lifecycle hook.

| Capability | Canonical Memory implementation / interface |
| --- | --- |
| Stable keys, requests and host namespace grants/exclusions | `StableKey`, `MemoryRequest`, `SealedDeclarationSnapshot`, `MemoryAllocationPool` |
| Backing ownership and committed allocation leases | `MemoryRuntime`, `RuntimeMemory`, `MemoryManagerConfig` |
| Host application checks and recovered-store admission | `RuntimeBootstrapPolicy`, `PolicyIdentity`, `BootstrapAdmission` |
| Read-only allocation and recovery facts | runtime diagnostics and allocation reports |
| Passive DTOs, raw images, bounded fixture Cells and fixture bootstrap | feature-gated `ic_memory::testing` |

The host composes declarations and one pool, chooses policy and backing geometry,
bootstraps one runtime, then shares that runtime or transfers its opened handles.
Component verification checks its requirements against committed host authority;
it does not choose another pool, rerun host admission or construct another manager.
Exact warm host verification is a separate check of the complete bootstrap binding.
See [advanced integration](../ADVANCED.md) and [the composed host example](../crates/ic-memory/examples/composed_host.rs).

IC Storage owns the governed collection adapters in `crates/ic-storage/src/adapter`.
Its existing source and formats are unchanged by the repository consolidation.
Storage depends on Memory through the root path-plus-version catalog; Memory
does not depend on Storage.

IC Metrics Stable owns validated V1 metric records and restoration in
`crates/ic-metrics-stable`. It depends on Memory, Storage and the independently
published `ic-metrics` arithmetic crate. Hosts own sampling, attribution,
allocation keys, bootstrap, collection lifecycle, cadence and resets. Neither
Memory nor Storage depends on the adapter. See [the import guide](migration/ic-metrics-stable.md).

## Dependencies and external capabilities

| Owner | Boundary |
| --- | --- |
| `ic-platform::stable_structures` | Exact 0.7.2 manager/Cell substrate and backing `Memory` identities. Memory retains the checked layout adapters needed for bounded reads and fallible growth; product admission checks the transitive protocol identity. |
| `ic-storage::adapter` | Production typed collections over `RuntimeMemory`. Storage depends on Memory; Memory must not depend back on Storage or expose its raw collection substrate. |
| `ic-metrics-stable` | Metric identities, records and restoration over governed Storage collections. Memory supplies allocation facts, not a second metric store. |
| Hosts such as Canic; components such as IcyDB | Complete component composition, policy identity, application admission, lifecycle, database schemas and operator response assembly. |
| `ic-host-fs`, `ic-host-process`, `ic-host-artifacts` | Native publication, bounded processes, file observation and artifact hashing/copying used by repo-tool. These are native-only development dependencies. |
| `ic-build` | Available Cargo metadata/configuration observation and lock projection admission used by repo-tool. Memory retains release intent, command recipes, budgets and source/package custody. |
| Shared Tooling snapshot | Common engineering rules, setup, formatting, release orchestration and archive primitives. Consumer selection and integration checks stay local. |
| IC Platform / Testkit CLI | Platform supplies the optional native PocketIC client identities; Testkit owns explicit server preparation and lifecycle. The independent runtime runner consumes that client and supplied Wasm. |

The library's normal dependencies are the substrate, Serde, serde_cbor and
Thiserror. Host tooling does not enter the canister graph. The independent
[qualification workspace](../testing/runtime-qualification/Cargo.toml) deliberately
keeps its host-only PocketIC graph and lockfile outside normal library gates.
Sibling manifests and call sites establish these source boundaries; they do not
establish that every published consumer or deployed application has adopted them.

## Why these local implementations remain

- The bounded ledger codec, checksum and current fixtures define Memory's stored
  contract. A general codec is not automatically a substitute for its bounds and
  integrity checks. Changing this format requires a separate retained-data decision.
- Manager/Cell layout readers protect Memory's corruption and capacity boundaries
  against the exact substrate pin. The upstream constructors do not supply the
  same bounded, read-only or fallible behavior. Storage wrappers consume leases
  and therefore cannot own Memory's own ledger bootstrap without a dependency cycle.
- Fixture Cells exercise governed byte storage and caller-selected limits; they
  are not a general production collection API. Installed IO/upgrade tests qualify
  this crate's behavior rather than duplicating Testkit's server implementation.
- Native repo-tool owns all packages' release proofs and recovery transaction. Shared
  runner delegation does not replace those proofs. A pending upstream editor or
  receipt proposal is not an available compatible dependency.

The [adoption guide](shared-tooling-adoption.md) records selected tooling and
consumer qualification. Historical reports under `docs/changelog`, `docs/audits`
and the qualification guides retain their original source/version scope; use the
current API and safety contract for present integration.

## Release tooling review — 0.62.1

The release adapter already delegates manifest rendering and lock projection to
IC Build, durable receipt/archive writes to Host FS, digests to Host Artifacts,
and bounded capture to Host Process. Saved intent, package ordering, source-bound
receipts and retry admission remain Persistence policy. Removing those checks
would remove consumer proofs rather than eliminate a duplicate implementation.

IC Build’s `qualify_package_archives` also owns packaging and runs all-target
and doctest suites. It cannot replace the current extracted-package checks and
native/MSRV/Wasm consumer runner without changing that qualification roster.
A future adoption needs a composable archive-admission boundary that retains
source/archive custody while leaving consumer commands explicit. Keep the
current gate until that boundary is available and qualified.
