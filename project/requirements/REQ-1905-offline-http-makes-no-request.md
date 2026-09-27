---
id: REQ-1905
artifact: requirement
topic: net
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1905

`OfflineHttp` MUST make no request.

It is how REQ-2443 is proved: a resolution that completes against `OfflineHttp`
made no network call, and no assertion about call counts can be forgotten (from
docs/spec/net.md, high). The alternative was to assert on a request counter,
which lost because a counter assertion can be left out of a test, while an
implementation that cannot make a request cannot be forgotten (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names
nothing that would reverse it (from docs/design/0.2.0-requirement-tradeoffs.md,
high).

Migrated from `R-NET-014` in `docs/spec/net.md`, its second obligation.
