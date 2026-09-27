---
id: REQ-3010
artifact: requirement
topic: engine
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3010

Components MUST be discovered by evaluating `init.star`, then merging
`local.star` when it exists, with a component declared in both counting once.

The alternative was requiring components in one file, which lost because
`local.star` is how one machine differs and is gitignored (from
docs/design/0.2.0-requirement-tradeoffs.md, high). Revisit it if parity is
abandoned (from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-ENGINE-010` in `docs/spec/engine.md`.
