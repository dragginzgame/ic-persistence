# Game of Compute and Memory

A canister spends instructions, occupies memory, sends messages and competes for turns. A useful profiler explains every observed charge with its amount, triggering condition, source rule and an actionable hint. Optimizing this game means reducing expensive moves while preserving behavior.

This paper describes the local IC working tree on **21 October 2026**, based on commit `ee8086a9773725fd1058832a9400297cf0990112`, branch `pocketic-memory-access-tracing`, including diagnostic changes under review. Defaults are configuration values, not promises about every deployed subnet. The [instruction analysis](instruction_analysis.md) distinguishes this source tree from the older measured PocketIC control binary. Examples below are arithmetic deductions from code, not benchmarks.

## The moves to recognize first

| Move | Normal application default | Meaning / useful response |
| --- | ---: | --- |
| First native access to a 64 KiB memory region | **80,000 execution instructions** | Keep the touched working set small. |
| First write to that region | **Another 80,000 execution instructions** | An initial write can therefore cost 160,000, before ordinary work. |
| Start or resume a scheduled Wasm slice | **2,000,000 round instructions** | Scheduler capacity, separate from the canister's execution invoice. |
| Prepare a canister / finalize an inner iteration | **8,000,000 / 12,000 per canister** round instructions | Preparation is conditional; finalization covers all subnet canisters. |
| Execute one replicated Wasm message or callback | **5,000,000 cycles + instruction cycles** | At 13 nodes: 1 cycle/instruction for Wasm32, 2 for Wasm64. |
| Receive ingress | **1,200,000 + 2,000 per signed-message byte** cycles | Count the whole encoded envelope, not just application arguments. |
| Send an inter-canister call | **260,000 + 1,000 per request byte**, plus actual response bytes | Response delivery and callback execution are prepaid and later settled. |

Sources: [memory fault debits](</srv/shared/ic/rs/memory_tracker/src/deterministic.rs>), [scheduler defaults](</srv/shared/ic/rs/config/src/subnet_config.rs>), [cycle accounting](</srv/shared/ic/rs/cycles_account_manager/src/cycles_account_manager.rs>). **The 2M scheduling cost and 5M cycle fee are different rules and units.**

## Separate the scoreboards

1. **Execution instructions:** weighted Wasm, function entry, dynamic lengths, APIs and memory faults; these consume the message limit and feed execution-cycle pricing.
2. **Round/query-graph instructions:** host scheduling and admission overhead; they constrain throughput without being additional guest instructions or an execution-cycle bill.
3. **Cycles:** actual consumed fees. Keep real cycles separate from nominal accounting on free schedules.
4. **Balances in transit:** prepayments, transfers, reserved balances and pending refunds. Moving cycles between these accounts is not another consumed fee.

Nonreplicated queries have no ordinary replicated execution-cycle invoice; query budgets, admission limits and ongoing storage/allocation rent still apply.

A physical execution reconciles as:

```text
reported instructions = weighted Wasm + function entry + bulk length
                      + API/stable replacement fees
                      + heap access/dirty + stable access/dirty
                      + identified other debits + signed reporting adjustment
```

Rule IDs ending in `.v1` identify a meaning, not a permanently fixed price. Each dictionary entry retains source symbols, excerpts and hashes; each event supplies actual amounts and available configuration/correlation. Unexplained differences remain errors. `ic.runtime.other.v1` is for identified observations, never a balancing plug. [Collector](</srv/shared/ic/rs/embedders/src/wasmtime_embedder/profiling.rs>), [runtime event types](</srv/shared/ic/rs/interfaces/src/execution_environment.rs>).

For ordinary cycle fees, let `N` be charging-subnet nodes and `R` its reference size: components scale by `floor(base × N / R)`. Normally `R=13`; SEV-enabled application pricing can use `R=7`. Runtime rounding applies at individual component/settlement sites. **HTTP uses its own formulas containing N; do not scale those again.** System subnets normally waive execution, ingress, storage, allocation and creation fees; HTTP is explicitly free there. Chain-key services have a separate exception below. [Scaling/configuration](</srv/shared/ic/rs/config/src/subnet_config.rs>), [CompoundCycles](</srv/shared/ic/rs/types/cycles/src/compound_cycles.rs>).

