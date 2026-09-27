---
id: REQ-2303
artifact: requirement
topic: starlark
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2303

`module(name, version = "", compat = None)` MUST accept `compat` and record it.

The `v0.1.0` reader for `MODULE.meow` ignores the arguments of `module()`, which
is why every `MODULE.meow` in the meowshed organisation carries `compat` while
the other two readers would reject it (from docs/spec/starlark.md, high). The
builtin records `compat` without acting on it, because nothing reads it yet and
dropping it would leave a module declaring a newer schema indistinguishable from
one that declares none (from docs/spec/starlark.md, high). This is wider than
any single `v0.1.0` reader, so every manifest that parses today parses here
(from docs/spec/starlark.md, high). The alternative was a separate `deps.mod`
parser, which lost because of the reasons the trade-off row for REQ-1210 gives,
and the table names no condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-STAR-006` in `docs/spec/starlark.md`, its second obligation.
