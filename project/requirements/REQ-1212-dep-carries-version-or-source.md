---
id: REQ-1212
artifact: requirement
topic: config
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1212

A `dep()` MUST carry a version or a source, never both and never neither, and a
`replace()` a path or a source on the same terms.

`internal/modfile/modfile.go` refuses both at parse time, with "version and
source are mutually exclusive", "exactly one of version or source is required",
and the same pair for `replace` (from docs/spec/config.md, high). An earlier
draft enforced the rule only in `meowctl dep add`, so a hand-written `deps.mod`
carrying both parsed, and the module then resolved from whichever field the
resolver reached for first (from docs/spec/config.md, high). [REQ-1210] merged
the separate `dep()` grammars of `deps.mod` and `init.star`, and the validation
had to move with the merge rather than be dropped in it (from
docs/spec/config.md, high). The alternative was allowing both and preferring
one, which lost because a `dep()` with both is ambiguous about where it came
from and the lock can't say, and the record expects nothing to reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CONFIG-012` in `docs/spec/config.md`, its first obligation.
