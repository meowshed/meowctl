---
id: REQ-2454
artifact: requirement
topic: module
class: functional
status: approved
revised: 2026-10-31
elaborates:
verification: behavioural
supersedes: [REQ-2450]
---

# REQ-2454

A module already present in the lock MUST be used at the locked version
without running selection again, unless the manifest pins a different exact
version, which is resolved in its place. Selection also runs when a module is
absent, or when the caller asked to ignore the lock.

The lock keeps an unpinned or `latest` dependency still, so resolution doesn't
move with what the registry published today. A changed pin is the user asking
for that version: `v0.1.0`'s `syncOneDep` in
`internal/starlark/loader/registry.go` resolves the version `deps.mod` names on
every sync, and REQ-2450, which this supersedes, kept the stale locked version.
