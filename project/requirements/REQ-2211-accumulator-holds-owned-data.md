---
id: REQ-2211
artifact: requirement
topic: starlark
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: static
---

# REQ-2211

The accumulator MUST hold owned data.

No Starlark value may outlive the evaluation that produced it, which is a
constraint `go.starlark.net` did not impose and `starlark-rust` does (from
docs/spec/starlark.md, high). M0 found that `Module::with_temp_heap` scopes a
module and its heap to a closure, so the compiler enforces this requirement and
the evaluation boundary is where declarations become owned data (from
docs/spec/starlark.md, high). The alternative was to hold Starlark values, which
lost because M0 showed the heap is closure-scoped and they cannot escape; the
table says nothing reverses it, since the compiler enforces it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-STAR-011` in `docs/spec/starlark.md`.
