---
id: REQ-2401
artifact: requirement
topic: module
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2401

Version selection MUST be Minimal Version Selection over the dependency graph,
selecting for each module the maximum version any path requires.

`internal/mvs/mvs.go` is the algorithm (from docs/spec/module.md, high). The
alternative was a SAT solver, or newest-wins, which lost because MVS gives a
reproducible answer a user can predict by reading their manifests, where a
solver gives a better answer nobody can predict; revisit if a constraint appears
that MVS cannot express (from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-MODULE-001` in `docs/spec/module.md`.
