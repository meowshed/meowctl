---
id: REQ-2813
artifact: requirement
topic: ctx
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2813

Every mutating method MUST go through an `Op`.

No method may call `FileSystem` directly, because an effect that is not an `Op`
has no inverse; see REQ-2002 (from docs/spec/ctx.md, high). The alternative was
to let a method call `FileSystem`, which lost because that is an effect with no
inverse, a defect by construction (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names
nothing that would reverse it (from docs/design/0.2.0-requirement-tradeoffs.md,
high).

Migrated from `R-CTX-013` in `docs/spec/ctx.md`.
