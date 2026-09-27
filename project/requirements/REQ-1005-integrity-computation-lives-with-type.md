---
id: REQ-1005
artifact: requirement
topic: common
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1005

Computing an `Integrity` from bytes MUST live with the type: SHA-384 in standard
base64, which is what `computeSRI` in `internal/starlark/loader/github.go`
produces and what every published `index.toml` carries.

Two things hash bytes, the module cache and `ctx.download`, and an
implementation each is how a tool ends up with two encodings of the same hash
and no way to tell them apart (from docs/spec/common.md, high). The alternative
was hashing where the bytes are, in the cache and in `ctx.download`, which lost
because two implementations of one hash is how a tool ends up with two encodings
of it and no way to tell a mismatch from a spelling, and the record expects
nothing to reverse it (from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-COMMON-005` in `docs/spec/common.md`.
