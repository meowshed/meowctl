---
id: REQ-2915
artifact: requirement
topic: ctx
class: functional
status: approved
revised: 2026-10-25
elaborates:
verification: behavioural
supersedes: [REQ-2830]
---

# REQ-2915

In a read-only phase, `ctx` MUST expose no mutating method other than
`add_path`.

`add_path` changes only the `PATH` later `ctx.run` calls in this run see, so a
check that calls it changes nothing outside meowctl, and 100 meowctl-stdlib
`verify` hooks call it to find the tools mise installed. ADR-0051 records the
decision; REQ-2830 left `add_path` off on a count that missed those hooks.