| Cycle rule | At N=R=13, normal schedule | Condition and tip |
| --- | --- | --- |
| `ic.cycles.execution.v1` | `5M + I` Wasm32; `5M + 2I` Wasm64 | Prepay the instruction limit, refund unused variable work. A new callback is another execution; a DTS continuation is not another message base. |
| `ic.cycles.ingress.v1` | `1.2M + 2,000b` | `b=SignedIngress.binary().len()`. Management ingress uses its effective payer; small `update_settings` induction is delayed so settings can unfreeze a canister. |
| `ic.cycles.transmission.v1` | `260k + 1,000(request method-name bytes + payload bytes + actual response payload bytes)` | Applies to self/local calls too. Attached cycles are a separate transfer. |
| `ic.cycles.storage.v1` | `317,500 cycles/GiB/second`, plus `10,000/second` for nonzero canister memory | Rent uses `max(memory allocation, actual usage)`: execution memory, Wasm/globals, history, chunks, logs and snapshots. Free unused assets and avoid excessive reservations. |
| `ic.cycles.message_memory.v1` | Same `317,500/GiB/second` byte rate | Both guaranteed-response and best-effort message memory count; guaranteed-response reservations also occupy capacity. Smaller, shorter-lived queues reduce rent. |
| `ic.cycles.compute_allocation.v1` | `10M × allocation percentage / second` | Reserved compute costs even while idle. A value of 1 means 1%, not the entire subnet. |
| `ic.cycles.canister_creation.v1` | `500B` | Creation fee is consumed; the remaining attached cycles fund the new canister. |
| Storage reservation / freezing | Saturation-dependent reserved cycles; default reserved-balance cap `5T` | Reservation moves liquid cycles into reserved balance; it is not immediate consumption. The maximum reservation period is 300M seconds. Freezing preserves enough funds for the configured idle horizon, accounting for reserved balance. |
| Explicit burn / lost balances | Actual amount destroyed | `cycles_burn128` burns at most liquid balance above freezing; out-of-cycles automatic uninstall burns the remaining liquid+reserved balance. Deletion leftovers and undeliverable/dropped-message cycles are losses, not fixed service tariffs. |

Resource collection checks every **50 rounds or at checkpoints**, requires at least **10 seconds** elapsed, and defers paused update/install executions. It uses elapsed whole seconds and integer rounding. Reserved storage cycles can later pay eligible resource charges. Native management instruction work uses the Wasm32 variable cycle rate, **without** the 5M sandbox-message base or Wasm64 multiplier. Sources: [accounting: `idle_cycles_burned_rate_by_resource`, `ingress_induction_cost`, `management_canister_cost`, `storage_reservation_cycles`](</srv/shared/ic/rs/cycles_account_manager/src/cycles_account_manager.rs>), [memory included in rent](</srv/shared/ic/rs/replicated_state/src/canister_state.rs>), [balance transitions](</srv/shared/ic/rs/replicated_state/src/canister_state/system_state.rs>), [use-case inventory](</srv/shared/ic/rs/types/cycles/src/cycles_use_case.rs>).

## Ordinary instructions and API moves

| Instruction rule | Charged work |
| --- | --- |
| `ic.wasm.static.v1` | Prepaid original block operator weights. Examples: integer divide/remainder 10; direct/indirect call 5/10; scalar and selected SIMD load/store 1 in Wasm32, 2 in Wasm64; bulk copy/fill/init static 100; memory/table grow 300. The source weight table covers the other operators. |
| `ic.wasm.function_entry.v1` | Minimum 1 at function entry, split out of the original block weight rather than added twice. |
| `ic.wasm.bulk_length.v1` | Dynamic copy/fill/init length: bytes for memory, elements for tables. `memory.init`/`table.init` lengths remain unsigned 32-bit in Wasm64. |
| `ic.api.base.v1`, `ic.api.bytes.v1`, `ic.api.direct.v1` | Actual API-specific fixed, variable and separately observed direct debits. A variable quantity is not necessarily payload length. |
| `ic.stable.api.v1` | Stable read/write replacement: normally `20 + length`. Its generated copy is not also an original bulk-operation charge. |

