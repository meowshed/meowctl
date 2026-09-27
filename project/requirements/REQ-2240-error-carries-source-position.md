---
id: REQ-2240
artifact: requirement
topic: starlark
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2240

An evaluation error MUST carry the file, the line, the column, and the source
text of the offending line, so `meowctl-cli` can render a diagnostic that points
at it; see [REQ-1032] and [REQ-3431].

M0 found that `Error::span()` returns a `FileSpan` of the form
`broken.star:2:1-12`, so the library satisfies this requirement without work in
`meowctl-starlark` (from docs/spec/starlark.md, high). The alternative was a
message with a file name, which is what `v0.1.0` gives, which lost because M0
showed the library gives spans for free, and the table names no condition that
would reverse it (from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-STAR-040` in `docs/spec/starlark.md`.
