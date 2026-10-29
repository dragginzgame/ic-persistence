# Current handoff

Released source is Persistence 0.55.2 at `464a400`. Pending 0.55.3 adopts published
IC Tooling 0.70.0 and IC Wire 0.7.22. Own versions remain 0.55.2; no commit,
release, registry publication, application deployment or data reset ran here.

## Selected graph and boundaries

Root Build, Host Artifacts/FS/Process and Testkit Native select 0.70.0. The exact
managed Build metadata editor and Testkit server CLI pins also select 0.70.0 and
are prepared through the reviewed installer. Platform is now solely 0.8.1 in the
root graph: Testkit Native no longer retains its older 0.7.6 facade.

Both catalogs/locks select Wire/derive 0.7.22 with existing default-feature and
capability choices. The incoming dirty root lock already selected that version;
Metrics 0.17.3 and unrelated identities are preserved. The independent graph
advances only Wire/derive from 0.7.21. Wire 0.7.23 was still an unpublished draft
when dependencies were selected. The published 0.7.21/0.7.22 library source trees
are identical; current consumer bytes and rejection behavior remain qualified
separately. No reset, decoder fallback or runtime API rewrite is introduced.

The stable-structures 0.7.2 substrate, PocketIC client/server 16.1.0, Rust selections,
independent workspace and reviewed Shared Tooling 0.27.6 snapshot are unchanged.
Tooling's upstream ledger/Registry type changes are not public Persistence APIs.
Four-package release ordering, mandatory Logging digests and original-source
recovery of older receipts remain the maintained policy.

## Focused evidence

Strict release-helper Clippy, both stable adapters' all-target Clippy and the
independent runner's all-target Clippy pass. The real selected metadata CLI admits
both manifest-first and lock-first interrupted bootstrap states without changing
tracked fixture inputs. The selected Testkit 0.70.0 CLI admits the prepared
16.1.0 server cache offline; no server was launched.

All 38 stable-adapter unit tests and five doctests pass, including current frozen
Wire rows, malformed records and continuity. All four runner unit tests pass;
its two explicitly installed tests remain ignored. The four-package version/
receipt projection regression passes. Logging fixture Wasm Clippy, dependency
pins, production graph, formatting, doc links and diff checks pass.

The selected native Testkit metered-backing regression and complete evidence are
retained in
`/tmp/ic-persistence-tooling-wire.NBx4Tn/`, together with incoming manifests,
lockfiles, CLI pins and the prior handoff. No full release suite, installed
runtime, native macOS or performance qualification is claimed. Earlier evidence
retains its original source/package identities; no attempt or archive was removed.
