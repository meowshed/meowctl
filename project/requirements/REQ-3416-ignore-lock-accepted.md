---
id: REQ-3416
artifact: requirement
topic: cli
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3416

`--ignore-lock` MUST be accepted.

Dropping the flag would break a script that passes it (from docs/spec/cli.md,
high). The alternative was to drop the flag or implement it, which lost because
dropping it breaks a script that passes it and implementing it is behaviour
neither binary has ever had; revisit if somebody needs to resolve without the
lock (from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CLI-016` in `docs/spec/cli.md`, its first obligation.
