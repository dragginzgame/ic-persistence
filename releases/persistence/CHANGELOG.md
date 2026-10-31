# Persistence release notes

Covers `ic-memory` and `ic-storage`. Independent group releases begin after the
coordinated 0.58.1 cutoff; [earlier shared history](../history.md) is preserved once.

## [0.60.3] - Unreleased

### 🩹 Fixed

- Fail every required native matrix check explicitly when classification or
  documentation admission fails. Preserve full-suite dispatch and docs-only
  reporting without relying on skipped dependency checks for refusal
  ([#145](https://github.com/dragginzgame/ic-persistence/issues/145)).

## [0.60.2] - 2026-10-31

### 🔧 Changed

- Add conservative documentation-only CI selection for three reviewed prose
  paths, with real documentation/snapshot checks and stable required check names.
  Code, policy, ambiguous history and failures retain full checks
  ([#145](https://github.com/dragginzgame/ic-persistence/issues/145)).

### ⬆️ Dependencies

- Adopt Platform 0.15.3 and Build/Host/editor 0.73.2. Preserve Wire 0.13,
  Testkit 0.75, stable-structures 0.7.2 and PocketIC 16.1.0.

### 🩹 Fixed

- Invoke the selected Rust toolchain explicitly in isolated hook fixtures,
  avoiding reliance on Rustup proxy-name dispatch. Local qualification passes;
  macOS acceptance remains pending
  ([#146](https://github.com/dragginzgame/ic-persistence/issues/146)).

## [0.60.1] - 2026-10-31

### 🔧 Changed

- Adopt Shared Tooling’s 0.27.22 rules (commit `143a9ab`, recorded VERSION
  0.27.21). Release preparation keeps the root’s versioned
  local dependencies aligned with their current packages, including patch
  releases, while preserving requirement operators and independent adapter versions.
  The root catalog includes every member and its owning release-policy alias;
  unused declarations add no dependency edges.

## [0.60.0] - 2026-10-31

### ⚠️ Breaking

- Adopt IC Wire 0.13.0 for durable codecs. Consumers exchanging exposed Wire
  errors must align that dependency identity. Encoded bytes, bounds and admission
  remain unchanged; no retained-data reset or migration is required.

### ⬆️ Dependencies

- Adopt Build/Host 0.73.1 and Testkit Native/CLI 0.75.0. Preserve the qualified
  stable-memory substrate and PocketIC 16.1.0 server selection.

## [0.59.0] - 2026-10-31

### ⚠️ Breaking

- Adopt IC Wire 0.12.1 in both maintained workspaces. Consumers exchanging
  Memory codec errors must align that dependency identity. Stored bytes, bounds and codec admission are unchanged.
  [Details](0.59.md#0590---unreleased).

### ⬆️ Dependencies

- Adopt Platform 0.15.1 in both maintained workspaces and Testkit 0.73.0.
  Preserve storage, runtime custody and the qualified IC substrates.

### 🔧 Changed

- Release Memory/Storage together and each stable adapter independently, retaining
  one workspace and existing numeric versions ([#144 — Separate core persistence and stable adapters into independent release groups](https://github.com/dragginzgame/ic-persistence/issues/144)).


- Adopt Shared Tooling 0.27.19, including symmetric group ledgers and explicit
  delegated delivery while preserving release and publication checks.
