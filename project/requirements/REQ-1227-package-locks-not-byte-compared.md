---
id: REQ-1227
artifact: requirement
topic: config
class: non-functional
status: withdrawn
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1227

Replaced by REQ-1223.

`pkgs.lock` and `pkgs.local.lock` MUST NOT be byte-compared against `v0.1.0`.

Withdrawn during onboarding: this obliges the test suite and not the system, so
no behaviour can verify it, and [REQ-1223]'s reason now says that byte-identity
covers `deps.lock` and not the package locks (from docs/spec/config.md, high).

`internal/lock/write.go` writes `deps.lock` by hand, in the layout [REQ-1223]
freezes, while `writePkgsLockFile` hands these two to `toml.NewEncoder` (from
docs/spec/config.md, high). They are two different writers, and nothing reads
either package lock back, not even `v0.1.0`, which writes them and never opens
them again, so what matters is that the structure round-trips (from
docs/spec/config.md, high). The alternative was holding these byte-exact too,
which lost because `v0.1.0` writes them with a different encoder than
`deps.lock`, and nothing reads either back; revisit it if something starts
reading a package lock (from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CONFIG-027` in `docs/spec/config.md`.
