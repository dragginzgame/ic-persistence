# Releasing IC Persistence

Release and publication require delivery authorization for this repository and
destination. Delegated delivery includes routine group, version and order choices
under the shared contribution rules. Ordinary fixes leave edits unstaged. See the
[contribution rules](rules/contributions.md), [shared release contract](docs/releases.md)
and [host prerequisites](docs/host-support.md). The native `repo-tool` example
supplies consumer policy to the existing Shared runner; it is not canister code.

## Release groups

All four crates remain in one development workspace and one dependency catalog.
The approved [group policy](ci/cargo-release-groups.json) is the executable owner;
root `Cargo.toml` comments give its member/version/ledger map.

| Group | Packages | Version source | Canonical ledger |
| --- | --- | --- | --- |
| `persistence` | Memory, Storage | `crates/ic-memory/Cargo.toml` | [Persistence](releases/persistence/CHANGELOG.md) |
| `metrics-stable` | Metrics Stable | `crates/ic-metrics-stable/Cargo.toml` | [Metrics Stable](releases/metrics-stable/CHANGELOG.md) |
| `logging-stable` | Logging Stable | `crates/ic-logging-stable/Cargo.toml` | [Logging Stable](releases/logging-stable/CHANGELOG.md) |

All members use literal `package.version` fields and group members match their
primary. Root `Cargo.toml` remains a virtual dependency/shared-metadata catalog
without `workspace.package.version`. There are no intermediate group manifests
or relocated packages. The initial layout adoption preserved all numeric versions.

Root [CHANGELOG.md](CHANGELOG.md) is an index. Earlier coordinated history through
0.58.1 is preserved once in [releases/history.md](releases/history.md), with the
original scopes/dates and retained historical details/evidence. Each group owns
one numbered pending entry in its canonical ledger. New detailed notes live at
`releases/<group>/<major>.<minor>.md`; the policy names the current core detail
file as metadata. Update it during source work when the group's next minor line
changes. An old crate-local changelog is only a navigation pointer.
The unpublished independent runtime workspace keeps its own version/lock outside
these groups under its existing approved scope.

Release, resume and publication require an explicit `RELEASE_GROUP`. There is no
default or implicit whole-workspace release. Missing selection refuses before
cache setup, helper compilation or release effects. For example:

```sh
make RELEASE_GROUP=persistence release-minor
make RELEASE_GROUP=metrics-stable release-minor
make RELEASE_GROUP=logging-stable release-minor
make RELEASE_GROUP=metrics-stable release-resume VERSION=0.59.0
make RELEASE_GROUP=metrics-stable qualify-release
make RELEASE_GROUP=metrics-stable publish-dry-run
make RELEASE_GROUP=metrics-stable publish
```

These are maintainer commands, not adoption checks. `release-patch` and
`release-major` keep their existing meanings. Pre-1.0 breaking public contracts
require a minor; compatible work uses a patch. Current Wire identity
changes select 0.60.0 pending notes, without bumping package manifests here.

Groups are release identities, not independent dependency graphs. Bare core
requirements retain a compatible minimum on core patch releases; a breaking core
release updates their shared root declarations without changing adapter versions.
The adapter then needs its own release to publish its changed requirement.
Current Wire 0.13 adoption exposes new Memory errors and adapter codec errors:
release the compatible core supplier first and then its adapters. Do not publish
an old consumer declaration under an existing version, broaden requirements to
hide a type mismatch, or remove production singleton checks. New versions of
Metrics or Logging remain real public type dependencies when exchanged with hosts.

## Preparation, validation and exact effects

Commit intended source through an authorized contribution workflow first. The
standard runner requires clean source/index and an admitted branch/destination.
This repository selects `RELEASE_DELIVERY=direct`; the runner owns commits, tags,
locking, ordered effects and atomic branch/tag push. PR release delivery remains
unselected. A contribution PR does not change that policy.

Prepare missing caches explicitly with `make fetch-dependencies` and, when needed,
`cargo fetch --locked --manifest-path testing/runtime-qualification/Cargo.toml`.
Release commands separately authorize locked cache and selected-tool preparation
before compiling the helper. Explicit caller offline settings remain in force;
ordinary validation/helper calls never fetch automatically or repair lockfiles.
`make install-tools` includes the pinned Build editor; `ci/release-tools.env`
owns its published selection. Build 0.73 supplies the group-aware renderer.
Toolchain/formatter and installed server preparation retain their separate owners.

