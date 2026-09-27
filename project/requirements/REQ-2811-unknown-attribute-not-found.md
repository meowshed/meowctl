---
id: REQ-2811
artifact: requirement
topic: ctx
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2811

An unknown attribute MUST report attribute-not-found rather than raise an
internal error, matching `Attr` returning `(nil, nil)`.

The alternative was to raise on an unknown attribute, which lost because
Starlark's `hasattr` would error in place of returning false (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names
nothing that would reverse it (from docs/design/0.2.0-requirement-tradeoffs.md,
high).

Migrated from `R-CTX-011` in `docs/spec/ctx.md`.
