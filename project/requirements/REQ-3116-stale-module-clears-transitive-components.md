---
id: REQ-3116
artifact: requirement
topic: engine
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3116

Every component belonging to a module whose fingerprint changed MUST be cleared,
including transitive ones `installed.lock` does not name directly.

`computeStaleComponents` in `internal/cli/apply.go` is the computation in
`v0.1.0`, and `fix: correct module updates` is why it exists (from
docs/spec/engine.md, high). The alternative was comparing recorded names only,
which lost because a module bump would not re-link, which is the bug
`fix: correct module updates` closed (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names no
condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-ENGINE-043` in `docs/spec/engine.md`, its second obligation.
