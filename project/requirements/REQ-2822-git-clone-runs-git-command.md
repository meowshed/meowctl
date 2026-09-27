---
id: REQ-2822
artifact: requirement
topic: ctx
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2822

`git_clone(url, dst, ref = None)` MUST build a `git` command and run it through
the `Executor`, rather than implementing a fetch.

The alternative was to implement a git client, which lost because a clone is one
command and a client is a project (from
docs/design/0.2.0-requirement-tradeoffs.md, high). Revisit it if `git` stops
being available where clones happen (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CTX-022` in `docs/spec/ctx.md`.
