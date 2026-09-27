---
id: REQ-3527
artifact: requirement
topic: cli
class: non-functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3527

`self-update` MUST check the integrity of what it downloaded against a checksum
the release publishes.

[REQ-1005] and [REQ-2430] are the rule everything else meowctl downloads already
follows (from docs/spec/cli.md, high). `v0.1.0` does not verify: `runSelfUpdate`
fetches a release asset over HTTPS and renames it over `os.Executable()` with no
check of any kind, so anything that can answer for the download URL replaces the
user's `meowctl` (from docs/spec/cli.md, high). This is the one place the parity
constraint does not reach, because reproducing it would ship the weakness on
purpose in a binary that already knows how to verify a download (from
docs/spec/cli.md, high). The alternative was to reproduce `v0.1.0`, which lost
for that reason, and the trade-off record names no condition that reverses it
(from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CLI-070` in `docs/spec/cli.md`, its second obligation.
