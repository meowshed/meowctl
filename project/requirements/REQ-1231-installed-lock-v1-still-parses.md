---
id: REQ-1231
artifact: requirement
topic: config
class: non-functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1231

The schema version 1 form of `installed.lock`, a bare `components = [names]`
array, MUST still parse, with every version treated as unknown.

`installedLock.versionMap` in `internal/cli/apply.go` handles both, and a user
upgrading from an earlier build has the older form on disk (from
docs/spec/config.md, high). The alternative was requiring version 2, which lost
because a user upgrading from an earlier build has version 1 on disk and would
see a parse error; revisit it if enough releases pass that nobody has it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CONFIG-031` in `docs/spec/config.md`.
