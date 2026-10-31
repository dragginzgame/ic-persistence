# IC Persistence tooling adoption and qualification boundaries

Read this before changing the shared snapshot, tool setup, release adapters,
formatting or evidence retention. [AGENTS.md](../AGENTS.md) owns the local overlay;
[.shared-tooling.snapshot](../.shared-tooling.snapshot) owns the exact shared
revision, selected roots, payload hashes and executable modes. The former
live-policy exception is retired: dirty sibling rules never override the reviewed
local baseline. Future policy changes require reviewed snapshot adoption.

## Current source selection

IC Persistence selects committed Shared Tooling 0.27.11 at
`62edfbd43087a718bf0283e9040676a49211976d`, with 77 explicit roots and 127 payloads.
The canonical impact preview and refresh retain the reviewed selection. Source
verification compares every payload and mode with that independent commit.

The adopted graph observer and captured validation-input checkpoint helper retain
source-drift refusal and failure evidence. The new Cargo publication adapter is
opt-in; this repository retains its coordinated four-package receipt/publication
contract. Example support now uses ordinary module discovery. Source-level lint
exceptions use scoped expectations; the existing Cargo-table style policy was
reviewed and its configuration-dependent ownership/documentation/naming reasons
are recorded beside the retained levels. No lint level, host/IC tool catalog,
formatter pin or maintenance activation changed.

Platform 0.12.0 is selected directly in both maintained graphs, with the same qualified
stable-structures 0.7.2 and PocketIC 16.1.0 substrate/client identities. Production
Wasm has one Platform runtime owner and no external IC0/executor supplier. Tooling's
native-only test meter retains its required non-runtime Platform 0.11.1 provider;
it does not enter production Wasm. The optional Platform gzip facade is disabled.
Metrics and Logging select published 0.21.0. Metrics durable bytes stay unchanged;
Logging's explicit hard-cut disposition is documented in its package guide.
Both maintained catalogs select Wire 0.10.0; Metrics now shares its native metadata
identity. The older Wire copies are removed. Production Wasm has one core Wire
provider and no Wire endpoint or platform-types integration. IC Tooling libraries
and separately prepared Build/Testkit executables select 0.72.1; the reviewed
server stays 16.1.0. Shared Tooling and common tool catalogs are unchanged.

The library [dependency catalog](../Cargo.toml) owns direct package requirements
and feature selections; its [lockfile](../Cargo.lock) owns resolved identities.
The independent qualification [catalog](../testing/runtime-qualification/Cargo.toml)
and [lockfile](../testing/runtime-qualification/Cargo.lock) retain their separate
host graph. The [Build editor pin](../ci/release-tools.env) and
[Testkit CLI pin](../ci/runtime-tools.env) select executables independently.
Read those owners for versions rather than copying their inventories here.

The production graph admits one Platform provider and the exact qualified
substrate declared in [the local overlay](../AGENTS.md). Library admission,
executable setup/check and installed qualification remain separate authorities.
Selection alone does not establish acceptance or permit relabelling older proof.
The [release ledger](../CHANGELOG.md) and [current handoff](status/current.md)
record completed changes and scoped work without becoming release gates. A
dependency update activates neither maintenance schedules nor a snapshot refresh.

### Earlier adoption and qualification records

