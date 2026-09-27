---
id: REQ-1301
artifact: requirement
topic: config
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1301

A configuration directory holding `meowctl.star` but no `init.star` MUST exit
with the configuration code.

The alternative was reading `meowctl.star` as a fallback, which lost because
that is a compatibility shim for a rename, carried forever; revisit it if the
rename is old enough that nobody has the old layout (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CONFIG-004` in `docs/spec/config.md`, its second obligation.
