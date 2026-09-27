---
id: REQ-1050
artifact: requirement
topic: common
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1050

Parsing any identifier from an untrusted string MUST return an error rather than
panic.

The inputs reach this crate from config files and lock files, both of which a
user edits (from docs/spec/common.md, high). The alternative was panicking on
malformed input, which lost because these strings come from files users edit,
and the record expects nothing to reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-COMMON-050` in `docs/spec/common.md`.
