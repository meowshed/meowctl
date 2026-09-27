---
id: REQ-1806
artifact: requirement
topic: net
class: non-functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1806

A response body MUST be bounded at 64 MiB.

`v0.1.0` calls `io.ReadAll` on every response, so a server that streams forever
is an out-of-memory kill with no message (from docs/spec/net.md, high). The
bound is a number rather than a judgement, and 64 MiB is two orders of magnitude
above the largest module published (from docs/spec/net.md, high). The
alternative was to read the body to the end, as `io.ReadAll` does, which lost
because a server that streams forever is an out-of-memory kill with no message
(from docs/design/0.2.0-requirement-tradeoffs.md, high). Revisit it if a module
legitimately exceeds the bound (from docs/design/0.2.0-requirement-tradeoffs.md,
high).

Migrated from `R-NET-006` in `docs/spec/net.md`, its first obligation.
