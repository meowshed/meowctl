---
id: REQ-1032
artifact: requirement
topic: common
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1032

An error that has a source location MUST carry it as a span into the file it
came from, so `meowctl-cli` can render a diagnostic that points at the line.

`v0.1.0` couldn't do this, and its Starlark errors name a file and nothing more
(from docs/spec/common.md, high). The alternative was errors that carry a
message only, which lost because `v0.1.0` does that, and a Starlark mistake
reports a file and no line, and the record says it never reverses, because M0
showed the span is free (from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-COMMON-032` in `docs/spec/common.md`.
