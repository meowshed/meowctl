---
id: REQ-1414
artifact: requirement
topic: fs
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1414

The three implementations MUST be interchangeable behind a trait object.
`meowctl-cli` constructs one from the flags and passes it down; no other crate
chooses.

The alternative was generics in place of a trait object, which lost because that
monomorphises the engine over three filesystems for no measured gain; revisit it
if a measurement shows the virtual call costs something (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-FS-014` in `docs/spec/fs.md`.
