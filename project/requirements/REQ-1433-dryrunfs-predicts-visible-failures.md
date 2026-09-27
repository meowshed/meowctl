---
id: REQ-1433
artifact: requirement
topic: fs
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1433

`DryRunFs` MUST fail where `RealFs` would fail for a reason it can see: a write
into a directory that does not exist and was not recorded as created, a symlink
over a regular file with no backup requested.

A dry run that succeeds where the real run will fail is worse than no dry run
(from docs/spec/fs.md, high). `v0.1.0`'s dry run is a set of early returns and
predicts no failure, and this requirement exists because `--dry-run` is
specified here as a prediction of the run (from docs/spec/fs.md, high). The
alternative was letting a dry run succeed at everything, which lost because a
dry run that succeeds where the real run fails is worse than none; revisit it if
false negatives become the common complaint (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-FS-033` in `docs/spec/fs.md`.
