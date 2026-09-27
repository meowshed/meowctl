---
id: REQ-3014
artifact: requirement
topic: engine
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3014

A component's `after` list MUST be read from both the `component()` declaration
and the component file's own top-level `after` global.

A component declaring `after = ["@stdlib//components/apt"]` is relying on apt
being installed, and requiring it to be named twice would make every component
file a change to `init.star` as well (from docs/spec/engine.md, high).
`expandWithFileDeps` in `v0.1.0` does this, and an earlier draft that said
`after` orders without depending and ignores an unknown name was wrong (from
docs/spec/engine.md, high). The alternative was reading `after` only from
`component()` and ignoring an unknown name, which lost because a component that
names a dependency is relying on it, and requiring it twice makes every
component file a change to `init.star` too (from
docs/design/0.2.0-requirement-tradeoffs.md, high). Nothing reverses it, because
it is what `v0.1.0` does (from docs/design/0.2.0-requirement-tradeoffs.md,
high).

Migrated from `R-ENGINE-014` in `docs/spec/engine.md`, its first obligation.
