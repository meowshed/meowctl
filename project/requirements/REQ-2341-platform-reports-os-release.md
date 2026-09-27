---
id: REQ-2341
artifact: requirement
topic: starlark
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2341

On Linux, `platform()` MUST report the `ID`, `ID_LIKE` and `VERSION_ID` values
of `/etc/os-release`.

Every `distros` guard and every distribution case of `select` reads these
fields, and `v0.1.0` reads them from `/etc/os-release` in `linuxDistroInfo`.

Added during onboarding from the failure-path research; the shipped binary reads
no distribution, so the three fields are empty on every Linux machine (from
crates/meowctl-starlark/src/platform.rs:28-44 and
crates/meowctl-cli/src/run.rs:273, high).
