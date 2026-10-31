# Current handoff

Delivery of the sibling patch refresh and pending issue fixes is authorized to
crates.io/GitHub. Release core 0.60.2, Metrics Stable 0.61.1 and Logging Stable
0.60.1 in supplier-first order. Platform 0.15.3, Metrics 0.24.1, Logging 0.23.3
and Build/Host/editor 0.73.2 are selected; Wire 0.13.0 and Testkit Native/CLI
0.75.0 remain unchanged. Both catalogs now select Platform 0.15.3/Types 0.15.1,
with the same stable-structures 0.7.2 and PocketIC 16.1.0 pair. No runtime/codec
implementation or stored-data contract is changed by these upstream patches.
Evidence and temporary files stay under `target/delivery/upstream-issues/`.

The working tree on delivered `c5b1f89` implements issue #145’s conservative
prose-only CI selection and a local repair for #146’s macOS hook-fixture failure.
The following implementation evidence predates this authorized delivery; hosted
acceptance and issue closure are checked against the final delivered source. Issue updates and evidence are retained under `target/issues/145/`.

The classifier is canonically exported from the existing reviewed Shared commit
`143a9abb9ae81feae9400ff3844b03b1ddfdd75a`; no newer dirty sibling rules or release
adapters are adopted. Snapshot VERSION stays the committed 0.27.21, with 79 roots
and 133 payloads. Root README, docs navigation and current handoff are the only
prose allowlist entries. Embedded member READMEs, machine/build inputs, policy,
workflow, unknown paths, unsafe modes, missing history and classifier failures
retain full qualification. The first adoption run is full because the policy is
new. See [the consumer selection contract](../ci-selection.md).

The docs job checks links and snapshot integrity; tooling lint remains unconditional.
All nine existing native matrix check names remain. Docs-only jobs explicitly
report that native suites did not run. Full changes retain all existing native,
Wasm, runtime, MSRV, artifact and failure-retention steps. Release gates remain
independent of this optimization. No measured timing or cost reduction is claimed.

The latest delivered hosted run is
[41224519753](https://github.com/dragginzgame/ic-persistence/actions/runs/41224519753).
Its macOS ARM native job fails in `local-tools-prepared`: Rustup treats `sort` as
an invalid toolchain/subcommand during hook formatter admission. The fixture’s
proxy-name dispatch is replaced by explicit selected-toolchain invocations.
Linux `make test-hooks` passes prepared/missing/wrong tools, partial staging and
both-workspace formatting. This is not macOS proof; #146 remains open pending
delivery and the required hosted matrix. The failed job log is retained locally.

#144’s release groups are implemented and delivered, including independent core
0.60.1 and Metrics Stable 0.61.0 publication while Logging Stable stays 0.60.0.
Local full release gates and registry checksums passed. It remains open because
current native-host acceptance is blocked by #146, not by an old cancelled run.
The issue records now identify that concrete remaining boundary.

Local qualification passes Actionlint, ShellCheck, snapshot/source comparison,
real documentation checks and `make test-ci-selection`. Selection fixtures execute
the actual workflow script with read-only Git observations substituted and cover
docs/code/mixed/policy/workflow/packaged/unknown paths, missing history, command
failure, invalid output, executable/symlink changes, tags/PRs and whitespace
refusal. Native matrices and full-step preservation were compared against the
delivered workflow. No fixture commits/tags are created. New tests participate in
`make test-tools`; no product APIs, formats, dependencies or package versions change.
