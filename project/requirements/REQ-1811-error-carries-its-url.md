---
id: REQ-1811
artifact: requirement
topic: net
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1811

An error MUST carry the URL it is about.

A resolution fetches an index, a tarball, and a commit; an error that says only
"connection refused" does not say which (from docs/spec/net.md, high). The
alternative was to report the failure without the URL, which lost for the same
reason (from docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off
table names nothing that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-NET-011` in `docs/spec/net.md`.
