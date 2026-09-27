---
id: REQ-2843
artifact: requirement
topic: ctx
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2843

`run` MUST NOT fail on a non-zero exit; see REQ-1604.

A non-zero exit is information a hook reads, as when `brew list` exits 1 for a
package that isn't installed, and failing on it would make every interrogation
hook catch the error (from docs/design/0.2.0-requirement-tradeoffs.md:86 and
https://github.com/meowshed/meowctl/pull/58, high). It follows from REQ-1604
(from docs/design/0.2.0-requirement-tradeoffs.md:300, high).

Migrated from `R-CTX-043` in `docs/spec/ctx.md`.
