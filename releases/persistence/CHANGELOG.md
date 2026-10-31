# Persistence release notes

Covers `ic-memory` and `ic-storage`. Independent group releases begin after the
coordinated 0.58.1 cutoff; [earlier shared history](../history.md) is preserved once.

## [0.60.0] - Unreleased

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
