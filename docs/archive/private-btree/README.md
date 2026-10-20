# Retired private B-tree research

The independent page-based B-tree and its unpublished comparison benchmark were
removed after the maintainer selected a different direction. These documents
retain the investigation and its evidence; they are not active implementation
requirements or executable setup instructions.

The archive was frozen from the working tree based on commit
`3183e896f2a43be2380425d98200bf15af1b4e92`, including the pending private-tree
cleanup. The [committed source](https://github.com/dragginzgame/ic-storage/tree/3183e896f2a43be2380425d98200bf15af1b4e92)
precedes that uncommitted cleanup. Previously retained benchmark artifacts and
failure evidence are preserved in their existing locations.

- [Format and private API](experimental-btree.md)
- [IcyDB requirements and comparison results](icydb-btree.md)
- [Canonical workload investigation](canonical-workloads.md)
- [Former handoff](handoff.md)
- [Former upstream ownership and dependency investigations](upstreams.md)

The supported library remains the governed upstream collection adapter. See
[current ownership](../../storage/upstreams.md) and the [package guide](../../../crates/ic-storage/README.md).
This removal supplies no reader or conversion for retired custom-format bytes.
