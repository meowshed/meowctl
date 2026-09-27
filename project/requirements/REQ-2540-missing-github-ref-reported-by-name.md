---
id: REQ-2540
artifact: requirement
topic: module
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2540

A `github://` load whose ref resolves to no commit MUST fail with the module
exit code, naming the repository and the ref and saying that the ref wasn't
found.

The code and the message then tell a typo in the ref apart from an unreachable
or rate-limited API, as [REQ-2461] does for a registry module.

Added during onboarding from the failure-path research; the shipped binary exits
3 and reports only `answered HTTP 422` (from
crates/meowctl-module/src/github.rs:78-103, high).
