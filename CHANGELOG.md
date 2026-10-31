# Release notes

All four packages remain in one development workspace. Each release group owns
its package versions and canonical ledger; there is no aggregate workspace version.

| Group | Packages | Ledger |
| --- | --- | --- |
| `persistence` | `ic-memory`, `ic-storage` | [Persistence](releases/persistence/CHANGELOG.md) |
| `metrics-stable` | `ic-metrics-stable` | [Metrics Stable](releases/metrics-stable/CHANGELOG.md) |
| `logging-stable` | `ic-logging-stable` | [Logging Stable](releases/logging-stable/CHANGELOG.md) |

[Shared release history through 0.58.1](releases/history.md) retains the original
coordinated entries and their earlier scope. See [release commands](RELEASING.md#release-groups).
