---
id: REQ-2020
artifact: requirement
topic: ops
class: non-functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2020

The journal MUST be a file of newline-delimited JSON records, one per operation,
in the format `internal/rollback/rollback.go` writes: a sequence number, the
phase, the component, the kind, and the inverse payload.

The field names are reproduced exactly, so a journal left by one binary is
readable by the other (from docs/spec/ops.md, high). The alternative was a new,
tidier format, which lost because two binaries share a config directory during
the rewrite, and a journal written by one must replay under the other (from
docs/design/0.2.0-requirement-tradeoffs.md, high). Revisit it once the Go tree
is deleted, a condition the cutover has met without reversing the requirement
(from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-OPS-020` in `docs/spec/ops.md`, its first obligation.
