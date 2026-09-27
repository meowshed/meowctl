---
id: REQ-2107
artifact: requirement
topic: ops
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2107

The `AppendFile` marker identifier MUST default to a generated one.

`inverseAppendFile` stores the marker; `removeMarkedBlock` is the removal (from
docs/spec/ops.md, high). The alternatives were to truncate what was appended, or
to always generate the marker (from docs/design/0.2.0-requirement-tradeoffs.md,
high). Truncating loses an edit made between apply and undo, and always
generating one means a component re-running appends a second block in place of
replacing its own (from docs/design/0.2.0-requirement-tradeoffs.md, high).
Revisit it if markers prove unreliable in a format that cannot carry comments
(from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-OPS-011` in `docs/spec/ops.md`, its third obligation.
