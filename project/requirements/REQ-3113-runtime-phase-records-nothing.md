---
id: REQ-3113
artifact: requirement
topic: engine
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3113

A runtime hook phase MUST NOT record what finished.

The runtime hook phases belong to no phase set, which is why there is nothing to
plan: `shell` runs on every shell spawn, so skipping a component because it ran
last time would mean a shell without its integration (from docs/spec/engine.md,
high). Recording thousands of completions would grow `state.toml` without bound,
and a rollback on a shell spawn would undo the previous one; see REQ-1013 and
REQ-3465 (from docs/spec/engine.md, high). The alternative was planning a
runtime hook like any other phase, which lost because a component skipped
because it ran last time means a shell without its integration, on every spawn
after the first (from docs/design/0.2.0-requirement-tradeoffs.md, high). The
trade-off table names no condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-ENGINE-035` in `docs/spec/engine.md`, its third obligation.
