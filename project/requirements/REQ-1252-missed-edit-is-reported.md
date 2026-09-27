---
id: REQ-1252
artifact: requirement
topic: config
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1252

An edit that finds no matching declaration MUST report it rather than write the
file unchanged.

`AppendComponent` in `internal/rewrite/rewrite.go` can append a duplicate, so
reporting a missed edit is a deliberate change (from docs/spec/config.md, high).
The alternative was writing the file unchanged, as `AppendComponent` can, which
lost because that is a silent no-op, or a duplicate declaration, and the record
expects nothing to reverse it (from docs/design/0.2.0-requirement-tradeoffs.md,
high).

Migrated from `R-CONFIG-052` in `docs/spec/config.md`.
