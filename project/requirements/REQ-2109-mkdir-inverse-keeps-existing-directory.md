---
id: REQ-2109
artifact: requirement
topic: ops
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2109

`Mkdir`'s inverse MUST do nothing when the directory already existed.

`inverseMkdir.created_by_meowctl` is the flag (from docs/spec/ops.md, high). The
alternative was to always remove the directory, which lost because it would
remove `~/.config` because a component created a file in it (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names
nothing that would reverse it (from docs/design/0.2.0-requirement-tradeoffs.md,
high).

Migrated from `R-OPS-013` in `docs/spec/ops.md`, its second obligation.
