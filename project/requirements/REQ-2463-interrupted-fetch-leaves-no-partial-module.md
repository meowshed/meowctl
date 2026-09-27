---
id: REQ-2463
artifact: requirement
topic: module
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2463

A fetch interrupted partway MUST NOT leave a partial tarball or a partially
extracted module that a later run would treat as cached; see [REQ-1431].

`v0.1.0` extracts straight into the cache directory, so an interrupted
extraction leaves a directory that `os.Stat` finds and every later run treats as
a complete module (from docs/spec/module.md, high). Extracting beside it and
renaming makes the directory's existence mean what the code already assumes it
means (from docs/spec/module.md, high). Extraction stages into a sibling
directory and renames it into place (from
https://github.com/meowshed/meowctl/pull/67, high). The alternative was to
leave the partial extraction, which lost because a later run treats it as
cached and evaluates half a module, and the table names no condition that
would reverse it (from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-MODULE-063` in `docs/spec/module.md`.
