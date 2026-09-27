---
id: REQ-2704
artifact: requirement
topic: pm
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2704

The message for a handler function that raises MUST name both the declaring
component and the handler component.

The alternative was to fail the handler component, which lost because the user
would look at the standard library in place of their own declaration (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names
nothing that would reverse it (from docs/design/0.2.0-requirement-tradeoffs.md,
high).

Migrated from `R-PM-031` in `docs/spec/pm.md`, its second obligation.
