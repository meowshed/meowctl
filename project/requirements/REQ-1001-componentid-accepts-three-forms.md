---
id: REQ-1001
artifact: requirement
topic: common
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1001

A `ComponentId` MUST be constructible from the three forms `v0.1.0` accepts: a
bare name (`neovim`), a registry-qualified path (`@stdlib//components/zsh`), and
a GitHub-qualified path (`github.com/owner/repo//components/zsh`).

The alternative was one string type for a component, which lost because the
three syntaxes have different resolution rules and a string can't say which it
holds, and the record says it never reverses, because the three forms are in
every user's `init.star` (from docs/design/0.2.0-requirement-tradeoffs.md,
high).

Migrated from `R-COMMON-001` in `docs/spec/common.md`, its first obligation.
