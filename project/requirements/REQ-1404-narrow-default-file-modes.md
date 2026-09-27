---
id: REQ-1404
artifact: requirement
topic: fs
class: non-functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1404

A created file MUST have mode `0o600` and a created directory `0o700`, matching
`v0.1.0`, except where a caller passes an explicit mode.

Configuration holds secrets often enough that the default has to be the narrow
one (from docs/spec/fs.md, high). `v0.1.0` creates a lock file's parent
directory with mode `0o755` in `internal/lock/write.go` and the sentinel's with
`0o700` in `internal/state/state.go`, for the same directory, and `0o700` is the
value here because the config directory holds whatever a user's components put
in it (from docs/spec/fs.md, high). The alternative was reproducing `v0.1.0`'s
mixed `0755` and `0700`, which lost because reproducing an inconsistency is
transcribing a bug; revisit it if someone needs the config directory
group-readable (from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-FS-004` in `docs/spec/fs.md`.
