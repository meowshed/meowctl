---
id: REQ-1222
artifact: requirement
topic: config
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1222

`meta` MUST record the version that wrote the file and an RFC 3339 timestamp.

`v0.1.0` declares both fields and sets neither, so every lock it has ever
written carries two empty strings (from docs/spec/config.md, high). Filling them
is a deliberate change, and it is the one place a user can see which binary last
touched their lock, which matters most during a cutover, when two of them share
a configuration directory (from docs/spec/config.md, high). The alternative was
leaving both fields empty, as `v0.1.0` does, which lost because during a cutover
two binaries share a lock, and which one wrote it is the first thing anybody
asks; revisit it if the cutover is over and nothing reads it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CONFIG-022` in `docs/spec/config.md`.
