---
id: REQ-2304
artifact: requirement
topic: starlark
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2304

`@name` alone MUST resolve to the module's root `init.star`.

The alternative was to require the extension, or to append `.star` in place of
`/init.star`, which lost because every stdlib component is a directory with an
`init.star`, so either alternative finds none of them; revisit if parity is
abandoned (from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-STAR-021` in `docs/spec/starlark.md`, its second obligation.
