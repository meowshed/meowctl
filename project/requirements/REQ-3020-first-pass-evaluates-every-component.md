---
id: REQ-3020
artifact: requirement
topic: engine
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3020

Every component MUST be evaluated once to collect its globals and register
package-manager handlers, before any hook runs. See REQ-2603.

The alternative was registering handlers as components are reached, which lost
because ordering would then decide whether a `pkg()` resolves (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names no
condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-ENGINE-020` in `docs/spec/engine.md`.
