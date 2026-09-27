---
id: REQ-2904
artifact: requirement
topic: ctx
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2904

When a checksum is given it MUST be verified before anything is written, for the
reason REQ-2430 gives.

The alternative was to download without journaling, which lost because an
interrupted run leaves a file the user did not have, with no undo (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names
nothing that would reverse it (from docs/design/0.2.0-requirement-tradeoffs.md,
high).

Migrated from `R-CTX-023` in `docs/spec/ctx.md`, its third obligation.
