# Current handoff

Sibling adoption and registry delivery are complete: Memory/Storage 0.60.3,
Metrics Stable 0.61.1 and Logging Stable 0.60.1. Each group passed its standard
release gate; the published archive checksums match the qualified receipts.
Independent group versions, ledgers, tags, receipts and recovery are active.

Platform 0.15.3, Types 0.15.1, Metrics 0.24.1, Logging 0.23.3 and
Build/Host/editor 0.73.2 are selected. Wire 0.13.0, Testkit Native/CLI 0.75.0,
stable-structures 0.7.2 and PocketIC 16.1.0 are unchanged. Both maintained
Platform graphs use the selected patches. Stored formats and allocation custody
are unchanged.

The [delivered full CI matrix](https://github.com/dragginzgame/ic-persistence/actions/runs/41229172386)
at `a6354951fe63e293645c4316d5610971ad750ce5` passed native validation,
installed runtime/upgrade/rollback and compiler-floor checks on Linux, Apple
Silicon macOS and Intel macOS. The hook fixture uses explicit selected-toolchain
Rustup dispatch; both macOS native suites passed. This completes the native-host
obligations for release groups (#144) and the hook repair (#146).

Conservative CI selection (#145) is delivered. Only the reviewed exact three-file
prose policy can select documentation-only checks. Unknown, source, workflow,
policy and unsafe observations retain full qualification. Required native check
names remain present; failed classification or documentation admission fails
those checks. Selection fixtures cover normal and failure paths. A docs-only
result reports that native suites did not run; it never replaces native evidence.
See [CI selection](../ci-selection.md) for the qualification boundaries.

The reviewed Shared snapshot remains commit
`143a9abb9ae81feae9400ff3844b03b1ddfdd75a` (upstream 0.27.22 commit, recorded
VERSION 0.27.21), extended canonically with the classifier: 79 roots, 133 payloads.
No dirty sibling rules were adopted. Delivery evidence and retained failures stay
under `target/delivery/upstream-issues/`; earlier issue evidence remains under
`target/issues/145/`.
