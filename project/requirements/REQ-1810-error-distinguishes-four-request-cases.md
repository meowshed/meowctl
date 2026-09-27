---
id: REQ-1810
artifact: requirement
topic: net
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1810

The error MUST distinguish four cases that a request can reach: the URL was
refused before any request, the host could not be reached, the server answered
with a status, and the body could not be read.

A fifth, the refusal REQ-1814 produces, is not one of them because no request
happened (from docs/spec/net.md, high). REQ-2460 requires a module resolution to
say which of these happened, and it can only say it if this trait reports it
(from docs/spec/net.md, high). The alternative was one error for every failure,
which lost because REQ-2460 requires naming which of the four happened, and it
can only report what it is told (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names
nothing that would reverse it (from docs/design/0.2.0-requirement-tradeoffs.md,
high).

Migrated from `R-NET-010` in `docs/spec/net.md`.
