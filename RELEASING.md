<p align="center">
  <img src="https://raw.githubusercontent.com/dragginzgame/shared-assets/main/ic-memory/ic-memory-readme-header.svg" alt="IC Memory — Stops upgrades from mixing up stored data" width="100%">
</p>

# Releasing IC Persistence

This guide covers explicitly authorized release work. Ordinary fixes and release
preparation leave edits unstaged; an explicit commit or PR request authorizes its
scoped contribution workflow, and an explicit standard release request authorizes
its documented commit/tag/push effects. See [contribution rules](rules/contributions.md)
and [AGENTS.md](AGENTS.md). The Rust `repo-tool` example is development tooling,
not canister runtime code. Prerequisites and native host evidence are in
[host support](docs/host-support.md).

This repository selects `RELEASE_DELIVERY=direct`: the authorized release pushes
the selected branch and tag atomically. Other delivery selections are refused
before runner dispatch. The shared runner's PR path requires separately adopted
merged-source preparation and receipt adapters; ordinary contribution PRs do not
select that release policy.

## Source and dependency preparation

Before 1.0, breaking consumer contracts require a minor release; compatible work
uses a patch. Maintain one numbered, undated pending entry, `## [X.Y.Z]`
(optionally followed by ` - Unreleased`), in both
`CHANGELOG.md` and `docs/changelog/<major>.<minor>.md`. Notes select the candidate
without changing package versions. Preparation refuses mismatched or duplicate
identities and preserves historical entries.

The Rust adapter calls the unchanged shared changelog finalizer with the saved
previous version, selected version and UTC date. Heading whitespace and a
candidate without a final newline do not block preparation. The optional
` - Unreleased` marker is replaced with the saved release date during preparation.
With no draft, the helper creates only the selected heading; it does not invent
note content.
Memory also rejects a selected identity already present in its bare historical
headings. Helper failures stop before any candidate output is installed.

Release preparation preserves the saved package README bytes without parsing or
rewriting dependency examples. Review examples during ordinary documentation
maintenance. The README remains source-bound release metadata: concurrent edits,
mode changes and receipt/source mismatches still refuse preparation or recovery.
The adapter skips publication when a metadata file already has the candidate
bytes. An unchanged README needs no replacement, and retries avoid republishing
metadata already installed before interruption. Changed metadata keeps its
ordinary rollback checks; `Cargo.toml` remains the final source-metadata write.
Saved releases prepared before this README cut retain their original projection.
Resume them using the original release tooling; do not rewrite their receipts or
archives to fit the new preparation contract.

Implementation and pending notes must be committed through an authorized
contribution workflow before releasing. The selected source must be clean, on
the selected branch, with no active build.

Development `make validate` qualifies working-tree edits, including packaging
with `--allow-dirty`; this does not create release receipts or admit dirty release
source. The release adapter and the separate `make package` command retain their
clean-source checks.

Preserve the tracked dependency selections. Release entry points prepare the
selected cache automatically before compiling the helper; for ordinary offline
checks, prepare it explicitly:

```sh
make fetch-dependencies
```

Both maintained workspaces track their `Cargo.lock`. The root lock is a required
source-bound qualification input. Release preflight prepares the existing
common tool selection with `make install-tools`,
then admits it through `make tools-check format-tools-check` before the full
gate. Common setup first checks platform/catalog and Rust/Cargo availability
without downloads or automatic toolchain installation. Setup follows coherent
saved-source admission/cache preparation and is not
replayed against interrupted metadata. Common setup includes the pinned Cargo tools and Memory's published IC Build
release editor; rustfmt/toolchain bootstrap and runtime-server setup retain their
explicit owners. `ci/release-tools.env` owns the editor selection. Ordinary
recovery calls check it offline; standard release bootstrap may explicitly
prepare it before rendering an interrupted manifest/lock pair. Standalone validation remains offline and checks all common tools plus
formatter prerequisites before builds, including under parallel Make.

Release preflight checks the selected cache with `cargo fetch --locked --offline`; it never retries
online or regenerates the lockfile. Validation, version refresh and packaging
remain offline. Version preparation changes only the coordinated package versions
and their local dependency references, never external dependency selection. Cache preparation remains a distinct step: the
documented release command authorizes `cargo fetch --locked` before helper
compilation, including during resume. Cargo's explicit offline environment and
configuration settings remain in effect. Ordinary helper calls use a locked,
offline cache check and never retry online. Preparation failures retain Cargo's
diagnostics/status and name `make fetch-dependencies` even before preflight.
Interrupted metadata writes prepare the launcher's coherent scratch workspace;
the tracked metadata remains unchanged. The internal `RELEASE_CACHE_PREPARE=1`
selection is scoped to release entry points and removed before dispatching the
compiled adapter, so its validation children retain offline behavior.
Prepare the complete checksum-pinned jq/yq/ripgrep/cloc host set with
`make install-host-tools`, as described in [host support](docs/host-support.md).
`make check-pins` checks declarations and
tracked locks; `make validate` includes it and the canonical pinning fixtures.
Release admission requires the selected tracked locks and committed source.
Recovery of an older attempt uses its original qualified source/tooling: old
source commits without the root lock cannot satisfy the new source binding.
Retain consumer receipts and archives with their original scope. A fresh normal
release does not require completing historical delivery first.

