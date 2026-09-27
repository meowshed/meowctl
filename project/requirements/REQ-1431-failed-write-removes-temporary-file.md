---
id: REQ-1431
artifact: requirement
topic: fs
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1431

A failed atomic write MUST remove its temporary file.

`v0.1.0` does this on every error branch in `internal/lock/write.go`, and
leaving `.meowctl-lock-*.tmp` files behind in the config directory is the
failure to avoid (from docs/spec/fs.md, high). The alternative was leaving the
temporary file, which lost because `.meowctl-lock-*.tmp` files accumulate in the
config directory, and the record expects nothing to reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-FS-031` in `docs/spec/fs.md`.
