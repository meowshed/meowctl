---
id: REQ-1051
artifact: requirement
topic: common
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1051

Constructing a `Phase` from a string that names no phase MUST fail.

A `state.toml` written by a newer version can contain one; see [REQ-1241] (from
docs/spec/common.md, high). The alternative was mapping an unknown phase to a
default, which lost because a `state.toml` from a newer build would be silently
misread, and the record expects nothing to reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-COMMON-051` in `docs/spec/common.md`.
