---
id: REQ-1412
artifact: requirement
topic: fs
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1412

`DryRunFs` MUST answer a read of a path it recorded a write for with the content
of that write, not with what is on disk.

Without this a hook that writes a file and then reads it back takes a different
branch under `--dry-run` than it will under a real run, and the plan stops
predicting the run (from docs/spec/fs.md, high). `v0.1.0`'s dry run is a set of
early returns and remembers nothing it pretended to write, and this requirement
exists because `--dry-run` is specified here as a prediction of the run (from
docs/spec/fs.md, high). The alternative was passing every read to disk, which
lost because a hook that writes then reads takes a different branch under
`--dry-run`; revisit it if the recorded writes get large enough to matter (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-FS-012` in `docs/spec/fs.md`.
