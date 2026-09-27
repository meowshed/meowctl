---
id: REQ-2206
artifact: requirement
topic: starlark
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2206

`dep`, `module`, and `replace` MUST read every manifest meowctl encounters: a
`deps.mod`, which meowctl writes, and a `MODULE.meow`, which a module author
writes.

One grammar serves both files; see REQ-1211 (from docs/spec/starlark.md, high).
`v0.1.0` has three readers: the evaluator's builtins read `init.star`,
`internal/modfile` has its own `module`, `dep` and `replace` for `deps.mod`, and
`parseModuleMeow*` inside the loader has a third pair for `MODULE.meow`, whose
`module()` ignores its arguments entirely (from docs/spec/starlark.md, high).
The alternative was a separate `deps.mod` parser, which lost because of the
reasons the trade-off row for REQ-1210 gives, and the table names no condition
that would reverse it (from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-STAR-006` in `docs/spec/starlark.md`, its first obligation.
