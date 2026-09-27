---
id: REQ-1430
artifact: requirement
topic: fs
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1430

Every method MUST return a typed error carrying the path and the operation.

A caller that surfaces "permission denied" without saying which file sends the
user to read a hook's source to find out (from docs/spec/fs.md, high). The
alternative was propagating the OS error, which lost because "permission denied"
with no path sends the user reading hook sources, and the record expects nothing
to reverse it (from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-FS-030` in `docs/spec/fs.md`.
