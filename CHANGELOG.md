# Release notes

All three packages remain in one development workspace. Each release group owns
its package versions and canonical ledger; there is no aggregate workspace version.

| Group | Packages | Ledger |
| --- | --- | --- |
| `storage` | `ic-storage` | [Storage](releases/storage/CHANGELOG.md) |
| `metrics-storage` | `ic-metrics-storage` | [Metrics Storage](releases/metrics-storage/CHANGELOG.md) |
| `logging-storage` | `ic-logging-storage` | [Logging Storage](releases/logging-storage/CHANGELOG.md) |

[Shared release history through 0.58.1](releases/history.md) retains the original
coordinated entries and their earlier scope. See [release commands](RELEASING.md#release-groups).
