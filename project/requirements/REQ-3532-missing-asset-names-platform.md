---
id: REQ-3532
artifact: requirement
topic: cli
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3532

A release with no asset for this platform MUST say which platform was looked for
and where the release is, rather than reporting a network failure.

The alternative was to report a missing asset as a fetch that failed, which lost
because a user on a platform nobody built for would retry a network that is
fine, and the trade-off record names no condition that reverses it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CLI-073` in `docs/spec/cli.md`, its second obligation.
