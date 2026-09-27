---
id: REQ-2004
artifact: requirement
topic: ops
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2004

Applying an `Op` and then applying its inverse MUST restore the filesystem to
the state it had before.

`v0.1.0` fails this in three places, all found by writing that test: undoing a
`link_file` leaves the backup behind, undoing an `append_file` that created its
file leaves a zero-byte file, and undoing a `mkdir` removes one directory where
`mkdir -p` created a chain (from docs/spec/ops.md, high). Each is repaired here,
and each needed a field in the journal payload that `v0.1.0` ignores (from
docs/spec/ops.md, high). The alternative was to test only the risky inverses,
which lost because the failure is a variant added later whose inverse is subtly
wrong (from docs/design/0.2.0-requirement-tradeoffs.md, high). Revisit it if the
property test proves too slow to run per commit (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The choice is argued at
length in `docs/design/0.2.0-decisions.md` (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-OPS-004` in `docs/spec/ops.md`, its first obligation.
