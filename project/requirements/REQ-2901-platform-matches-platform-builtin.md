---
id: REQ-2901
artifact: requirement
topic: ctx
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2901

`platform` MUST be the same struct `platform()` returns; see REQ-2207 and
REQ-1013.

The name is the base name of `$SHELL`, which is the login shell rather than
whatever spawned the process (from docs/spec/ctx.md, high). A component reads it
to choose between `set -gx` and `export`, so it has to be the shell that will
evaluate the line (from docs/spec/ctx.md, high). The alternative was to omit
`shell` outside a runtime hook phase, which lost because a component checking
`ctx.shell` would get attribute-not-found in place of `None` (from
docs/design/0.2.0-requirement-tradeoffs.md, high). Revisit it if parity with
`v0.1.0` is abandoned (from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CTX-002` in `docs/spec/ctx.md`, its third obligation.
