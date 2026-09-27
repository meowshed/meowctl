---
id: REQ-1226
artifact: requirement
topic: config
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1226

A run that declared no packages MUST leave both `pkgs.lock` and
`pkgs.local.lock` alone.

The alternative was replacing the file each run, which lost because a package a
component stopped declaring would vanish silently, and the file is the only
record that it was ever managed, and the record expects nothing to reverse it
(from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CONFIG-026` in `docs/spec/config.md`, its first obligation.
