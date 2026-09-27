---
id: REQ-2140
artifact: requirement
topic: ops
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2140

A run that journals MUST NOT start its first phase unless its journal file can
be written.

A failure found at the first reversible effect comes after every earlier
non-reversible step, such as `ctx.run` or a package install, has already run
with nothing to undo it.

Added during onboarding from the failure-path research; the shipped
`Journal::open` only reads, and the first `append` creates the file, so an
unwritable configuration directory is found at the first reversible effect (from
crates/meowctl-ops/src/journal.rs:87-100 and journal.rs:155-162, high).