## Maintainer commands and recovery

```sh
make release-patch
make release-minor
make release-major
# Defaults: RELEASE_REMOTE=origin RELEASE_BRANCH=main
```

All three delegate to the unchanged vendored Shared Tooling runner. It owns
preflight, the full gate, exact preparation, explicit staging, the `Release X.Y.Z`
commit, annotated `vX.Y.Z` tag and atomic branch/tag push. All kinds use the same
pinned `make validate` gate. The branch must already exist remotely, with its
refreshed remote head an ancestor of local HEAD. Overrides select a remote name
and branch explicitly; the saved push URL identity must remain unchanged.

Normal release targets always start fresh from current clean, committed source
and the canonical version, with new preflight and full validation. They do not
scan historical plans, reuse their validation or replay their commit/tag/push
effects. A missing older tag or registry/Git delivery disagreement does not
select recovery. The newly computed candidate's stale local runner plan, staging
file list and PR receipt may be discarded; build logs, qualification receipts and
retained package archives remain evidence with their original source scope.

Recover a specific interrupted attempt explicitly:

```sh
make release-resume VERSION=X.Y.Z
```

Resume admits that version's saved source, kind, date, payload and destination.
After commit, newer descendant fixes can remain at HEAD: late adapters verify
the exact older `RELEASE_COMMIT` and annotated tag without another bump or commit.
Conflicting source/index, occupied locks, candidate refs/payloads or destinations
still stop execution. Inspect a stale lock's recorded owner before manual removal;
do not steal it. Recovery of an older release requires its original qualified
source/tooling and intact consumer evidence.

The Rust adapters retain their independent source-bound receipt admission.
Fresh validation preserves an admitted older prepared receipt for the same
release/destination and ancestor source before replacing it. Corrupt or unrelated
consumer qualification receipts remain conflicts; normal runner freshness does
not silently repair them. Retention failure preserves active evidence, and
immutable archives remain retained. Adoption and ordinary repairs do not
execute a release or recovery.

The recorded `make/consumer.mk` includes `make/release.mk` for standard entrypoint
routing and conflicting-goal admission. Memory's Makefile retains direct-only
delivery, binds the shared tooling root to this checkout even with conflicting
caller settings, and forces release-only cache preparation on the four standard
release targets. The Rust adapters continue to own metadata, qualification and
publication.

The adjacent `make/execution.mk` admits Make before recipes and resolves its
probe from the reviewed include rather than an ambient tooling root. Memory's
exported target policies also support GNU Make 3.81.

Push uses `--no-follow-tags --atomic` and exactly the selected branch and candidate
tag refspecs. There is no force push or non-atomic fallback. A lost push reply is
reconciled with exact remote identities; a failed remote query stops recovery.
Success retains logs, receipts and archives. Runner plans remain available for
explicit resume but are not historical archives. Cleanup and package publication
are separate operations.

The runner rechecks the committed payload, index, working files and exact
annotated tag object after the final push hook. Explicit resume of a completed
direct release verifies that local and remote tag objects agree and that the
observed remote branch contains the release commit. A known descendant tip is
valid. Missing or replaced tags, missing or diverged branches and failed remote
queries stop completion without recreating or pushing refs. Fetch unknown branch
history and reconcile conflicts before resuming; retain the selected recovery
plan while needed and preserve its consumer evidence.

The Rust consumer adapters consume the runner's eight `RELEASE_*` selections.
They validate source/input identities, finalize the root and detail notes with
the saved UTC date, update Cargo metadata, preserve the saved README and qualify the packages. The
explicit staged file set is `Cargo.toml`, `Cargo.lock`, `crates/ic-memory/README.md`, `CHANGELOG.md`
and the candidate minor-line detail file. The root lock edit changes only the
local `ic-memory`, `ic-storage`, `ic-metrics-stable` and unpublished
`ic-logging-stable` package versions; the independent qualification
lock is not a release edit. Git mutations belong exclusively to the common runner.

Preflight checks both working files and the entire index. Restoring a working
file does not hide unrelated or arbitrary staged content. The same index guard
runs again before commit admission, including file modes. Committed checks read
metadata from the selected SHA and require its sole parent, release subject and
exact prepared metadata to match saved intent.

