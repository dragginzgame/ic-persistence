# Current handoff

Sibling patch delivery is complete: core 0.60.2, Metrics Stable 0.61.1 and Logging
Stable 0.60.1 are published through their standard gates. The final #145 admission
guard is selected for core 0.60.3: required native matrix jobs explicitly fail
when classification/docs prerequisites fail, rather than relying on skipped
checks. The isolated preview passes actual admission scripts for success,
failure, skipped and cancelled inputs, plus normal selection fixtures and lint.
This guard changes CI only; adapters and stored formats remain unchanged.

Platform 0.15.3, Metrics 0.24.1, Logging 0.23.3 and Build/Host/editor 0.73.2 are
selected. Wire 0.13.0, Testkit Native/CLI 0.75.0, stable-structures 0.7.2 and
PocketIC 16.1.0 are unchanged. Both Platform graphs now use Types 0.15.1.
The prepared macOS hook repair uses explicit selected-toolchain dispatch.
Evidence and retained failures stay under `target/delivery/upstream-issues/`.
The initial stale evidence-condition assertion was fixed; subsequent full gates
passed. A concurrent tool-store lock was allowed to release without deletion.

Issues #144/#145/#146 remain open until the final delivered source has the
required hosted qualification. Classification and real documentation checks have
passed on the earlier code push; all native matrices remain present. No hosted
macOS pass is inferred from local Linux or docs-only results. Issue closure will
cite exact successful source/run identities. Earlier local issue evidence stays
under `target/issues/145/`; no older cancelled run is an artificial acceptance gate.

The reviewed Shared snapshot remains commit
`143a9abb9ae81feae9400ff3844b03b1ddfdd75a` (upstream 0.27.22 commit, recorded
VERSION 0.27.21), extended canonically with the classifier: 79 roots, 133 payloads.
No dirty sibling rules are adopted. The exact three-file prose policy excludes
embedded member READMEs and all unknown/build/policy paths. See
[CI selection](../ci-selection.md) for full-versus-docs qualification boundaries.
