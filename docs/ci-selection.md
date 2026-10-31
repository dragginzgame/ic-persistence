# Conservative CI change selection

The [workflow](../.github/workflows/ci.yml) uses Shared’s reviewed
[classifier contract](ci-change-selection.md). Its executable is exported from
the already selected commit `143a9abb9ae81feae9400ff3844b03b1ddfdd75a`.
No unrelated upstream release behavior is adopted.

[The exact allowlist](../ci/docs-only-paths.txt) contains only the root README,
documentation navigation and current handoff. They have no companion declarations,
are not embedded Rustdoc or generated/package inputs, and are not maintained
machine configuration. Member READMEs are embedded with `include_str!` and remain
outside the allowlist. New prose paths require explicit review; unknown paths,
code, workflow, policy and executable/symlink changes select full checks.

The classification checkout fetches complete history. Pull requests compare the
exact tested merge commit against their base using the merge base; pushes use
before/after identities and require ancestry. Policies must match in both
commits. Missing history, malformed observations, tags and classifier errors
select full qualification. Required matrix jobs explicitly fail admission when a prerequisite fails;
failed prose whitespace or documentation checks never become a successful result.
The first adoption run is full because its policy is new.

The docs job always checks snapshot integrity and local documentation links.
Tooling lint remains unconditional. All existing Linux, macOS ARM and macOS Intel
matrix check names remain present. For admitted prose, those jobs report a
separate documentation-only result without installing Rust or running native
suites. For full changes, every existing native setup, product, Wasm, runtime,
MSRV, upload and failure-retention step is preserved behind the full-scope guard.
No workflow-level path filter is used. Release commands still run their complete
configured gates, independently of this optimization.

`make test-ci-selection` executes the actual workflow classification script with
read-only Git observations substituted, then runs the real docs job command.
It covers docs/code/mixed/policy/workflow/packaged/unknown paths, missing history,
classifier errors, invalid output, executable/symlink changes, tags, PRs and
whitespace failure, plus failed/skipped/cancelled prerequisite admission. It checks required matrix names and full-suite dispatch.
Fixtures create no commits or tags. The test participates in `make test-tools`.
Linux fixture passes are not hosted native/macOS qualification and no timing or
cost reduction is claimed from these checks.
