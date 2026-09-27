---
id: REQ-1410
artifact: requirement
topic: fs
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1410

`RealFs` MUST perform the effect against the real filesystem.

`RealFs` is the implementation the binary constructs for every run that isn't a
dry run, so a real filesystem effect happens there or nowhere, and `DryRunFs`
and `MemFs` stand in for it only in a dry run and in tests (from
https://github.com/meowshed/meowctl/pull/57, high). No alternative was
considered, because it is the baseline (from
docs/design/0.2.0-requirement-tradeoffs.md:66, high).

Migrated from `R-FS-010` in `docs/spec/fs.md`.
