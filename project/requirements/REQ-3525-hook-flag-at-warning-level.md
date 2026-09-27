---
id: REQ-3525
artifact: requirement
topic: cli
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3525

`status` and `doctor` MUST use the warning level, which is what a reader scans
for.

`v0.1.0` writes the `status` line to stderr so that `meowctl status | ...` stays
clean, and that does not carry over: a sink owns its destination and writes one
stream, and splitting one command across two would make the live region and the
JSON stream disagree about what the command said (from docs/spec/cli.md, high).
The line is a warning in the stream instead, inside the terminal output
carve-out; see [REQ-3232] (from docs/spec/cli.md, high). The alternative was to
say nothing and let the user find the file, which lost because a shell silently
missing its integration is exactly what nobody thinks to look for, and the
trade-off record names no condition that reverses it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CLI-064` in `docs/spec/cli.md`, its fourth obligation.
