---
id: REQ-2300
artifact: requirement
topic: starlark
class: non-functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2300

The predeclared set MUST NOT change: no name is added to it and none is removed
from it.

The Starlark surface is what users touch, and `v0.2.0` changes the
implementation underneath it from `go.starlark.net` to `starlark-rust` without
changing what a component file may say (from docs/spec/starlark.md, high). The
alternative was to add a builtin that would be useful, which lost because every
addition is surface that `v0.1.0` cannot evaluate, so a component using it stops
working on the old binary; revisit if the Go tree is deleted and 0.3.0 opens the
API (from docs/design/0.2.0-requirement-tradeoffs.md, high). The Go tree was
deleted at the cutover, which meets the first half of that condition without
reversing the requirement on its own (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-STAR-001` in `docs/spec/starlark.md`, its second obligation.

Onboarding reworded the source's "No ... MUST" to MUST NOT, because the record's
check refuses a negated subject; the meaning is unchanged.
