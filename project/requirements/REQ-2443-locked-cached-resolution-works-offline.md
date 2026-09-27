---
id: REQ-2443
artifact: requirement
topic: module
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2443

Resolution MUST work offline when every module in the lock is cached and
verified.

A shell hook that triggers a resolution on a machine with no network is
otherwise a hang (from docs/spec/module.md, high). `v0.1.0` reaches the network
whenever a module is absent from the lock and never says that a fully locked,
fully cached configuration must not, which matters for `meowctl hook shell` on
every shell spawn (from docs/spec/module.md, high). The alternative was to
require the network, which lost because `meowctl hook shell` hangs on a plane,
and the table names no condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The table marks this row as
argued at length in `docs/design/0.2.0-decisions.md` (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-MODULE-043` in `docs/spec/module.md`.
