---
id: REQ-2801
artifact: requirement
topic: ctx
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2801

`ctx` MUST expose the six data properties `New` in `internal/ctx/ctx.go`
registers: `home`, `dry_run`, `component_dir`, `state_dir`, `shell`, and
`platform`.

The alternative was to expose more properties or fewer, which lost because every
component reads these six (from docs/design/0.2.0-requirement-tradeoffs.md,
high). Revisit it if parity with `v0.1.0` is abandoned (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CTX-001` in `docs/spec/ctx.md`.
