---
id: REQ-1020
artifact: requirement
topic: common
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1020

The config directory MUST resolve to `$MEOWCTL_CONFIG` when set, then
`$XDG_CONFIG_HOME/meowctl`, then `~/.config/meowctl`, in that order. The
`--config` flag overrides all of them; see [REQ-3404].

`$MEOWCTL_CONFIG` is a deliberate change, because `internal/cli/config.go`
doesn't consult it and the compat corpus needs to point two binaries at one
configuration without a flag on every invocation (from docs/spec/common.md,
high). The alternative was resolving exactly as `v0.1.0` does, which lost
because the corpus needs two binaries on one config without a flag per call;
revisit it if the corpus stops needing it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-COMMON-020` in `docs/spec/common.md`.
