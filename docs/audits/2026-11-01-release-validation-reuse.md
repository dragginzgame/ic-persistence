# Grouped release validation reuse investigation

This read-only comparison concerns the 2026-11-01 cleanup delivery. It proposes no
change to current release admission. [Shared #334](https://github.com/dragginzgame/shared-tooling/issues/334)
owns review of the common contract.

| Group | Validation source | Gate seconds |
| --- | --- | ---: |
| Persistence 0.60.5 | `2ffeaa58e87e9e99e30bdb3ce4091f3595c637dc` | 261.099 |
| Metrics Stable 0.61.3 | `c74da381573674ec18a7b4a5a0aaf84b5fc99045` | 230.746 |
| Logging Stable 0.60.3 | `3b9647caafc43c3efba30f53162bcc3bdbd1cc9d` | 202.747 |

The three executed full gates total 694.592 seconds (11.6 minutes), excluding
preparation, final archives and upload. Their receipts have identical recorded
compiler identities and configuration. Only the preceding group's owned release
metadata differs between successive validation sources: the root catalog/lock,
selected package manifests and group notes. The comparison is retained in
`target/improvements-platform-2026-11-01/release-reuse-observation.json`; original
logs are `target/release-validation/attempts/verify.heWtzx`, `verify.J4VanX` and
`verify.egLFlk`. These are local Linux observations, not hosted timings.

Identical Rust files are insufficient to reuse a whole-gate receipt. Package
versions, dependency identities, Cargo environment, generated inputs and archive
VCS provenance can change observable behavior. Current receipts bind exact source,
group, intent, configuration and lock bytes. They must not be rebound to a later
source or reported as executed there.

A possible common design is opt-in stage proofs with complete caller-owned input
selection. Exact equality remains the default. A group-version projection would
need canonical editor verification and an explicit stage contract establishing
that projection's admissibility. Every group still needs fresh source/index/group
admission, metadata checks, final package graphs and archives, registry/provenance
checks and its own receipts. A new schema must retain the original executed proof
and record reuse separately; it must not reinterpret old receipts.

Missing, changed, ambiguous, failed or cancelled proof inputs require ordinary
execution. Qualification must include drift between stages, changed compiler or
native inputs, external checksum/feature changes, stale evidence, interruptions,
partial failure and all supported hosts. No execution saving is claimed until an
implemented contract measures actual reuse. The existing full gates and receipts
remain unchanged.
