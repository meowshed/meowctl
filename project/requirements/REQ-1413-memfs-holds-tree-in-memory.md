---
id: REQ-1413
artifact: requirement
topic: fs
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1413

`MemFs` MUST hold the whole tree in memory and touch no disk.

It exists for tests, and it is what makes the engine testable without a
temporary directory (from docs/spec/fs.md, high). The alternative was using a
temporary directory in tests, which lost because every engine test then needs a
filesystem and cleanup, and none can run in parallel safely, and the record
expects nothing to reverse it (from docs/design/0.2.0-requirement-tradeoffs.md,
high).

Migrated from `R-FS-013` in `docs/spec/fs.md`.