Every group runs the same complete `make validate` release gate. Group selection
does not narrow Clippy, compiler-floor, Wasm, runtime prerequisites or whole-workspace
archive integration. Development validation may qualify dirty source; release
admission and `make package` retain clean-source checks. `package-workspace.sh`
qualifies all four current archives as one integration graph with each package's
own version. The selected group is then packaged alone: published suppliers use
their registry checksums, while coordinated members package together. The full
extracted integration graph is qualified using that final archive set; selected
receipts retain only the group's publishable archives.
This integration proof is not registry publication or isolation of incompatible stacks.

The native adapter admits saved source, group policy, compiler identities,
configuration hashes, selected lock bytes and immutable source-derived metadata.
The exact file plan matches Shared's `cargo-release-group.sh files`:
root manifest, root lock, selected ledger, every selected member manifest and declared group metadata. Other groups' package identities, notes
and unrelated requirements are preserved. An unchanged root manifest can be in
that plan without becoming a changed staged file. The index check requires exactly
the resulting changed bytes, including file modes; unrelated work is refused.

Returned preparation failures restore only admitted owned edits and retain all
rollback failures. Conflicting writes and independent dependency changes are
preserved and refused. For interrupted lock-first or manifest-first writes, the
launcher reads each group's version source and lock identity. It builds the helper
against coherent scratch copies through the prepared editor, leaving the real
tracked inputs unchanged. Compilation grants no release authority; the adapter
still needs saved source-bound validation before accepting an interrupted release.

The runner uses `Release GROUP X.Y.Z` subjects and annotated `GROUP-vX.Y.Z` tags.
Journals are `.git/release-state/GROUP-X.Y.Z.plan`; one repository-wide lock still
protects the shared source and Cargo lock. A retry must use the exact group,
version, remote and branch. Normal releases start fresh; recovery is explicitly
`release-resume`. Never remove another owner's lock or select a new version to
avoid conflicting evidence. Historical ungrouped and earlier group-layout releases/receipts remain retained.
Recover them with their original source/tooling; never reinterpret an unfinished
attempt under changed version ownership or ledger paths.

## Receipts and publication

Native validation/preparation/final receipts live under Cargo's selected target:
`release-validation/GROUP-X.Y.Z-validated.json`, `GROUP-X.Y.Z-prepared.json` and
`GROUP-X.Y.Z.json`. They bind group selection and exact source/destination; the
source commit binds the group policy. An admitted archive map must contain exactly
the selected members. Retained archives remain content-addressed under
`release-validation/artifacts/`; earlier attempts, logs, receipts and tools are
preserved. The common runner's saved intent, validation, prepared and final stages
remain separate evidence. No fixture pass proves a full gate or live release.

Publish from the exact selected group's qualified release checkout. Admission
requires clean source, release subject/parent, exact annotated tag, retained and
working archive digests, configuration and final receipts. Observe every selected
registry package before any upload; only HTTP 404 means absence. A yanked version,
checksum conflict or failed observation refuses all effects. Core publishes Memory
then Storage; each adapter publishes only itself. `--no-verify` avoids replaying
compilation already proved by archive qualification. Shared retains each upload's
stdout/stderr, exact selector and independent Cargo/logger statuses.

After a lost response or partial upload, rerun `make RELEASE_GROUP=... publish`
from that qualified checkout. Matching checksums are skipped; delayed visibility
stops safely for a later explicit retry. `publish-dry-run` is not publication
proof and an upstream group's selected release may need to be available first.
Never overwrite a published version or infer publication from cache/source state.

## Focused workflow checks

Select affected checks during development; full release gates require explicit
release/validation authorization:

- Native `cargo test -p ic-memory --example repo-tool --locked --offline` covers
  consumer source/index/receipt checks, all groups, rollback, retention and upload retry
  using substituted Git/Cargo/gate/registry effects; it creates no commits or tags.
- `scripts/ci/test-release-group-plans.sh` compares real native and shared exact
  plans, substituting only Git source observation and retaining unchanged source hashes.
- `scripts/ci/test-release-adapters.sh` exercises actual Make wiring, jobserver
  propagation, failure retention and parser/cache refusal using effect substitutes.
- `scripts/ci/test-consolidated-bootstrap.sh` uses actual offline Cargo and the
  prepared editor for both interrupted states in every group, and resolves/compiles
  coherent group upgrades with unchanged unrelated identities in disposable copies.

`make test-release-adapters` aggregates the three shell fixtures. Preserve partial
staging refusal, earlier artifacts and failed logs. None invokes a live release,
registry upload or installed runtime matrix. Native macOS acceptance stays with
its declared CI job. The inherited Make guard's command-line `MAKEFLAGS=` replacement
limitation remains tracked in [Shared #30](https://github.com/dragginzgame/shared-tooling/issues/30);
retain failure-enforcing Make settings and do not bypass guards or branch protections.
