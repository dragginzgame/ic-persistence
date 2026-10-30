# Current ownership ledger

The ledger retains each permanent stable key and claimed numeric ID. It holds
at most 255 records, including governance. Omitted identities retain their
records and slots. Application lifecycle state is not stored. Store schema
versions, migrations and schema admission belong to each storage owner, such as
IcyDB's schema subsystem; Memory carries no parallel schema metadata.

One checked `generation` counter in the protected snapshot owns logical
commitment and advances on cold bootstrap. Matching warm bootstrap returns the
established capability without another write or admission replay. There is no
per-bootstrap event, timestamp, ownership-label history or upgrade log.

Fixture-only `testing::AllocationLedger::new(counter, records)` checks count,
unique key/ID ownership and empty-genesis rules under `test-support`. Decoded
ledgers and `testing::LedgerCommitStore` are passive DTOs; recovered authority
remains internal. Production callers use their runtime's recovery/diagnostics.
The stable Cell contains the commit store directly. Its required `current` field
holds the snapshot without an intermediate physical-store object.
The private commit-store readers admit bounded current bytes; protected recovery
still establishes integrity and counter binding. Raw readers are exposed only
under `testing` for explicit fixtures and do not grant open authority.

Bootstrap owns one recovered ledger through validation and staging. It protects
the staged commit, persists the owning Cell, then constructs committed allocation
authority. There is no intermediate persistence-confirmation token or public
manual commit path. Any failure before persistence publishes no new authority.

The records-only payload is an IC Wire native sequence, limited to 64 KiB and
255 rows. Each positional row contains a borrowed stable-key string and a numeric
slot. Decoding admits the shared byte, allocation, node and depth budgets before
Memory checks key/slot grammar and ownership integrity. Key text is borrowed
until its length and grammar have been checked; malformed input cannot request
an unchecked owned key allocation.

The Cell value begins with `ICML`, followed by an IC Wire native commit-store
record. Its required positional `current` option distinguishes explicit absence
from truncated input. The complete value is limited to 64 KiB + 4 KiB. A present
snapshot uses marker `ICMWIRE1`; marker, generation, payload length and opaque
payload bytes are checksum-bound. Logical recovery takes its generation from
that checked header. JSON remains diagnostic output, not a durable input format.
Wire owns serialization, resource admission and exact input consumption; Memory
owns the schema, numeric ceilings, checksum and recovery decisions.

The 0.56.0 hard cut replaces both earlier CBOR layers and removes the
CBOR dependency and local syntax walker. Public raw-record decoder failures now
carry `ic_wire::Error`; runtime serialization refusal is
`RuntimeBootstrapError::StableCellLedgerEncode`. Callers inspecting the previous
error types must update. Existing manager and Cell substrate layouts do not change.

Recovery checks framing, marker/checksum, bounded Wire records and ownership
integrity. Present invalid or incompatible state cannot initialize genesis or
fall back to another generation. Publication checks exact successors. IC message
execution owns stable-write rollback; arbitrary native backing needs its own
transaction boundary.

Physical diagnostics remain `CommitSnapshotDiagnostic::{Empty, Valid, Invalid}`.
Only `Valid` carries a checksum-checked generation, and logical ledger recovery
remains a separate boundary. Diagnostics do not grant allocation authority.

## Host configuration and component adoption

Components contribute `MemoryRequest::new(authority, key)` and use the host's
committed runtime. Exact warm host verification checks the complete declarations,
canonical allocation pool and policy identity. Component `verify_authority`
checks only its required keys and their current host-assigned owners; it does not
reassert the host's policy or grants. Hosts supply a 32-byte token
identifying all policy semantics/configuration and prepare fallible policy setup
before bootstrap. Labels, versions and token derivation stay with the host; Memory
compares the complete bytes without allocating a policy name. This token is not
persisted. These checks do not check application schemas or reinterpret stored collection bytes. The policy has one
`validate_allocation(key, slot)` callback for each external declaration; recovered
admission remains separate and is never replayed by doctor or warm bootstrap.

`MemoryAllocationPool::new(grants, excluded_ids)` accepts usable numeric IDs,
checks the domain, includes governance 0..=9, and stores one immutable mask.
`excluded_ids()` projects canonical ascending IDs; serialized pools contain
`excluded_ids`, not ranges. Equivalent orderings and duplicates have the same
binding. Canic or another host owns range grouping for presentation/configuration.
This pool policy is not durable ownership and changes no retained key/ID mapping.

## Hard cut and retained installations

The maintainer explicitly approved a hard cut to Wire for 0.56.0. Earlier
CBOR ledger/Cell bytes, including `ICMKEY01`, are retired: hosts need fresh/reset
memory or their own separately reviewed transition before installing this
release. The new `ICML` Cell-value prefix and `ICMWIRE1` protected marker are
incompatible with those bytes. Incompatible enclosing shapes fail record decoding;
a mismatched protected marker fails with `CommitRecoveryError::UnsupportedFormat`.
There is one current reader and fixture set, with no legacy decoder, converter,
fallback, or automatic reset. Numeric slots, allocation custody and the pinned
manager/Cell layout remain unchanged.

This approval changes source/fixture support. It resets no deployed canister,
real stable memory or backup. Consumers must rebuild on the new minor, adopt the
current Wire record/error contracts, and retain their own schema checks.
Each application owns its retained-data disposition and installation qualification.

Before installing over earlier bytes, identify the installation and its explicit
disposition. A whole-canister reinstall also discards application data and needs
its owner's authorization. Preserving retained data requires a separately
reviewed application transition carrying every key/ID binding and lifecycle
obligation into the current contract. Never clear only ID 0 while leaving
collection bytes behind. Keep earlier binaries and recovery evidence until the
application's disposition is qualified.

See [current qualification](current-ledger-qualification.md) for source-bound
measurements and installed checks. Earlier-format results do not establish
compatibility with this cut.
