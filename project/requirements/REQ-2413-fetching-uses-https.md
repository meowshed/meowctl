---
id: REQ-2413
artifact: requirement
topic: module
class: non-functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2413

Fetching MUST use HTTPS.

`v0.1.0` fetches a tarball over pure-Go HTTPS (from docs/spec/module.md, high).
The alternative was to shell out to `git`, which lost because a machine being
bootstrapped may not have `git`, which is exactly when `init <repo>` runs;
revisit if `git` becomes a hard prerequisite anyway (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-MODULE-013` in `docs/spec/module.md`, its first obligation.
