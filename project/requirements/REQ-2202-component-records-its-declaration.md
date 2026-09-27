---
id: REQ-2202
artifact: requirement
topic: starlark
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2202

`component(name, after = [], **kwargs)` MUST record a declaration carrying the
name, the ordering hints, and any extra keyword arguments, in declaration order.
`name` may be positional or named and is required; `after` is named only and
must be a list of strings.

The alternative was to take `after` as a dependency, which lost because `after`
orders without pulling in, and conflating the two changes which components run;
revisit if parity is abandoned (from docs/design/0.2.0-requirement-tradeoffs.md,
high). `name` is positional or named because `component(name = ...)` is how a
generated configuration writes it (from
crates/meowctl-starlark/tests/evaluation.rs:372-373, high).

Migrated from `R-STAR-002` in `docs/spec/starlark.md`.
