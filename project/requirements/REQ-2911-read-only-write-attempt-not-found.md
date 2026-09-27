---
id: REQ-2911
artifact: requirement
topic: ctx
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2911

A hook that tries to write in a read-only phase MUST get attribute-not-found
rather than a silent no-op.

The read-only phases are in REQ-1012 (from docs/spec/ctx.md, high). No `verify`,
`install_check`, `upgrade_check` or `uninstall_check` hook in `meowctl-stdlib`
or in `dotmeow` calls a mutating method, which was checked rather than assumed
(from docs/spec/ctx.md, high). The alternative was to leave every phase with the
full `ctx`, as `v0.1.0` does, which lost because a `verify` that writes is a
check with a side effect, and nothing in the standard library or in `dotmeow`
does it (from docs/design/0.2.0-requirement-tradeoffs.md, high). Revisit it if a
component needs to write during a check (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The choice is argued at
length in `docs/design/0.2.0-decisions.md` (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CTX-030` in `docs/spec/ctx.md`, its second obligation.
