---
id: REQ-1012
artifact: requirement
topic: common
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1012

`Phase` MUST report whether it is read-only. The read-only phases are
`install_check`, `upgrade_check`, `uninstall_check`, and `verify`.

A read-only phase gets a restricted `ctx`; see [REQ-2830] (from
docs/spec/common.md, high). `v0.1.0` never asks whether a phase is read-only,
and every hook in every phase gets the full `ctx`, so the four names are what
the phases mean, not what a Go identifier says (from docs/spec/common.md, high).
The alternative was checking the phase name at each call site, which lost
because `v0.1.0` does, in `methods.go`, and the set has to agree with `ctx` and
the executor; revisit it if read-only stops being a property of the phase (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-COMMON-012` in `docs/spec/common.md`.
