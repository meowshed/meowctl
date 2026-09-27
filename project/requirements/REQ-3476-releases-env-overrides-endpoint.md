---
id: REQ-3476
artifact: requirement
topic: cli
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3476

`MEOWCTL_RELEASES` MUST override where the latest release is asked for.

The override is for a fork that publishes its own releases, and for the tests: a
release that does not exist yet cannot be asked of GitHub, and a `self-update`
that could only be checked against the real service would be checked once and
then not again (from docs/spec/cli.md, high). The alternative was to hard-code
the releases URL, which lost because a fork could not update from its own
releases and a release that does not exist yet cannot be tested against; the
trade-off record names no condition that reverses it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CLI-076` in `docs/spec/cli.md`.
