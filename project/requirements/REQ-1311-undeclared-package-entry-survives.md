---
id: REQ-1311
artifact: requirement
topic: config
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1311

An entry for a package no component declares any more MUST survive.

The alternative was replacing the file each run, which lost because a package a
component stopped declaring would vanish silently, and the file is the only
record that it was ever managed, and the record expects nothing to reverse it
(from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CONFIG-026` in `docs/spec/config.md`, its third obligation.
