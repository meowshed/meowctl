---
id: REQ-2342
artifact: requirement
topic: starlark
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2342

When `/etc/os-release` is absent or can't be read, `platform()` MUST return
empty distribution fields in place of failing.

An unknown distribution isn't a failure: a distribution-specific `select` case
falls through to its default, and a `distros` guard drops the component with its
reason [REQ-3012].

Added during onboarding from the failure-path research; the shipped binary
returns empty fields on every Linux machine, because it never reads the file
(from crates/meowctl-starlark/src/platform.rs:28-44, high).
