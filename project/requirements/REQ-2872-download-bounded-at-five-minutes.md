---
id: REQ-2872
artifact: requirement
topic: ctx
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2872

`download` MUST allow a transfer to take up to 5 minutes end to end.

`v0.1.0` bounds `ctx.download` at 5 minutes, apart from the 30 seconds its
module loaders get, because a tool archive is larger than a module (from
`git show v0.1.0:internal/ctx/methods.go` line 840, high). The shipped
`download` shares the one 30-second client, so a component whose download took a
minute under `v0.1.0` fails under `v0.2.0` (from
crates/meowctl-cli/src/run.rs:262 and crates/meowctl-net/src/real.rs:39-45,
high).

Added during onboarding to answer which timeout a download gets (BUG-0022).
