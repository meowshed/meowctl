---
id: REQ-2115
artifact: requirement
topic: ops
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2115

Reading the prior value MUST be part of computing the inverse, per REQ-2003,
because `defaults read` after the write returns the new value.

This is new behaviour: in `v0.1.0` both operations apply with no journal entry
at all, so a failed run leaves the user's system preferences changed with no
record of what they were (from docs/spec/ops.md, high). The alternative was to
leave them unjournaled, as `v0.1.0` does, which lost because these are the two
operations a user notices most (from docs/design/0.2.0-requirement-tradeoffs.md,
high). Revisit it if a prior value proves unreadable for some key, which would
make the inverse a lie (from docs/design/0.2.0-requirement-tradeoffs.md, high).
The choice is argued at length in `docs/design/0.2.0-decisions.md` (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-OPS-017` in `docs/spec/ops.md`, its fourth obligation.