The root `[workspace.package]` owns the release version and MSRV;
all four package manifests inherit them and stay unchanged during a bump.
The helper launcher requires all inherited declarations and matching local lock
versions. Interrupted metadata recovery copies all member manifests unchanged
and leaves tracked inputs intact; the root manifest is rendered for the selected
coordinated lock. Split package versions are conflicting input, because all packages
lock entries are published together in one atomic file replacement.
The canonical dependency example is in the package README.

Metadata writes use same-directory atomic replacement, with `Cargo.toml` last.
An interrupted earlier write can resume from exact original/prepared files. Once
the candidate manifest is present, all owned metadata including the lock is
complete; prepared checks can finish offline workspace verification and
packaging from saved successful validation, without another bump or full gate.
Returned preparation failures restore only owned edits; conflicting files and
independently changed dependency selections are preserved and refused.

If the candidate lock precedes the base manifest, the Make helper launcher uses
`--no-deps` Cargo metadata and the prepared yq to find the selected target
directory. It compiles current source against a coherent scratch manifest and
an unchanged copy of the lock, using `--locked --offline`. It runs the adapter
in the real checkout; compilation alone grants no release authority. Preflight
requires the saved source-bound successful validation before accepting this
partial state. Metadata and builds remain under `repo-tool-bootstrap/` in the
selected target directory. Normal helper calls use the original workspace.
Compiler admission returns the checked workspace MSRV on stdout, with compiler
identities/errors on stderr. Archive qualification carries that value into its
consumer manifest and native compiler dispatch rather than repeating a floor.

`release-version`, `release-files` and the other named adapters are runner
interfaces, not alternative maintainer orchestration. They require the saved
selection and evidence where applicable. `make qualify-release` can finish final
package qualification without Git effects, using the original prepared evidence.
Creating a final receipt requires HEAD to remain the selected release commit
through packaging. A changed HEAD stops before recording final evidence;
historical recovery reuses intact existing qualification.

## Evidence and publication

Under Cargo's actual target directory (including configured overrides),
`release-validation/` contains:

- `<version>-validated.json`: successful full-gate source, saved selection,
  original lock bytes, toolchain identities, configuration and exact command.
- `<version>-prepared.json`: that validation plus refreshed lock digest and the
  package qualified before the release commit.
- `<version>.json`: final package bound to the exact release commit. Cargo embeds
  Git metadata, so final packaging follows commit creation.
- `artifacts/<sha256>.crate`: retained qualified archives independent of Cargo's
  replaceable working package path.
- `attempts/verify.*`: unique stdout/stderr logs, including failed full gates.
  Replaced successful validation receipts are also archived here before replacement.

Receipt JSON streams into the shared typed durable publisher with the same
pretty JSON format and replacement permissions. Serialization must finish before
publication; a failed producer preserves the previous receipt and retains its
original error and any staging cleanup failure. Existing prepared/final receipts
and archives are checked, never silently repaired. Missing/corrupted prepared
evidence stops final qualification before packaging; a working archive replaced by failed final
packaging does not invalidate its intact retained prepared archive. Completed
prepared/final checks reuse valid evidence without repackaging.

Archive retention streams into the shared durable publisher's private staging
file and checks the copied digest before publication, then independently hashes
the retained file. Publication can report an error after complete bytes become
visible. Retry checks and reuses those bytes; an existing corrupt archive stops
qualification rather than being replaced. Staging allocation and synchronization
belong to `ic-host-fs`; release identities and receipts remain local.

Older-commit recovery requires intact prepared and final receipts and retained
archives. The replaced working package may belong to newer source; only the
retained archive for the selected older SHA supplies historical package evidence.
The selected lockfile, compiler identities, build configuration and qualification
commands must still match the saved evidence. Changed qualification inputs,
missing receipts or corrupted archives stop recovery. No receipt schema changes
or evidence migration are required by the 0.27.1 tooling update. If an older
release has no final receipt, qualify that exact release commit first using
`make qualify-release`; the adapter never packages newer source as that release.

Compiler identities include pinned Cargo/rustc and MSRV rustc. Discovered Cargo
configuration digests and build flags/profile overrides are recorded. Qualification
rejects compiler/wrapper replacements: `RUSTC`, `RUSTC_WRAPPER`,
`RUSTC_WORKSPACE_WRAPPER`, `RUSTDOC`, their `CARGO_BUILD_*` aliases and corresponding
`build` keys in checkout, ancestor and Cargo-home configuration. Unset even empty
assignments. Ordinary recorded flags/profile settings remain supported.

The current receipt schema is a hard cut from the earlier phase workflow. Old
receipts are not accepted or migrated. Finish outstanding earlier releases with
their original tooling before adopting this workflow. Preserve old evidence;
there is no automatic deletion or conversion of it or canister stable data.

```sh
make publish-dry-run
make publish # Separate network effect; requires crates.io credentials.
```

