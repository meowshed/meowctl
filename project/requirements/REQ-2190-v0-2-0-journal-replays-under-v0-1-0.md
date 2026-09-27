---
id: REQ-2190
artifact: requirement
topic: ops
class: non-functional
status: approved
revised: 2026-09-27
elaborates:
verification: judgement
verifier: person
---

# REQ-2190

A journal written by `v0.2.0` MUST replay under `v0.1.0`.

Two binaries share a config directory during the rewrite, so a journal written
by one must replay under the other (from
docs/design/0.2.0-requirement-tradeoffs.md, high). A reviewer holds this by
comparing the journal `v0.2.0` writes against the format
`internal/rollback/rollback.go` recorded, because nothing can run `v0.1.0` since
the Go tree and the compatibility corpus were deleted (from CLAUDE.md
parity_is_the_contract, high).

Added during onboarding from the research on thin requirements, as the reverse
half of [REQ-2116]; the shipped journal reproduces the `v0.1.0` field names.