The compact baseline derives `rules/engineering-details.md` as a companion;
specialist contracts remain binding when the task touches them. Reporting metrics
are opt-in, and focused checks follow the completed coherent edit batch. The
adopted npm Platform boundary adds no dependency to this Rust-only consumer.
The refresh adds the canonical Rust formatting driver and updates Make dispatch,
preserving common tool pins and the installer build-policy identity.
The preceding consumer-owned Build/Testkit selections were 0.69.6. Both dependency catalogs
select Platform 0.7.3 without enabling optional provider/gzip features.
The stable-structures 0.7.2 substrate and PocketIC 16.1.0 pair remain selected.
Release 0.50.7 passed the complete native, MSRV and installed-runtime matrix
on Linux and both macOS architectures; snapshot integrity alone is not acceptance.
See [the adoption issue](https://github.com/dragginzgame/ic-persistence/issues/127).
The canonical takeover exporter retires the imported Storage snapshot after
verifying its original bytes. Storage PR #18's release retry protection is
therefore included; uncommitted Shared sibling changes are excluded.
The canonical [preview and refresh procedure](consuming-snapshots.md#preview-before-refreshing)
exports committed bytes from a clean source checkout. Direct callers remain
explicit roots and required companions are derived by the exporter.

The Platform 0.7.3 / Tooling 0.69.6 refresh preserves the incoming library lock
selection and advances only Platform in the independent lock. The selected
Platform interfaces are unchanged; native Tooling adds its portable source-mode
fix and reuses existing observation/process helpers. Build and Testkit CLI
selections advance separately to 0.69.6. Shared source and its executable closure
remain at the reviewed 0.27.3 revision, with no dirty upstream rules adopted.

The 0.27.3 refresh retains that closure and verifies every payload's bytes and
executable mode against the independently selected source commit, using the
trusted source verifier's paired `--source` and `--revision` options. Ordinary
builds retain offline snapshot checks. Consumer hook fixtures now use the shared
closure copier and stage all copied regular-file changes, including companions
absent from HEAD, while leaving opaque gitlinks intact.

The optional sccache pin does not change the common tool roster or enable a
compiler cache. Scheduled maintenance remains inactive unless explicitly
configured and qualified under the adopted report-only policy. The supplying
owner tracks shared dependency defects; no waiting issue is created per consumer.

The release prerequisite inventory remains consumer-owned: native system/compiler
bootstrap precedes common Cargo tools and the selected Build editor; both Rust
compilers and the Wasm target, host/IC tools and locked caches are admitted before
validation. The separate installed-runtime path prepares Testkit and its selected
server before qualification. Existing Make dispatch fixtures exercise ordering
and failure stopping; they do not replace actual native or installed acceptance.

The current refresh uses a clean detached committed source and canonical impact
preview. The shared Rust driver now owns the complete formatting sequence. The
Makefile's `FORMAT_RUST_ARGS` selects `testing/runtime-qualification` as the extra
workspace and `testing/package-consumer.rs` as an edition-2024 standalone source.
The paired overrides and copied adapter are retired. Root sorting/formatting runs
before extra-workspace sorting/formatting, followed by standalone Rustfmt, with
each pass visible and timed. Working-tree writes remain nontransactional; check
mode is non-mutating and staged-hook publication waits for the complete formatter
to succeed.

Standalone selections always run uncached, including workspace passes, so source,
module and configuration changes cannot be hidden by an incomplete result receipt.
Prepared formatter admission still runs first. Consumer fixtures exercise the real
new driver, source/config changes, exact ordering, jobserver descriptor forwarding,
failed standalone admission before workspace passes and retained failure logs.
The hook fixture's exported-index closure includes the driver and preserves
partial staging and unrelated work. This implements the local migration for
[#135 — Adopt shared formatter orchestration without dropping package-consumer formatting](https://github.com/dragginzgame/ic-persistence/issues/135),
using the capability delivered for
[Shared #314 — Support standalone Rust sources in shared formatting orchestration](https://github.com/dragginzgame/shared-tooling/issues/314).
Remote issue closure and delivery remain separate from these working-tree changes.

Shared 0.26.26 replaces broad module-path screening with the shared Rust lexer's
balanced-attribute inspection for workspace-only cache admission. The driver and
`RustSource.pm` are both source-bound; the hook fixture copies/stages the module,
and the jobserver fixture follows and compares that companion closure. Standalone
selection still disables all reuse, so Persistence's active formatting behavior
stays uncached. Shared 0.26.25's scanner filename qualification changes are outside
this consumer's selected executable closure. Tool pins and build identity remain
unchanged.

The refreshed hook accepts opaque tracked gitlinks, preserving their staged
identities while formatting selected regular files. This repository has no
production submodule integration to migrate. The Node guide is a required policy
companion, not Node activation. The Clap-based runtime scanner is not selected by
this snapshot and requires no consumer compiler package or dependency changes.

The gzip policy names Tooling's opt-in encoder as the application/fixture owner;
Cargo archives and system tar used for packaging/evidence retain their explicit
infrastructure boundary. No gzip dependency/feature or product encoding changes.

The 0.26.22 refresh uses a clean detached copy of the committed source and the
canonical impact preview, preserving the existing roots and payload closure.
Shared 0.26.21 makes emoji categories the current/future root changelog format;
Persistence already uses it, and older release entries are preserved. Shared
0.26.22 explicitly prohibits owned `CandidType` models and strengthens existing
native error, machine-output, bounded-process, terminal-rendering and source-scan
review rules. Persistence's maintained Rust sources contain no `CandidType`
derives or implementations. These are reviewed governance changes, not new
executable scanners or a completed repository-wide hygiene audit. Existing
caller qualification and negative tests retain their owning gates.

The refresh uses the canonical exporter from a clean detached copy of that
commit and retains the existing capabilities. The companion-copy and Make-adapter helpers become explicit roots because the
consumer release fixture calls them for formatter closure and parallel lint-order
qualification, keeping its target roster and effect stubs local. Explicit Cargo-tool
setup waits up to 60 seconds for an admitted exact-slot owner;
`SHARED_TOOLING_SETUP_WAIT_SECONDS=0..300` selects another canonical integer
budget. Offline checks/preflight and Host/IC setup remain immediate. No foreign
lock is removed and no owner is killed. After waiting, compiler/source/lock and
completed receipts are admitted again before reuse or activation.

The merged [Shared PR #302](https://github.com/dragginzgame/shared-tooling/pull/302)
owner-cleanup fix remains selected. Cache initialization now uses idempotent
private directory creation with existing path/configuration admission, without a
global source-preparation lock. Cargo owns download serialization; exact selected
tool slots still serialize builds. Old source-preparation slots and generations
remain retained.

The earlier snapshot selected 0.26.11 timing work while its committed `VERSION`
still declared 0.26.10. The previous refresh recorded the producer's committed
0.26.19 identity; sibling working-tree edits remain excluded.
The refresh retains common tool pins, compiler selections and the tool-store
build-policy identity. Existing prepared tools can be admitted without rebuilding.
The retired checkout-local Rust build directory no longer affects admission;
current shared-store checks and retained evidence remain in force.

Cargo admission now preserves exit status 75 for an admitted busy tool slot.
Plain checks remain immediate; a direct caller may explicitly select
`--check --wait-seconds N` for a bounded read-only wait. The runtime-server Make
adapter preserves the busy diagnostic without recommending installation. Missing
or invalid preparation retains its setup advice. No automatic wait or installer
retry is added to normal checks or helper recovery.

Normal release commands now start fresh from current clean committed metadata,
without scanning old journals or replaying their effects. Recovery requires
`release-resume VERSION=X.Y.Z`; the selected source, payload, refs, locks and
destination remain admitted. This replaces the earlier automatic recovery rule.
The runner may discard only its new candidate's stale plan/staging/PR metadata;
consumer validation receipts, failed gate logs and retained archives preserve
their existing custody. No release is executed during adoption.

Shared 0.26.17 diagnoses unsupported retention-probe prefixes/selections and
avoids partial generated probes. The canonical helper supports column-one,
single-line EXIT traps; multiline consumer actions are not silently reinterpreted.
Shared 0.26.19 allows archived Draft labels beneath known historical release
sections while refusing active drafts and conflicting current candidates.
The source commit message says 0.27.19, but its committed VERSION and exported
snapshot declare 0.26.19; adoption records the committed declaration.

The RPC-budget rule is adopted as governance; Persistence owns no generated
cross-runtime application RPC client, so no product budget or codec is added.

`make tools-report` runs every existing offline checker and retains individual
failures and durations. It is diagnostic; fail-fast `tools-check` remains release
admission. The validation-target runner is selected only as its required companion,
without changing this consumer's validation/release ordering. Installer `--describe`
reports admitted selection inputs as JSON without executing or preparing tools.
It does not replace installed-tool checks. Explicit Cargo setup now reports source
preparation, lock waiting, selection/cache reuse, compilation and receipt/build
admission with start/outcome/duration on stderr, delivering
[Shared #303 — Report Cargo tool setup phases and durations](https://github.com/dragginzgame/shared-tooling/issues/303).
Checks remain quiet and offline; stdout remains the selected executable path.
No owner is killed, no deadline extended and no build identity changed.

Formatting now uses the same literal catalog parser as installation, rejecting
duplicate assignments and executable configuration. Both local workspace adapters
remain selected. `check-storage-boundary` opts into the shared external IC package
inventory while preserving this consumer's stricter no-direct-substrate rule.
The existing production graph policy remains local; structured graph explanations
are available for diagnosis. The existing consumer lint-order fixture uses the shared parallel-Make probe;
its expected stages and effect stubs remain local. Optional common CI cache setup
does not replace this consumer's current cache bindings.

Ordinary expensive hosted integration follows Shared's asynchronous cadence.
Focused checks, enforced GitHub requirements and separate release/publication/
deployment and issue-specific acceptance remain in force. No CI dispatch or
branch-protection change is part of adoption. DFINITY work stops at the owned
Platform boundary and does not depend on future DFINITY contributions.

Validation runs the existing all-target Clippy gate after prerequisite admission
and before fixture suites, tests, Wasm builds and packaging. Sequential recipes
preserve that ordering under parallel Make; the feature-specific fixture lint
also remains in its focused target. Release notes may use a numbered
` - Unreleased` heading, which the shared finalizer dates during preparation.

Shared 0.25.3 binds explicit Cargo feature sets to selected tool installations
and admission receipts. Build and Testkit 0.69.6 binaries select `--features cli`
in setup, offline admission and bootstrap calls. Library dependencies retain
disabled defaults. Prepare the selections with `make install-tools` and
`make install-runtime-server`; previous installations and failed attempts remain
retained, and ordinary checks never repair tools.

Memory's failure collector delegates tool selection and archiving to Shared's
`collect-tool-evidence.sh`. Failed selector output remains diagnostic data;
it cannot admit physical payloads. Memory retains product log selection,
redirected-parent admission and hosted upload/download verification.

The retired Perl manifest/lock renderers leave the snapshot. Repo-tool calls
published IC Build's pure rendering API and retains its independent semantic
projection, saved release intent, receipts and rollback. Changelog finalization
still uses Shared's canonical helper with retained bounded inputs. Interrupted
manifest/lock bootstrap uses the published editor selected in
[`ci/release-tools.env`](../ci/release-tools.env), with regular source copies and
no source symlinks. `make install-release-tools` explicitly prepares it;
`make release-tools-check` admits it offline. The common setup/check aggregates
include that tool. Standard release bootstrap can prepare the selected editor;
ordinary helper recovery only checks it. Neither path changes dependency
selection or installs a Rust compiler.

Earlier adoption decisions and exact qualification results remain in the
numbered changelogs, [release-workflow evidence](release-workflow-qualification.md)
and Git history. Their former setup exceptions or tool selections are historical
evidence, not another current source of policy.

## Ownership and consumer qualification

See [the implementation ownership boundary](ownership.md) for runtime, collection
and host-tooling owners. Shared Tooling owns the exported runner, installers,
formatter prerequisites, Make modules, archive primitives and their generic
regression matrices. Memory owns which capabilities it selects and the following
consumer proofs:

| Retained proof | Local caller |
| --- | --- |
| Snapshot hashes, modes and selected roots | `make shared-tooling-check` |
| Actual catalogs, installed tools and native reuse | `make tools-check`, formatter admission and native CI |
| Two-workspace pin policy and exceptions | `make check-pins` |
| Default production Wasm normal/build dependency exclusions | `make check-production-graph` |
| Release source, receipts, rollback and Make dispatch | repo-tool tests and `make test-release-adapters`, with substituted Git/Cargo effects |
| Actual index formatting and failure isolation | `make test-hooks` |
| Archive selections, hosted transport bindings and retained failures | `make test-tools` and `make test-failure-evidence` |

Fleet reporters, exporter regression fixtures and generic runner, parser,
installer, Make, formatter, LOC and checksum matrices are not consumer tests.
Run them at Shared Tooling's owner when qualifying its implementation. No local
product proof, native host or installed-tool admission is removed by that split.

The production graph check uses Shared's canonical observer with Memory-owned
exclusions for native tooling and test dependencies. It checks the default
`wasm32-unknown-unknown` library graph; it does not replace compilation, artifact
size budgets or installed qualification. `test-support` adds no dependencies.

## Tool setup and formatting

The shared catalog owns common host/Cargo pins and the IC tool catalog. Memory
selects its validation compiler, library MSRV, independent qualification workspace
in its own manifests and Makefile, and Testkit CLI in `ci/runtime-tools.env`.
Follow [host support](host-support.md) for prerequisites and matching native evidence.
Source adoption, explicit setup, local activation and native qualification are
separate; an exported script or configured CI job is not acceptance evidence.

Memory selects only `wasm-opt` for installed runtime qualification, including
native offline reuse and compact failure evidence. The full canonical catalog
remains admitted; unselected tools are not downloaded or required. Full evidence
retention and prior installations remain unchanged.
Memory extends the common ordered setup/check targets with `install-ic-tools`
and `ic-tools-check`, and exports `.tools/ic/bin`. Common setup supplies
`.tools/host/bin` and `.tools/rust/common/bin`. Prepared bundles, failed attempts
and old selections remain retained; refreshing source never silently installs
or deletes tools. Fleet reports run in Shared Tooling; local `make cloc` selects
the root workspace, or an explicit `CLOC_MANIFEST` for the independent graph.

Formatting covers the root workspace, `testing/runtime-qualification` and the
standalone edition-2024 package consumer through declarative `FORMAT_RUST_ARGS`.
Shared owns each timed pass; the consumer owns the roster and selected compiler.
Standalone selections deliberately run without result reuse. Formatting builds,
fetches, installs and selected-lock changes remain prohibited. Hook setup is
explicit and real index effects stay in disposable fixtures or explicitly
authorized commits.

Testkit owns PocketIC selection, provisioning, offline admission and managed
lifecycle. The separately selected CLI is outside both maintained lockfiles;
`install-runtime-server` prepares it explicitly, `runtime-server-check` is offline
and `test-runtime` uses prepared tools and both caches. Installed qualification
retains its separate authority. No local server installer or supervisor competes
with Testkit. See the [qualification runner](../testing/runtime-qualification/README.md).

## Release and evidence boundaries

The shared runner owns ordering, locks, saved intent and Git effects.
Memory selects direct delivery and owns its Rust source/package receipts,
metadata transaction and proof admission; IC Build supplies the available Cargo
observers and projection checks through explicit native development dependencies.
The unused alternate release delivery and validation checkpoint paths are not
selected as Memory workflows. See [RELEASING.md](../RELEASING.md).

Normal checks remain offline. Authorized release entry points separately allow
locked cache preparation before helper compilation; an explicitly offline caller
stays offline. Never run a release command, create a fixture commit or alter the
real index merely to qualify adoption.

Memory selects its product logs and archive contents, while shared primitives
own tool selection, checksums and archive construction. Full retention is the
default; CI requests compact retention only for freshly admitted complete active
sets. Failed, unknown and unselected bundles stay full. The workflow binds the
exact upload ID/digest, repository/run and token. Preserve unsuccessful collection
observations, gate logs, prior receipts and package archives across retries.

## Stable-memory governance policy

Production stores consume checked Memory handles after admission and durable
allocation commitment. [AGENTS.md](../AGENTS.md#stable-memory-governance) and
[SAFETY.md](../SAFETY.md) record this boundary. The common rule was adopted with
Shared 0.20.0; policy adoption alone does not qualify consumer integrations.

IC Storage owns governed collections consuming `RuntimeMemory`; its generic
substrate stays private. Memory's exclusive leases and narrow backing exports do
not require a dependency on Storage. Hosts still own exclusive backing custody;
Rust import visibility cannot detect aliases or enforce application policy.
