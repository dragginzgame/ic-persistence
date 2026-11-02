# Platform runtime composition qualification

This unpublished consumer reproduces the dependency boundary in Storage #152:
local candidate Storage plus published `ic-platform` with its `runtime` feature.
It belongs to the existing independent qualification workspace and is excluded
from its default member. It neither replaces the native PocketIC driver nor adds
the facade to production Storage.

`make check-dependency-adoption` admits its native and Wasm graphs, requiring one
Platform, Runtime, Types and executor guard, then compiles both with Clippy. Cargo
also enforces the guard's exact internal requirement and singleton `links` identity.
The caller fixtures check that a duplicate guard or compilation failure refuses.

Run the composed native create/reopen check explicitly:

```sh
cargo test --locked --offline --manifest-path testing/runtime-qualification/Cargo.toml \
  -p ic-storage-platform-qualification
```

The same public helper compiles the deployed `DefaultMemoryImpl` path on Wasm.
Installed IO, bounds/growth refusal, upgrades and rollback remain the separate
Storage runtime suite; compilation alone does not qualify those behaviors.
This tests the working candidate, not a repaired registry Storage 0.64.0 archive.
A newly published Storage release is required before published-only composition
can pass. IC0-only backing is separate Platform #135 work.