Helper compilation and workspace discovery use explicit `--offline` flags;
the launcher preserves the caller's `CARGO_NET_OFFLINE` setting for dispatched
commands. Publication needs network access, including registry inspection in a
dry run. An explicitly offline caller remains offline; setup/validation never
change that setting to retry online.

`PUBLISH_DRY_RUN=1 make publish` also selects a dry run. Publication requires the
exact commit, annotated tag, selected dependencies and qualified archive; it
never creates replacement evidence. A fresh checkout alone is not qualification.
The selected release and exact observed annotated tag object are checked again
immediately before Cargo dispatch and after successful Cargo return. A changed
or missing tag stops the check without replaying publication or changing refs.
A post-dispatch conflict does not undo external effects.
If publication is interrupted, inspect crates.io's exact version before retrying;
a lost reply is not proof of failure. Release targets never publish implicitly.

## Focused workflow checks

The runner and Rust validation adapter use the shared Make execution check before
dispatching gates. It rejects inherited ignore-errors, dry-run, question, touch
and version-only modes; rerun without those modes. Normal release selections and
jobserver settings remain inherited. A refusal preserves earlier validation
receipts and does not create a successful validation attempt.
The recorded Shared Make guard still has a reproduced gap when an explicit
command-line `MAKEFLAGS` replacement hides unsafe invocation modes:
`make -i release-patch MAKEFLAGS=`
can report success after a failing runner. Keep normal failure-enforcing Make
settings; the canonical correction is tracked in
[Shared #30](https://github.com/dragginzgame/shared-tooling/issues/30).

- `make test-tooling`: Rust adapters with substituted Git/Cargo/gate effects;
  recovery, rollback, input binding, artifact refusal and publication checks.
- `make test-release-adapters`: the shared checker exercises Make entry points,
  runner failures and conflicting selections with a substitute runner. Local
  fixtures cover selection forwarding, unique gate logs and locked bootstrap.
- `make shared-tooling-check`, `make test-hooks`, `make fmt-check` and
  `make lint-tooling`: snapshot, setup/formatting and portable tooling checks.

These tests create no commits/tags/pushes and perform no publication or full gate.
They do not prove a live release or native macOS behavior. The declared native CI
matrix includes them in `make validate-toolchain`. Generic runner implementation
cases qualify at Shared Tooling; Memory keeps its actual adapter and recovery
contracts. Local development uses focused checks, and full validation requires
explicit authority under the [shared cadence](DRAGGINZGAME.md#validation-and-evidence).
Release, publication and installed runtime qualification retain their separate authority.
Current focused evidence is recorded in
[release workflow qualification](docs/release-workflow-qualification.md).

## Coordinated package publication

Memory, Storage, Metrics Stable and the unpublished Logging Stable adapter inherit
the common workspace version. Preparation updates all four local lock records and
the internal Memory/Storage requirements; the Storage and Metrics Stable manifests
are admitted unchanged as a source-bound release surface. The initial 0.50.0
conversion is complete. Pre-consolidation receipts and plans retain their
original source/tooling; do not convert them with the current adapter.

All three qualified archive digests are retained in every package receipt. Packaging
creates all three archives offline and independently builds extracted packages and a
consumer on native Rust 1.91 and the selected compiler, then checks Wasm. The
consumer patches registry names only to those extracted archives, avoiding Cargo's
temporary-registry offline download failure and preventing accidental use of an
older published Memory. Its manifest/lock/logs are retained under
`target/package-qualification/`; the maintained root lock is unchanged.

Logging Stable remains `publish = false`: it advances with the workspace but has
no archive receipt or publication entry. Existing receipt fields and publication
order remain unchanged. Its implemented adapter and publication integration must
be qualified before adding it to the publication roster.
Resume saved releases from before Logging Stable using their original source and
tooling; the current workspace projection requires its fourth local lock record.

`make publish` admits clean release source, exact tag, all three archives and receipts
before registry effects. It observes all three crates.io versions; only HTTP 404 means
absent. A conflicting checksum, yanked version or failed observation stops the
command before uploading. It uploads Memory first, confirms the expected registry
checksum, then uploads Storage and finally Metrics Stable. `--no-verify` avoids repeating compilation during
upload: the exact archives already have independent package qualification.

Shared's canonical publication runner retains each attempt, exact package selector,
Cargo output and separate Cargo/logger statuses under
`target/release-validation/publication/`. Interrupted locks are retained for
inspection rather than automatically broken.

After a partial upload or lost response, retain the release checkout, receipts and
archives and rerun `make publish`. Matching registry checksums are skipped; no
automatic overwrite, version selection or retry occurs. If visibility is delayed,
the command stops and can be rerun later. `publish-dry-run` never supplies evidence
of real publication; Storage's registry resolution may still require Memory's
selected release to be available. The offline archive consumer proves the
unpublished pair independently. No real publication was performed by migration.
