---
id: REQ-2221
artifact: requirement
topic: starlark
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2221

A registry URL whose final segment contains no `.` MUST resolve to
`<path>/init.star` inside the module. A path whose final segment does contain a
`.` is taken as written.

So `@stdlib//components/apt` resolves to `components/apt/init.star`, which is
how every stdlib component is laid out, and `@stdlib//components/apt.star`
resolves to that file (from docs/spec/starlark.md, high). The doc comment on
`RegistryLoader` in `internal/starlark/loader/registry.go` says the two are
equivalent, but `parseRegistryURL`, which is what runs, appends `/init.star`
(from docs/spec/starlark.md, high). The alternative was to require the
extension, or to append `.star` in place of `/init.star`, which lost because
every stdlib component is a directory with an `init.star`, so either alternative
finds none of them; revisit if parity is abandoned (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-STAR-021` in `docs/spec/starlark.md`, its first obligation.
