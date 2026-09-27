---
id: REQ-1023
artifact: requirement
topic: common
class: non-functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1023

The home directory MUST come from `$HOME`, and from `$USERPROFILE` when `$HOME`
is unset on Windows.

`os.UserHomeDir`, which `v0.1.0` calls, reads `$USERPROFILE` on Windows and
`$HOME` everywhere else (from docs/spec/common.md, high). Reading only `$HOME`
makes every invocation on Windows fail before it does anything, which is what
the test matrix caught once a test ran the binary rather than a function (from
docs/spec/common.md, high). The alternative was reading `$HOME` everywhere,
which lost because Windows doesn't set it, so every invocation there fails
before it does anything, and the record expects nothing to reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Its tests run only on a Windows host: CI runs on Linux only and `mise run
check-windows` only compiles, so the evidence still missing is a Windows CI job
(from CLAUDE.md, high).

Migrated from `R-COMMON-023` in `docs/spec/common.md`, its first obligation.
