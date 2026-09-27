---
id: REQ-3522
artifact: requirement
topic: cli
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3522

A failure to evaluate a component or to run a hook MUST NOT change the exit
code, which stays 0.

A shell that cannot start is worse than a shell that starts without its
integration: the failure is a configuration error the user fixes when
convenient, and the alternative is a terminal that reports an error on every
prompt and a login that fails (from docs/spec/cli.md, high). `v0.1.0` made the
same choice (from docs/spec/cli.md, high). The alternative was to exit non-zero
so the failure is visible, which lost because every shell spawn would report an
error and a login shell that fails is a machine you cannot log into; revisit if
`hook` stops being the command a shell runs (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CLI-062` in `docs/spec/cli.md`, its second obligation.
