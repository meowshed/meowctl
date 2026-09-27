---
id: REQ-1109
artifact: requirement
topic: common
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1109

`Message` MUST carry a severity.

The alternative was allowing free text anywhere, which lost because it is how
`Writer.Log` made `v0.1.0`'s output unstructurable; revisit it if a case appears
that genuinely has no structure and no severity (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-COMMON-042` in `docs/spec/common.md`, its second obligation.
