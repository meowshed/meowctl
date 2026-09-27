---
id: REQ-3107
artifact: requirement
topic: engine
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3107

The `component_dir` of a component named by a bare name MUST be
`components/<name>/` when that directory exists and `components/` otherwise.

Both layouts are in use: `meowctl-stdlib` lays every component out as a
directory with an `init.star`, and a hand-written configuration usually has one
file per component (from docs/spec/engine.md, high). `cf1774d` added the
directory rule so a component's `render_file` finds the data files beside its
hook (from docs/spec/engine.md, high). The alternative was accepting one layout,
which lost because both are in use: the standard library is directories, and a
hand-written configuration is files (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names no
condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-ENGINE-017` in `docs/spec/engine.md`, its second obligation.