Blocks are charged at metering points before all their operators necessarily execute. A trap can leave prepaid operators unexecuted. Source heatmaps allocate charged blocks; they do not count executed operators. [Weights and instrumentation](</srv/shared/ic/rs/embedders/src/wasm_utils/instrumentation.rs>).

Useful API base fees: `call_new` 1,500 plus callee-principal and method-name bytes at 1 instruction/byte, `call_perform` 5,000, `global_timer_set` and `cycles_burn128` 500; many small metadata/copy APIs have a 500 base. Reply append, reject and call-data append add **50 instructions per byte**. Log handling charges `2×min(n,C) + 50×min(n,remaining)`; `trap` adds `n`, and enabled `debug_print` adds `n`, besides base fees. Native stable grow uses an actual host base of 100; native stable-size replacements have no separate host base. Keep the called API and measured quantity beside the broad rule. [API constants](</srv/shared/ic/rs/embedders/src/wasmtime_embedder/system_api_complexity.rs>), [actual dispatch, logging and multipliers](</srv/shared/ic/rs/embedders/src/wasmtime_embedder/linker.rs>).

`ic0.cost_call`, creation/HTTP/signing/vetKD estimators only quote service prices. They still incur their own instruction work: usually a 500 base; `cost_http_request_v2` has 10,000, with applicable input-copy costs. They do not themselves perform or charge the quoted service. [Estimator implementations](</srv/shared/ic/rs/embedders/src/wasmtime_embedder/system_api.rs>).

## Memory territories and the stable opening

The page-map/bytemap granule is **4 KiB**; the deterministic native tracker admits aligned **64 KiB** regions. With `page_overhead=5,000`, admission costs `16×5,000=80,000`; first dirtying adds another 80,000 when dirty tracking is enabled. Subsequent accesses in the same physical execution reuse admission. Crossing a region boundary can trigger another fee for even a short operation. Bytemap bookkeeping is not another instruction tax. In `Ignore` modification mode, mapping is read/write and pays access only. [Granules](</srv/shared/ic/rs/memory_tracker/src/conversions.rs>), [actual access/dirty branches](</srv/shared/ic/rs/memory_tracker/src/deterministic.rs>).

“First stable read free, second paid” is conditional: a nonempty read copies directly from page-map buffers only when **every overlapping 4 KiB stable bytemap granule is untouched in that execution**. This avoids stable native fault fees, but still pays the replacement transfer and any heap destination access/dirty fees. If any requested granule was touched, the whole read selects native copy; it can pay each stable region's first native admission once. Reading a different untouched granule can still use the direct path. [Replacement path selection](</srv/shared/ic/rs/embedders/src/wasm_utils/system_api_replacements.rs>), [direct buffer reads](</srv/shared/ic/rs/embedders/src/wasmtime_embedder/system_api/stable_memory.rs>).

For one-byte reads, untouched memory, dirty tracking and the default overhead, excluding ordinary Wasm:

| Successive move in one execution | Transfer | Heap fees | Stable native fees |
| --- | ---: | ---: | ---: |
| Stable byte 0 → heap byte 0 | 21 | 160,000 | 0: direct |
| Same read and destination again | 21 | 0 | 80,000: first native access |
| Repeat again | 21 | 0 | 0 |
| Stable byte 4,096 → same destination | 21 | 0 | 0: untouched bytemap granule |

These are deductions, not measurements. A first stable write can pay stable access+dirty fees and heap-source access. Coalesce useful transfers or cache repeated values, but account for the extra heap regions and copies introduced by a cache.

