---
id: REQ-1812
artifact: requirement
topic: net
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1812

`RealHttp` MUST perform the request.

The alternative was a real implementation only, which lost because every test
would then need a server, which is the shape `v0.1.0`'s loader tests have (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names
nothing that would reverse it (from docs/design/0.2.0-requirement-tradeoffs.md,
high).

Migrated from `R-NET-012` in `docs/spec/net.md`.
