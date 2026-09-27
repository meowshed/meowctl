---
id: REQ-1606
artifact: requirement
topic: exec
class: non-functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1606

Resolving a name on Windows MUST also try the extensions `PATHEXT` names, in the
order it names them, falling back to `.COM;.EXE;.BAT;.CMD` when it is unset.

A component asks for `git`, and on Windows the file is `git.exe` (from
docs/spec/exec.md, high). Every shell there resolves this way and a user who has
put a `.ps1` on `PATHEXT` expects it to count (from docs/spec/exec.md, high). On
every other platform the name is the file, and nothing is appended (from
docs/spec/exec.md, high). The alternative was to try the bare name only, which
lost because every program on Windows would report as missing, since the file is
`git.exe` (from docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off
table names nothing that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Its tests run only on a Windows host: CI runs on Linux only and `mise run
check-windows` only compiles, so the evidence still missing is a Windows CI job
(from CLAUDE.md, high).

Migrated from `R-EXEC-006` in `docs/spec/exec.md`.
