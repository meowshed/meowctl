---
id: REQ-3572
artifact: requirement
topic: cli
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3572

`self-update` MUST allow its binary download to take up to 5 minutes end to end.

`v0.1.0` downloaded the release through `http.DefaultClient`, which has no
timeout at all, so an unbounded wait was its behaviour (from
`git show v0.1.0:internal/cli/selfupdate.go` lines 44 and 69, high). A release
binary is 8 to 9 MB, so the shared 30-second bound fails any link slower than
about 2.5 Mbit/s (from `gh release view v0.1.0`, high; the rate is arithmetic on
those sizes). Five minutes is the bound `v0.1.0` gives `ctx.download`, the other
large transfer, and keeps a stalled download from hanging a terminal forever
(from `git show v0.1.0:internal/ctx/methods.go` line 840, medium).

Added during onboarding to answer which timeout `self-update` gets (BUG-0022).
