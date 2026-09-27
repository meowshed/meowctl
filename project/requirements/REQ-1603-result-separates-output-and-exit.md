---
id: REQ-1603
artifact: requirement
topic: exec
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1603

The result MUST carry stdout, stderr, and the exit code separately.

`v0.1.0` returns a struct with exactly these three fields from `ctx.run`, and
component code branches on `exit_code`, so the shape is load bearing; see
REQ-2820 (from docs/spec/exec.md, high). The alternative was to return combined
output, which lost because component code branches on `exit_code` and reads
`stderr` separately today (from docs/design/0.2.0-requirement-tradeoffs.md,
high). Revisit it if parity with `v0.1.0` is abandoned (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-EXEC-003` in `docs/spec/exec.md`.
