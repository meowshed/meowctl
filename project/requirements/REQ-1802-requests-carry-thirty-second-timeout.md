---
id: REQ-1802
artifact: requirement
topic: net
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1802

A module or registry request MUST be bounded by a timeout covering the whole
exchange, from the DNS lookup to the last byte of the body, 30 seconds by
default, which is what `v0.1.0` gives its module loaders.

A shell hook that triggers a resolution is otherwise a hang with no output (from
docs/spec/net.md, high). The alternative was no timeout, as a bare `http.Client`
would have, which lost because a shell hook that triggers a resolution becomes a
hang with no output (from docs/design/0.2.0-requirement-tradeoffs.md, high). The
trade-off table names nothing that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-NET-002` in `docs/spec/net.md`.

Reworded during onboarding from the failure-path research to say which bound it
is; the shipped `RealHttp` sets the `ureq` global timeout, which is end to end
(from crates/meowctl-net/src/real.rs:40-51, high). A download gets its own
bound: REQ-2872 for `ctx.download` and REQ-3572 for `self-update`, because
`v0.1.0` gave neither the 30 seconds its loaders had (from
`git show v0.1.0:internal/ctx/methods.go` line 840 and
`v0.1.0:internal/cli/selfupdate.go` line 44, high).
