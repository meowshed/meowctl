---
id: REQ-2024
artifact: requirement
topic: ops
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2024

Replay MUST report one of three outcomes: every inverse applied, some applied,
or none.

These map to the `ok`, `partial`, and `failed` values `state.toml` records; see
REQ-1244 (from docs/spec/ops.md, high). The alternative was a boolean, which
lost because `partial` is the outcome a user most needs to see, and `state.toml`
already has three values (from docs/design/0.2.0-requirement-tradeoffs.md,
high). Revisit it if parity with `v0.1.0` is abandoned (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-OPS-024` in `docs/spec/ops.md`.
