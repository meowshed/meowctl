---
id: REQ-3430
artifact: requirement
topic: cli
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3430

Mapping an error to an exit code MUST happen here and only here, using the
taxonomy in [REQ-1030].

Here is the `meowctl-cli` crate (from docs/spec/cli.md, high). The alternative
was to map at the error site, which lost because a crate below would then know
about exit codes, and the trade-off record names no condition that reverses it
(from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CLI-030` in `docs/spec/cli.md`.