Zero-length 32-bit `stable_read`/`stable_write` pays 20; stable64 variants return before the replacement debit and pay zero. Bounds-check order also differs: empty stable32 write checks stable bounds, whereas empty stable64 calls return earlier. These are API variants, independent of the module's Wasm32/Wasm64 mode. Zero-fee path IDs (`ic.stable.read.direct.v1`, `.native.v1`, `ic.stable64.read.zero_length.v1`, `.write.zero_length.v1`) explain decisions without inventing savings. [Replacement debits and bounds](</srv/shared/ic/rs/embedders/src/wasm_utils/system_api_replacements.rs>).

## Messages in travel and timers

A call pays request transmission, reserves maximum response transmission and callback execution, then settles actual response bytes and unused instructions. The byte fee includes request method name plus arguments; a rejection counts its message bytes plus `size_of::<RejectCode>()`. Self/local calls use the same fee path, although local request limits differ. Callback prepayment can require a top-up after a Wasm mode or schedule change; an unexecuted callback retains at most its prepaid fixed execution fee. **Do not sum prepayment and final consumption as two bills.** Queue-memory rent is additional to byte transmission. [Call/prepayment/refund accounting](</srv/shared/ic/rs/cycles_account_manager/src/cycles_account_manager.rs>), [response settlement](</srv/shared/ic/rs/execution_environment/src/execution/response.rs>).

Timers and heartbeats are replicated executions, so they consume normal instruction/cycle fees and scheduler capacity. Setting a global timer costs 500 instructions; it does not purchase a recurring subscription. The Rust **ic-cdk-timers 1.0.0** implementation uses a self-call for isolation/error handling. A simple firing can therefore produce **global-timer dispatch + self-call update + callback**: 15M cycle execution bases and, for its 32-byte method name plus 8-byte timer ID and empty response, 300k transmission cycles at N=R=13, before actual instructions. Shared dispatch can amortize overhead; retries/asynchronous work can add executions. Cancellation removes the SDK task but can leave an already armed dispatcher wakeup. This is an SDK-version example, not a universal timer tariff. [SDK dispatch](</home/devbox/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ic-cdk-timers-1.0.0/src/global_timer.rs>), [executor](</home/devbox/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ic-cdk-timers-1.0.0/src/timer_executor.rs>), [system-task charging](</srv/shared/ic/rs/execution_environment/src/execution/call_or_task.rs>).

## Installation, snapshots, chunks and logs

These are host instruction charges outside the physical Wasm block ledger. Their cycle conversion remains a separate view of the same work. System configurations zero the configured chunk/snapshot/log baselines, but byte components and operation-specific handling must still be read from the active configuration.

| Operation | Application instruction cost / charge point |
| --- | --- |
| Compile/create execution state | `20M + 6,000×K`, with K the instrumented local operator count plus 2 per global, excluding diagnostic instrumentation. Expected-compiled handling discounts the variable component by 100; the 20M base remains. Compilation failure uses the conservative `20M + 6,000×max(decoded bytes, supplied bytes)` estimate. |
| Assemble chunked Wasm | `1 MiB × number of chunk hashes` in assembly work, separately from compilation. |
| Upload Wasm chunk | 200M per charged upload, including a duplicate; chunk capacity is 1 MiB. |
| Take snapshot | `2B + logical snapshot bytes`, charged on success. |
| Load snapshot | `2B + logical snapshot bytes` on success, plus applicable compilation work. |
| Read snapshot data | `5M + requested maximum returned bytes` upfront; a later bounds failure does not undo already charged work. |
| Upload snapshot metadata | `2B + declared logical snapshot bytes` upfront. |
| Upload snapshot Wasm/heap/stable data | `2B + chunk bytes` upfront; despite the 5M data-baseline configuration comment, this implementation uses the 2B snapshot baseline. Snapshot Wasm-chunk upload uses 200M. |
| Resize log storage | `32 × previously stored bytes` when resize is needed; charged on stored content, not allocated capacity. A no-op has zero resize work. |

