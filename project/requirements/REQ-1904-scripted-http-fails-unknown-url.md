---
id: REQ-1904
artifact: requirement
topic: net
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1904

`ScriptedHttp` MUST fail a URL that is not in the table rather than returning
empty bytes.

A test that silently gets nothing back passes for the wrong reason (from
docs/spec/net.md, high). The alternative was to return empty bytes for an
unknown URL, which lost for that reason (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names
nothing that would reverse it (from docs/design/0.2.0-requirement-tradeoffs.md,
high).

Migrated from `R-NET-013` in `docs/spec/net.md`, its third obligation.
