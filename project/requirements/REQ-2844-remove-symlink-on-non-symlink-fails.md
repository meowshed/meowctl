---
id: REQ-2844
artifact: requirement
topic: ctx
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2844

`remove_symlink` on a path that is not a symlink MUST fail; see REQ-1422.

A `remove_symlink` on a mistyped path would otherwise delete a real file the
user wrote (from docs/design/0.2.0-requirement-tradeoffs.md:73 and
docs/spec/fs.md:101-102, high). It follows from REQ-1422 (from
docs/design/0.2.0-requirement-tradeoffs.md:301, high).

Migrated from `R-CTX-044` in `docs/spec/ctx.md`.
