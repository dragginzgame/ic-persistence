# Current allocation contract

Allocation now belongs to Storage's named directory. The fixed-slot manager and
separate ledger are retired by the maintainer's explicit hard cut. There is no
legacy reader or automatic reset. Hosts must use fresh backing or explicitly
retire the complete previous image, including application allocations.

See the maintained [allocation guide](../crates/ic-storage/ALLOCATION.md) for
namespace admission, durable commits, resource limits, custody and recovery.
The [directory design](design/dynamic-allocation.md) records implementation and
qualification details. Historical ledger documentation remains historical proof,
not the current retained-data contract.
