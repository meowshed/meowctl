---
id: REQ-2820
artifact: requirement
topic: ctx
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2820

`run(cmd, args = [], env = {}, cwd = None, interactive = False)` MUST return a
value carrying `stdout`, `stderr`, and `exit_code`, under those names.

Component code branches on `exit_code`, so the shape is part of the API (from
docs/spec/ctx.md, high). The alternatives were to return a tuple or to raise on
failure, which lost because components index `stdout` and `exit_code` by name
(from docs/design/0.2.0-requirement-tradeoffs.md, high). Revisit it if parity
with `v0.1.0` is abandoned (from docs/design/0.2.0-requirement-tradeoffs.md,
high).

Migrated from `R-CTX-020` in `docs/spec/ctx.md`.
