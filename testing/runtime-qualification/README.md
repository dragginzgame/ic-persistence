# Installed runtime qualification

This independent virtual workspace contains the unpublished host package
`crates/ic-memory-runtime-qualification/` and owns its separate `Cargo.lock`.
The package keeps PocketIC dependencies out of
the library's normal graph. It installs the supplied
[`wasm-io-qualification`](../../crates/ic-memory/examples/wasm_io_qualification.rs) fixture in a
caller-owned server selected by IC Testkit. The Storage-owned
[`wasm-storage-qualification`](../../crates/ic-storage/examples/wasm_storage_qualification.rs)
fixture additionally checks maps, bounded cells and region scans through successful
upgrades and rollback of trapped updates/upgrades. It never starts or downloads a server.
The Logging-owned
[`wasm-logging-qualification`](../../crates/ic-logging-stable/examples/logging_qualification/main.rs)
fixture checks query non-persistence, update/upgrade rollback, canonical reopen
and watermarks after pruning every row. Its bounded native Wire protocol is shared
with the host runner; it installs no library endpoint or lifecycle hook.
IC Platform's optional native `pocket-ic` interface supplies the client,
locked to PocketIC 16.1.0; Testkit owns its reviewed 16.1.0 server selection
separately. Platform adds no lifecycle wrapper or alternate client. Prepare the
server explicitly with `make install-runtime-server`,
check it offline with `make runtime-server-check`, and prepare the selected IC
tools with `make install-ic-tools` before running `make test-runtime`.
The latter builds the supplied fixture and runner offline, then qualifies the
original Wasm and Binaryen's `-O3`, `-Os` and `-Oz` outputs. Optimization admits
`memory.copy`/`memory.fill` through `--enable-bulk-memory-opt`, without selecting
a broader feature set. Each variant receives the same
startup, IO and upgrade checks with a separately managed Testkit server. Wasm,
input hashes and optimizer/server/runtime logs remain under
`target/qualification/runtime/` in the repository root, including after failure.
Normal library Wasm builds and budgets remain unoptimized. Prepare both locked
dependency caches separately.

For the focused Logging proof, run `make test-runtime-logging` after preparing
the server and both dependency caches. It builds and installs only the original
Logging Wasm; it requires no optimizer and skips the Memory/Storage runtime suites.
The full `make test-runtime` includes Logging in each optimization variant.

Use the [qualification record](../../docs/runtime-io-qualification.md) for
inputs, commands, observed results and remaining limitations. The `IC_MEMORY_QUALIFICATION_WASM`, `IC_STORAGE_QUALIFICATION_WASM`, `IC_LOGGING_QUALIFICATION_WASM`
paths and `IC_TESTKIT_POCKET_IC_URL` are required when invoking the runner
directly. `--measure-valid-io` skips the rejection
and upgrade checks and does not require the Storage fixture; its output alone is not qualification evidence.
`--logging-only` requires only the Logging Wasm and the server URL and conflicts
with measurement mode. The Logging proof reports assertions, not cost measurements.
`--help` lists the supported arguments without reading Wasm or contacting the
server. Unknown and trailing arguments are rejected before runtime setup.
Measurements retain the `valid_io` instruction rows and additionally report
`valid_io_cycles` from the actual cycle-balance change around each 1,000-operation
update. That cycle total includes update admission and lease opening outside
the inner instruction counter. Corruption qualification exercises both lease
opening and growth checkpoints and subsequent poisoned-handle refusal.
