---
id: REQ-1803
artifact: requirement
topic: net
class: non-functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1803

`RealHttp` MUST refuse a URL whose scheme is not `https`.

`v0.1.0` accepts whatever string the index hands it, so a registry index could
downgrade a module fetch to plaintext by writing one `source` template; see
REQ-2413 (from docs/spec/net.md, high). The alternative was to fetch whatever
scheme the URL names, as `v0.1.0` does, which lost because one `source` template
in a registry index would silently downgrade every module fetch to plaintext
(from docs/design/0.2.0-requirement-tradeoffs.md, high). Revisit it if a module
has to be served from a host without TLS (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The choice is argued at
length in `docs/design/0.2.0-decisions.md` (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-NET-003` in `docs/spec/net.md`, its first obligation.