Install/reinstall/upgrade and snapshot loading use the execution-prepayment path: a single normal 5M message base plus their consumed instructions at the Wasm32 cycle rate, including for Wasm64 modules. Several lifecycle-hook traces within one installation do not each add another base. The other native management charges above use variable instruction pricing without that base. Installation also executes its actual lifecycle hooks. Stored Wasm, chunks, snapshots and logs subsequently occupy rented memory. Sources: [compile configuration](</srv/shared/ic/rs/config/src/embedders.rs>), [instrumented compilation count](</srv/shared/ic/rs/embedders/src/wasm_utils/instrumentation.rs>), [compilation execution](</srv/shared/ic/rs/embedders/src/wasm_executor.rs>), [management charge sites](</srv/shared/ic/rs/execution_environment/src/canister_manager.rs>), [baseline configuration](</srv/shared/ic/rs/config/src/subnet_config.rs>).

## External service moves

**Legacy HTTP:** with request variable bytes Q and declared maximum response L, cycles are

```text
N × (3M + 60kN + 400Q + 800L)
N=13: 49.14M + 5,200Q + 10,400L
```

Q includes URL, headers, body, transform method name and transform context. Omitted L means **2,000,000 bytes**: its response allowance alone costs 20.8B cycles at 13 nodes. This legacy service fee is not refunded merely because the actual response is smaller; excess attached payment is a separate refund. Choose realistic response limits. [Legacy formula](</srv/shared/ic/rs/cycles_account_manager/src/cycles_account_manager.rs>), [request sizing/version selection](</srv/shared/ic/rs/types/types/src/canister_http.rs>).

**PAYG HTTP:** let m=min responses, K=delivered responses and S=transformed bytes. These are cycles:

| Component | Formula | N=13 example |
| --- | --- | --- |
| Fully replicated nonrefundable base | `N(1M + 50Q + 140kN + 800N²)` | `38,417,600 + 650Q` |
| Flexible base; nonreplicated uses m=1 | `N(1M + 50Q + 90kN + (2,000N+100k)m)` | `28,210,000 + 650Q + 1,638,000m` |
| Per attempting replica | `50×raw bytes + 300×roundtrip milliseconds + floor(transform instructions/13)` | Depends on measured work. |
| Flexible/nonreplicated gossip per replica | `50NS` | `650S`; absent for fully replicated mode. |
| Response consensus | `N(10N+600)B` | `9,490B`; B is delivered response size, or `Σ(181+S_i)` for flexible responses. |
| Extra flexible responses | `N(2,000N+100k)max(K−m,0)` | `1,638,000` per extra response. |

The estimator reserves worst-case self-funding consensus allowances, including possible rejects and participating replicas; that reserve exceeds some actual outcomes. Base is retained, usage settles from actual reports, and unused allowances are refunded. Late reports can settle in later blocks; nonresponding replicas release their full allowance on timeout. A release first enters the **routing refund pool**, not the caller's liquid balance. Usage from an already-transferred request payment is not a second balance debit. `ic.cycles.http_allowance.v1`, `.http_usage.v1` and `.http_refund.v1` expose these stages. [Pricing functions](</srv/shared/ic/rs/https_outcalls/pricing/src/fees.rs>), [upfront allocation](</srv/shared/ic/rs/execution_environment/src/execution_environment.rs>), [actual asynchronous settlement](</srv/shared/ic/rs/messaging/src/canister_http_spent.rs>).

PAYG support is feature-gated and locally enabled by default, but ordinary `http_request` defaults to **legacy version 1** unless version 2 is selected; flexible requests use PAYG when enabled. `http_request_fee_beta` is comparative telemetry, not a third charged tariff. [Versions](</srv/shared/ic/rs/types/management_canister_types/src/http.rs>), [feature/configuration](</srv/shared/ic/rs/config/src/execution_environment.rs>).

