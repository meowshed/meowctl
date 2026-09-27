---
id: REQ-1213
artifact: requirement
topic: config
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1213

Writing `deps.mod` MUST emit keyword arguments in the order `name`, then
`version` or `source`.

`v0.1.0` requires this order for its regular-expression rewriter, and here it is
required so the two binaries produce identical files (from docs/spec/config.md,
high). The alternative was writing whatever order is convenient, which lost
because two binaries writing the same file have to produce the same bytes;
revisit it if the Go tree is deleted (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CONFIG-013` in `docs/spec/config.md`.
