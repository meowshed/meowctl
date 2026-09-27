---
id: REQ-3410
artifact: requirement
topic: cli
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3410

The `FileSystem`, the `Executor`, and the sink MUST be constructed here, once,
from the parsed flags, and passed down. No crate below constructs one; see
[REQ-1414].

Here is the `meowctl-cli` crate (from docs/spec/cli.md, high). The alternative
was to construct effects where they are used, which lost because a crate below
would then know about flags, which is how `internal/cli` became the god-package,
and the trade-off record names no condition that reverses it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CLI-010` in `docs/spec/cli.md`.