Threshold **ECDSA, Schnorr and vetKD each cost 10B cycles at 13 reference nodes**, scaled by the destination charging subnet's N/R. System-subnet configurations retain those service fees; **NNS-origin callers are exempt**. Public-key retrieval does not itself incur a signature/derivation tariff. Other execution/transmission costs still apply. [Fees](</srv/shared/ic/rs/config/src/subnet_config.rs>), [`sign_with_threshold` validation and deduction](</srv/shared/ic/rs/execution_environment/src/execution_environment.rs>).

Public **Bitcoin/Dogecoin service tariffs belong to separately versioned canisters**. An official-documentation snapshot checked on 21 October 2026 gives:

| Service operation | BTC mainnet / DOGE mainnet | BTC testnet/regtest | Minimum attachment |
| --- | ---: | ---: | --- |
| Balance / fee percentiles | 10M | 4M | 100M / 40M |
| UTXOs / block headers | 50M + 1×service instructions | 20M + 0.4×service instructions | 10B / 4B |
| Send transaction | 5B + 20M/byte | 2B + 8M/byte | Deterministic payment; excess attached is retained. |

The attachment minimum is not the net fee for read operations; unused cycles return. Miner fees are separate. RPC canisters add service pricing: EVM documents `N×providers×(5,912,000+60,000N+2,400Q+800L+10,000,000)` including retained collateral; SOL documents `providers×(legacy_HTTP+10,000,000N)`. Request their current cost quote and pin service configuration before interpreting a run. These are external prices, not another replica tariff. [Official service-cost reference](https://docs.internetcomputer.org/references/cycle-costs/#bitcoin-integration-api).

The local privileged Bitcoin adapter does not implement those public tariff schedules. SDK attached-cycle constants alone do not establish final net fees or a particular deployed configuration. Pin the service interface/implementation before including its tariff in a run's dictionary. [Local adapter scope](</srv/shared/ic/rs/execution_environment/src/bitcoin.rs>), [Bitcoin SDK attachment helpers](</home/devbox/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/ic-cdk-bitcoin-canister-0.2.0/src/lib.rs>).

## Turns, capacity and hard boundaries

A round is a consensus-delivered execution batch, with scheduling iterations and possibly several canisters per thread. It is not a fixed wall-clock interval. The 300ms initial notary delay is one protocol delay, not a guarantee of three rounds/second; rank delay defaults are 1s on application subnets and 3s on NNS, with execution/backlog also affecting progress. PocketIC ticks advance simulated execution; default state-machine time increments by only **1ns per round** unless explicitly advanced. Auto-progress's 100ms minimum sleep does not establish mainnet round frequency. [Consensus delays](</srv/shared/ic/rs/limits/src/lib.rs>), [PocketIC/state-machine ticking](</srv/shared/ic/rs/state_machine_tests/src/lib.rs>).

| Execution limit | Application default | System default |
| --- | ---: | ---: |
| Scheduler threads | 4 | 4 |
| Round budget per thread | 4B | 7B |
| Replicated message / DTS slice | 40B / 2B | 50B / 2B |
| Single query method | 5B | 50B |
| Install-code message / slice | 300B / 2B | 1T / 5B |
| Paused long update executions | 4 | 4 |
| Install-code instruction rate per canister/round | 2B; excess defers/rejects later installs | Effectively disabled (`10^15`) |

Round budgets include scheduling overhead and reserves; **4B is a per-thread budget, not 4B executed Wasm guaranteed every round**. Per-canister preparation charges 8M when its completed instruction consumption is nonzero; every inner iteration finalizes all subnet canisters at 12k each. DTS dirty-page-copy optimization adds 3,000 round instructions per copied 4 KiB page when heap dirty pages exceed 262,144. Query call graphs separately have 5B instructions, 10s wall-time and depth 6, including 50M graph overhead per valid nested callee; the initial query does not pay that nested-call overhead. Inspect-message has 200M instructions. [Scheduler limits and comments](</srv/shared/ic/rs/config/src/subnet_config.rs>), [actual overhead paths](</srv/shared/ic/rs/execution_environment/src/scheduler.rs>), [query configuration](</srv/shared/ic/rs/config/src/execution_environment.rs>), [nested query accounting](</srv/shared/ic/rs/execution_environment/src/query_handler/query_context.rs>).

| Memory / network boundary | Default / scope |
| --- | --- |
| Architectural heap / stable | Wasm32 4 GiB; Wasm64 6 GiB; stable 500 GiB. |
| Configured Wasm-memory setting | Default 3 GiB; 0 removes this setting's ceiling, up to architectural caps. Enforced for Init/Start/Update and at installation finish; other API contexts are exempt from this setting, not architecture limits. |
| Unique stable accessed / dirty bytes per execution | Each limit separately: regular 2 GiB; Init/pre-upgrade 8 GiB; query/inspect/composite-query contexts 1 GiB. |
| Heap delta | Application: 75 MiB/canister/round, 200M bytes/inner iteration, 32 GiB initial reserve. System: 2B bytes/iteration, 140 GiB initial reserve, rate effectively disabled. Both: 140 GiB subnet capacity between checkpoints; overshoot/throttling rules apply. |
| Stable32 API addressing | 4 GiB / 65,536 Wasm pages; use stable64 beyond that. Stable32 size/grow and transfer paths enforce their API-specific checks. |
| Module / compiler | Uncompressed Wasm 100 MiB; code section 12 MiB; 1M operators and weighted compiler-complexity limit per function; 10k locals/function. Wasmtime/native stack 5 MiB (separate from a language stack in the heap); one table up to 1M elements; user multi-memory disabled. |
| Module interface / metadata | 50k local functions, 1k globals, 1k exported update/query/composite methods, sum of method names 20k bytes; 16 IC public/private custom sections totaling 1 MiB; 1,024 raw custom sections; names up to 1 MiB. |
| Subnet logical memory | Default 2 TiB, high-usage threshold 750 GiB, 10 GiB execution reserve; a nonzero registry maximum-state-size overrides capacity. Guaranteed-message capacity `min(15 GiB, heap-delta-capacity/3)`; best-effort `min(5 GiB, heap-delta-capacity/3)`, enforced by end-round shedding. |
| Logs | Default/minimum nonzero 4 KiB; maximum 2 MiB; limit 0 disables storage. Per-execution log delta also max 2 MiB. |
| Ingress per block | **1,000 messages**, 4 MiB ingress bytes and 4 MiB combined block payload defaults; not a promise of 1,000 executed updates/round. |
| Signed ingress message | Application 2 MiB; NNS 3.5 MiB; ingress expiry horizon 5min with 30s allowed clock drift. |
| Inter-canister payload | Remote request 2 MiB; local request 10 MiB; replies 2 MiB; nonreplicated query reply 3 MiB. |
| Queues / callbacks | Per-peer 500 request + 500 response slots; subnet callback soft limit 1M, guaranteed quota 50/canister. |
| Streams / best-effort calls | Target 10k messages / 10 MiB, with possible overshoot; best-effort response timeout at most 300s. |
| HTTP | Headers+body request and raw response each 2,000,000 decimal bytes; URL 8,192B; 64 headers, individual name/value 8 KiB, total headers 48 KiB; 3,000 outstanding requests; 500 responses/block excluding timeouts/async receipts. Client ceiling 60s, adapter timeout 30s; budget can shorten it. Delivered Candid response allowance adds 1,024B. |
| Canisters / reserved compute | 250k canisters/subnet default. Each canister can reserve 0–100%; four threads give 300% scheduler capacity, the current 50% allocation cap reduces this to 150%, and management preserves 1% headroom. |
| Stored chunks / snapshots | Chunk ≤1 MiB; 100 MiB chunk store budgets a full 1 MiB slot per distinct hash, up to 100 chunks. Snapshots: 10/canister by default. Both contribute to rent. |

Sources: [architectural memory constants](</srv/shared/ic/rs/types/types/src/lib.rs>), [stable/compiler configuration](</srv/shared/ic/rs/config/src/embedders.rs>), [module validation](</srv/shared/ic/rs/embedders/src/wasm_utils/validation.rs>), [memory-setting contexts](</srv/shared/ic/rs/embedders/src/wasmtime_embedder/system_api.rs>), [installation finish checks](</srv/shared/ic/rs/execution_environment/src/execution/install_code.rs>), [chunk slots](</srv/shared/ic/rs/replicated_state/src/canister_state/system_state/wasm_chunk_store.rs>), [log limits](</srv/shared/ic/rs/types/types/src/canister_log.rs>), [execution configuration](</srv/shared/ic/rs/config/src/execution_environment.rs>), [registry capacity override](</srv/shared/ic/rs/registry/resource_limits/src/lib.rs>), [message capacity](</srv/shared/ic/rs/replicated_state/src/metadata_state.rs>), [ingress/consensus limits](</srv/shared/ic/rs/limits/src/lib.rs>), [message payload limits](</srv/shared/ic/rs/types/types/src/messages.rs>), [queue implementation](</srv/shared/ic/rs/replicated_state/src/canister_state/queues.rs>), [stream targets](</srv/shared/ic/rs/config/src/message_routing.rs>), [HTTP validation](</srv/shared/ic/rs/types/types/src/canister_http.rs>), [adapter timeout](</srv/shared/ic/rs/https_outcalls/adapter/src/config.rs>), [engine stack/table limits](</srv/shared/ic/rs/embedders/src/wasmtime_embedder.rs>).

A successful DTS slice refill continues the same physical execution and memory tracker; it does not reset first-touch fees. A new message/callback does. Raw debits can cross a terminal limit before the runtime clamps reported instructions: preserve raw total, final total and signed adjustment. `ic.budget.dts_refill.v1` and `.message_limit_clamp.v1` explain budget events without adding work. [DTS accounting](</srv/shared/ic/rs/canister_sandbox/src/dts.rs>).

## What the profiler can honestly explain

The implementation records the instruction ledger and memory path decisions, original-module blocks plus final-code origin ranges, bounded physical call paths and actual page-fault evidence, and separate host events for cycles, management work, scheduler/query overhead, prepayments, transfers, reservations and refunds. Actual HTTP asynchronous settlement is observed rather than estimated from attached maxima. Upfront legacy/PAYG-base charges and per-replica allowance funding are distinguished; threshold signing and actual dropped-message losses have direct observations. Events carry explicit known correlations; host events are not assigned to whichever Wasm execution happened most recently. Keep source/configuration identity, exact large integers, missing origins, retention limits and truncated detail visible. Totals completeness is separate from detail completeness; a trace stopped before refunds cannot establish final net consumption. [Physical execution collector](</srv/shared/ic/rs/embedders/src/wasmtime_embedder/profiling.rs>), [host accounting observer](</srv/shared/ic/rs/cycles_account_manager/src/cycles_account_manager.rs>), [HTTP observer](</srv/shared/ic/rs/messaging/src/canister_http_spent.rs>).

Coverage is not omniscience: block totals allocate prepaid charges rather than executed operators; retained physical Wasm stacks and fault/scoped-operation origins are bounded (the underlying backtrace capture still walks full physical depth), and unproven native-helper intervals remain unmapped. Physical attempt IDs survive DTS resumes and connect retries only through actual runtime message/task identities. Aborted attempts retain available host-owned debit evidence; unavailable pages or counters remain explicit. Slice round-accounting totals, including scheduling-only dirty-copy yields, stay separate from guest instructions. Commit/discard status comes after actual state application or final install/upgrade settlement, and completed rows wait for that disposition before publication. Rust source and inline views still require matching SDK/debug artifacts and offline analysis. Separately deployed service-canister prices and unexercised paths need their own pinned evidence. A complete game explanation names these gaps instead of manufacturing source attribution or calling reserved cycles consumed. Agents should optimize the largest reconciled rule, then compare matched configuration, outcomes, memory working sets and final—not merely prepaid—costs.