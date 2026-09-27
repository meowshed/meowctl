---
id: REQ-2433
artifact: requirement
topic: module
class: non-functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2433

Extraction MUST reject an entry whose path escapes the module root.

A tarball is remote input (from docs/spec/module.md, high). The alternative was
to trust the tar, which lost because a remote tarball could write outside the
module root, and the table names no condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-MODULE-033` in `docs/spec/module.md`.
