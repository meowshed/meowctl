---
id: REQ-2116
artifact: requirement
topic: ops
class: non-functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2116

A journal written by `v0.1.0` MUST replay under `v0.2.0`.

The field names are reproduced exactly, so a journal left by one binary is
readable by the other (from docs/spec/ops.md, high). The alternative was a new,
tidier format, which lost because two binaries share a config directory during
the rewrite, and a journal written by one must replay under the other (from
docs/design/0.2.0-requirement-tradeoffs.md, high). Revisit it once the Go tree
is deleted, a condition the cutover has met without reversing the requirement
(from docs/design/0.2.0-requirement-tradeoffs.md, high).

The source also required the reverse, which [REQ-2190] now carries apart,
because nothing can run `v0.1.0` since the Go tree and the compatibility corpus
were deleted; this half is held by fixtures in the `v0.1.0` format (from
CLAUDE.md parity_is_the_contract, high).

Migrated from `R-OPS-020` in `docs/spec/ops.md`, its second obligation.
