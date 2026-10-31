---
id: REQ-2033
artifact: requirement
topic: ops
class: functional
status: approved
revised: 2026-10-31
elaborates:
verification: behavioural
supersedes: [REQ-2015]
---

# REQ-2033

`LinkFile`'s inverse MUST re-create the prior symlink when the link replaced
one, and remove the symlink otherwise.

A symlink a config file sits behind is the user's working setup, and removing
it on undo leaves the file gone: a failed apply on 2026-10-31 deleted 24 such
links, among them `~/.ssh/config` and every file under `~/.config/fish`
(BUG-0045). REQ-2015 removed the link in every case, which `Symlink`'s
inverse doesn't do (REQ-2014), and ADR-0052 records the break from `v0.1.0`.
