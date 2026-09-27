---
id: REQ-2906
artifact: requirement
topic: ctx
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2906

`add_path(dir)` MUST do nothing when the directory is already first.

It emits nothing: a component that installs a tool whose binary is not yet on
`PATH` calls it so the next `ctx.run` can find that binary, which is what
`starAddPath` does by setting the process environment (from docs/spec/ctx.md,
high). A component wanting a shell to see the directory calls `emit` itself
(from docs/spec/ctx.md, high). The alternative was to emit a shell statement,
which the name suggests, and it lost because the standard library calls
`add_path` so the next `ctx.run` finds a binary it just installed, and that
needs the process `PATH` (from docs/design/0.2.0-requirement-tradeoffs.md,
high). Revisit it if a component needs the shell to see the directory and calls
`emit` (from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CTX-025` in `docs/spec/ctx.md`, its second obligation.
