---
id: REQ-3542
artifact: requirement
topic: cli
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3542

When `hook` can't record or clear `.hook-error`, it MUST say so on stderr.

The exit code has to stay 0 [REQ-3522], so one line on stderr naming the path is
the only channel left, and stderr doesn't corrupt the shell code `eval` reads
from stdout.

Added during onboarding from the failure-path research; the shipped command
drops a failed write silently and turns a failed removal into exit 3 (from
crates/meowctl-cli/src/run/commands.rs:541-558, high).
