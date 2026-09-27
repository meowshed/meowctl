---
id: REQ-3018
artifact: requirement
topic: engine
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3018

A bare name in the `after` list of a module's component MUST resolve inside that
module, as `<module>//components/<name>`.

A module's components refer to each other by name: `@dotmeow`'s root component
says `after = ["fish-config"]`, meaning the one beside it (from
docs/spec/engine.md, high). Resolving that name against the configuration looks
for a file the user never wrote, and fails on every configuration that uses an
aggregate module (from docs/spec/engine.md, high). The alternative was requiring
a module's components to name each other in full, which lost because every
component would carry its own module's name, which it cannot know when the
module is replaced or renamed (from docs/design/0.2.0-requirement-tradeoffs.md,
high). The trade-off table names no condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-ENGINE-018` in `docs/spec/engine.md`.
