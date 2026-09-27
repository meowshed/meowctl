---
id: REQ-1804
artifact: requirement
topic: net
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1804

A response whose status is not 200 MUST fail.

`v0.1.0` does this and the code is what tells a missing module from a
rate-limited one (from docs/spec/net.md, high). The alternative was to treat any
response as the body, which lost because a 404 page would be extracted as a
tarball (from docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off
table names nothing that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-NET-004` in `docs/spec/net.md`, its first obligation.
