---
id: REQ-1106
artifact: requirement
topic: common
class: non-functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1106

Neither `$HOME` nor `$USERPROFILE` being set MUST be an error rather than a
guess.

`os.UserHomeDir`, which `v0.1.0` calls, reads `$USERPROFILE` on Windows and
`$HOME` everywhere else (from docs/spec/common.md, high). Reading only `$HOME`
makes every invocation on Windows fail before it does anything, which is what
the test matrix caught once a test ran the binary rather than a function (from
docs/spec/common.md, high). The alternative was reading `$HOME` everywhere,
which lost because Windows doesn't set it, so every invocation there fails
before it does anything, and the record expects nothing to reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-COMMON-023` in `docs/spec/common.md`, its second obligation.
