---
id: REQ-1010
artifact: requirement
topic: common
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1010

`Phase` MUST have exactly the thirteen variants `v0.1.0` defines in
`internal/lifecycle/runner.go`: `install_check`, `install`, `install_configure`,
`update`, `upgrade_check`, `upgrade`, `upgrade_configure`, `uninstall_check`,
`uninstall`, `uninstall_cleanup`, `shell`, `login`, and `verify`.

The names are the hook names a component author writes, and they appear in
`state.toml` (from docs/spec/common.md, high). The alternative was fewer,
coarser phases, which lost because thirteen is what components already export as
hook names, and merging any breaks files; revisit it if hook names change, which
breaks parity (from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-COMMON-010` in `docs/spec/common.md`, its first obligation.
