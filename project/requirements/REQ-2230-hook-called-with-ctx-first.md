---
id: REQ-2230
artifact: requirement
topic: starlark
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2230

The evaluator MUST call a named global function in an evaluated file, passing
the `ctx` value first and whatever else the caller supplies after it,
positionally and by keyword.

A lifecycle hook takes `ctx` alone, while a package-manager handler takes
`install_pkg(ctx, name, version, **kwargs)`, so the "its single argument" an
earlier draft said would make REQ-2610 unimplementable (from
docs/spec/starlark.md, high). The alternative was to pass arguments positionally
by convention, which lost because the one argument is named in every component
and changing it breaks all of them; revisit if parity is abandoned (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-STAR-030` in `docs/spec/starlark.md`.
