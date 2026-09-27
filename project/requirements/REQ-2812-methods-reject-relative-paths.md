---
id: REQ-2812
artifact: requirement
topic: ctx
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2812

Every method MUST reject a relative path.

The alternative was to resolve a relative path against the component directory,
which lost because it is plausible and different from what `v0.1.0` does (from
docs/design/0.2.0-requirement-tradeoffs.md, high). Revisit it if parity with
`v0.1.0` is abandoned (from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CTX-012` in `docs/spec/ctx.md`, its first obligation.
