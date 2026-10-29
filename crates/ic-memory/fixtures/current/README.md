# IC Memory current Wire fixtures

This is the only maintained durable fixture set. Files ending in `.wire.hex`
contain lowercase hexadecimal bytes. Tests decode, recover and re-encode them
through the production Wire paths to catch accidental format drift.

The Cell value is `ICML` followed by an IC Wire native commit-store record. Its
positional `current` option is required. A present snapshot contains generation,
`ICMWIRE1` marker, checksum and a bounded opaque byte payload, in that order.
The payload is a variable-length sequence of permanent key/numeric-slot rows;
its generation comes only from the checked physical header. ID 255 is refused.

The 0.56.0 hard cut retires earlier CBOR bytes. Fresh/reset memory or a separately
reviewed host transition is required. No old decoder or automatic clearing is
included. See [the current ledger contract](../../../../docs/current-ledger.md).

- `*_ledger.wire.hex`: logical record sequence, without Cell framing.
- `ledger_commit_store_single.wire.hex`: the full framed Cell value.
- `snapshot_store_valid.wire.hex`: a valid generation-two Cell value.
- `snapshot_store_corrupt.wire.hex`: the same snapshot with a corrupt checksum.
- `memory_manager_slot.wire.hex`: the numeric checked-slot representation.

Regenerate explicitly with the production encoders:

```sh
cargo test -p ic-memory --lib --locked --offline regenerate_current_wire_fixtures -- --ignored
```

Manager and Cell header layouts remain owned by the pinned substrate. Fixtures
change only the Cell value and ledger payload contract; recovery never rewrites
retained bytes on rejection.
