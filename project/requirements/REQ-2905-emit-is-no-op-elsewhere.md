---
id: REQ-2905
artifact: requirement
topic: ctx
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2905

`emit(text)` MUST be a no-op outside the `shell` and `login` phases.

This is how a component contributes to the shell environment, and stdout in any
other phase would corrupt a piped run; see REQ-1013 (from docs/spec/ctx.md,
high). The alternative was to always write to stdout, which lost because a piped
`apply` would emit shell code into the pipe (from
docs/design/0.2.0-requirement-tradeoffs.md, high). Revisit it if `emit` gets its
own channel (from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CTX-024` in `docs/spec/ctx.md`, its second obligation.
