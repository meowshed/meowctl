---
id: REQ-2434
artifact: requirement
topic: module
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2434

Extraction MUST drop a single top-level directory when every entry in the
archive is inside one.

A release tarball, which is what every entry in the published registry points
at, holds `MODULE.meow` at its root, while a GitHub archive from
`github.com/<owner>/<repo>/archive/<ref>.tar.gz` holds
`repo-<commit>/MODULE.meow` (from docs/spec/module.md, high). `v0.1.0` extracts
both as they come and then looks for `<cache>/MODULE.meow`, so for a `github:`
dependency it reads no transitive dependencies and serves every file from a path
one directory above where the files are (from docs/spec/module.md, high). The
alternative was to extract the archive as it comes, as `v0.1.0` does, which lost
because every entry of a GitHub archive is under `repo-<commit>/`, so every file
of a `github:` module sits one directory below where the loader looks; revisit
if a module ships one top-level directory on purpose (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-MODULE-034` in `docs/spec/module.md`, its first obligation.
