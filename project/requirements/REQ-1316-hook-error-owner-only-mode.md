---
id: REQ-1316
artifact: requirement
topic: config
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1316

`.hook-error` MUST be written with mode `0o600`.

It is a flag rather than a log: each failure replaces the last, because what a
user needs is the reason their shell has no integration now, not a history of
every spawn since it broke (from docs/spec/config.md, high). The alternative was
appending each failure to a log, which lost because a shell spawn happens
hundreds of times a day, and what a user needs is why it is broken now; revisit
it if diagnosing a failure that only happens sometimes becomes the need (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CONFIG-064` in `docs/spec/config.md`, its second obligation.
