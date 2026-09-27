---
id: REQ-2012
artifact: requirement
topic: ops
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2012

`CopyFile`'s inverse MUST delete the destination.

The alternative was to restore prior content at the destination, which lost
because `copy_file` in `v0.1.0` journals a delete, and matching it keeps
journals interoperable (from docs/design/0.2.0-requirement-tradeoffs.md, high).
Revisit it if parity with `v0.1.0` is abandoned (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-OPS-012` in `docs/spec/ops.md`.
