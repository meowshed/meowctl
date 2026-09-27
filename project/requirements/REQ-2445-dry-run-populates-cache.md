---
id: REQ-2445
artifact: requirement
topic: module
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2445

The cache MUST be populated during a dry run.

A dry run has to evaluate a configuration's modules to produce a plan at all,
and a machine with a cold cache could otherwise produce none (from
docs/spec/module.md, high). The cache is meowctl's own storage and not the
user's configuration, so filling it is not a change `--dry-run` promises not to
make; see REQ-3411 (from docs/spec/module.md, high). The alternative was to
refuse to fetch under `--dry-run`, which lost because a cold machine could then
produce no plan at all, which is the case a dry run is for; revisit if the cache
becomes user state and stops being meowctl's (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-MODULE-045` in `docs/spec/module.md`.
