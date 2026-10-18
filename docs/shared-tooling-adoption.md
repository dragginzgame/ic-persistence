# Memory tooling adoption and qualification boundaries

Read this before changing the shared snapshot, tool setup, release adapters,
formatting or evidence retention. [AGENTS.md](../AGENTS.md) owns the local overlay;
[.shared-tooling.snapshot](../.shared-tooling.snapshot) owns the exact shared
revision, selected roots, payload hashes and executable modes. The former
live-policy exception is retired: dirty sibling rules never override the reviewed
local baseline. Future policy changes require reviewed snapshot adoption.

## Current source selection

Memory selects Shared Tooling 0.23.0 at
`0b351cd2ba2bcd8927b783a5fec89afc8013f708`, with 70 explicit roots and 103 payloads.
The canonical [preview and refresh procedure](consuming-snapshots.md#preview-before-refreshing)
exports committed bytes from a clean source checkout. Never patch vendored files
or attribute dirty source to a committed export. Keep directly invoked helpers as
explicit roots; required companions are derived by the exporter.

The current selection retires the host-installer producer fixture and its two
fixture companions. Memory's failure-evidence test now invokes the actual
installer through its public interface with controlled assets and a failing
version probe, then verifies the retained payload and original diagnostics across
its archive round trip. It no longer relies on a producer test's early exit,
environment switches or private fixture directory layout. The implementation
matrix remains at Shared Tooling. The source revision and all retained payload
bytes, tool pins, workspace lockfiles and compiler selections are unchanged.

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
| Release source, receipts, rollback and Make dispatch | repo-tool tests and `make test-release-adapters`, with substituted Git/Cargo effects |
| Actual index formatting and failure isolation | `make test-hooks` |
| Archive selections, hosted transport bindings and retained failures | `make test-tools` and `make test-failure-evidence` |

Fleet reporters, exporter regression fixtures and generic runner, parser,
installer, Make, formatter, LOC and checksum matrices are not consumer tests.
Run them at Shared Tooling's owner when qualifying its implementation. No local
product proof, native host or installed-tool admission is removed by that split.

## Tool setup and formatting

The shared catalog owns common host/Cargo pins and the IC tool catalog. Memory
selects its validation compiler, library MSRV, independent qualification workspace
and Testkit CLI in its own manifests and Makefile. Follow
[host support](host-support.md) for prerequisites and matching native evidence.
Source adoption, explicit setup, local activation and native qualification are
separate; an exported script or configured CI job is not acceptance evidence.

Memory extends the common ordered setup/check targets with `install-ic-tools`
and `ic-tools-check`, and exports `.tools/ic/bin`. Common setup supplies
`.tools/host/bin` and `.tools/rust/common/bin`. Prepared bundles, failed attempts
and old selections remain retained; refreshing source never silently installs
or deletes tools. Fleet reports run in Shared Tooling; local `make cloc` selects
the root workspace, or an explicit `CLOC_MANIFEST` for the independent graph.

Formatting covers the root workspace and `testing/runtime-qualification` through
one paired `FORMAT_WRITE_COMMAND` / `FORMAT_CHECK_COMMAND`. Shared admission
checks the selected cargo-sort and rustfmt before dispatch. Neither formatting
command builds, fetches, installs tools or changes selected lockfiles. Hook setup
is explicit and exercises real index effects only in allowed disposable fixtures
or an explicitly authorized commit.

Testkit owns PocketIC selection, provisioning, offline admission and managed
lifecycle. The separately selected CLI is outside both maintained lockfiles;
`install-runtime-server` prepares it explicitly, `runtime-server-check` is offline
and `test-runtime` uses prepared tools and both caches. Installed qualification
retains its separate authority. No local server installer or supervisor competes
with Testkit. See the [qualification runner](../testing/runtime-qualification/README.md).

## Release and evidence boundaries

The unchanged shared runner owns ordering, locks, saved intent and Git effects.
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
