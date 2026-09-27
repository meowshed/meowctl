---
id: REQ-1314
artifact: requirement
topic: config
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1314

The editor MUST parse the file rather than match text.

`internal/rewrite/rewrite.go` matches a regular expression that requires `name`
before `version` and documents the limitation, so a file a user hand-edited into
the other order is silently not found (from docs/spec/config.md, high). The
alternative was porting the regular expression, which lost because it requires
canonical keyword order and reports "not found" for a declaration that is
plainly there, and the record expects nothing to reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CONFIG-051` in `docs/spec/config.md`, its second obligation.
