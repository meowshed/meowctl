---
id: REQ-2903
artifact: requirement
topic: ctx
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2903

`download(url, dst, checksum = None)` MUST journal a `Download` op, so an
interrupted run restores what was at `dst`; see REQ-2016.

The alternative was to download without journaling, which lost because an
interrupted run leaves a file the user did not have, with no undo (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names
nothing that would reverse it (from docs/design/0.2.0-requirement-tradeoffs.md,
high).

Migrated from `R-CTX-023` in `docs/spec/ctx.md`, its second obligation.
