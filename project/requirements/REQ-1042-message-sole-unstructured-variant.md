---
id: REQ-1042
artifact: requirement
topic: common
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1042

`Message` MUST be the only variant carrying unstructured text.

The alternative was allowing free text anywhere, which lost because it is how
`Writer.Log` made `v0.1.0`'s output unstructurable; revisit it if a case appears
that genuinely has no structure and no severity (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-COMMON-042` in `docs/spec/common.md`, its first obligation.
