---
id: REQ-1805
artifact: requirement
topic: net
class: non-functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1805

`Http` MUST NOT follow a redirect to a non-`https` URL.

A redirect is the other way a plaintext fetch happens, and it is not visible in
the URL a caller passed (from docs/spec/net.md, high). This and REQ-1803 are one
setting on the client: a caller can check the URL it passed and cannot check
where a redirect goes, so the rule lives below the call (from docs/spec/net.md,
high). The alternative was to follow redirects wherever they lead, which lost
because a redirect to `http` is a plaintext fetch that is not visible in the URL
the caller passed (from docs/design/0.2.0-requirement-tradeoffs.md, high). The
trade-off table names nothing that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The choice is argued at
length in `docs/design/0.2.0-decisions.md` (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Only ureq's per-hop `https_only` check holds this today, and no test would
notice if the setting were removed; the evidence still missing is a test in
`crates/meowctl-net/src/real.rs` asserting that the client is configured
`https_only` (from crates/meowctl-net/src/real.rs:41-47 and :133-147, high).

Migrated from `R-NET-005` in `docs/spec/net.md`.
