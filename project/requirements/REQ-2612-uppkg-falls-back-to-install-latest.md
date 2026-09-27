---
id: REQ-2612
artifact: requirement
topic: pm
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2612

When a handler exports no `update_pkg`, `uppkg()` MUST fall back to
`install_pkg(ctx, name, "latest", **kwargs)`.

`DispatchUpdate` in `internal/pkg/registry.go` is the fallback, and removing it
would break every handler that never defined an update path (from
docs/spec/pm.md, high). The alternative was to fail when `update_pkg` is absent,
which lost for that reason (from docs/design/0.2.0-requirement-tradeoffs.md,
high). The trade-off table names nothing that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-PM-012` in `docs/spec/pm.md`.
