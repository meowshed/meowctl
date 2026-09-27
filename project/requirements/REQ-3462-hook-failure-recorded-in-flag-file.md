---
id: REQ-3462
artifact: requirement
topic: cli
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3462

A failure to evaluate a component or to run a hook MUST be recorded in
`.hook-error`.

The flag file is how the failure stays visible without being fatal (from
docs/spec/cli.md, high). The alternative was to exit non-zero so the failure is
visible, which lost because every shell spawn would report an error and a login
shell that fails is a machine you cannot log into; revisit if `hook` stops being
the command a shell runs (from docs/design/0.2.0-requirement-tradeoffs.md,
high).

Migrated from `R-CLI-062` in `docs/spec/cli.md`, its first obligation.
