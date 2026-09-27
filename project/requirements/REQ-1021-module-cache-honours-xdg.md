---
id: REQ-1021
artifact: requirement
topic: common
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1021

The module cache directory MUST resolve to `$XDG_CACHE_HOME/meowctl/modules`,
falling back to `~/.cache/meowctl/modules`.

`cacheDir` in `internal/cli/sync.go` uses the fallback unconditionally and never
reads `$XDG_CACHE_HOME`, so a machine that relocates its cache still has
`v0.1.0` writing to the default (from docs/spec/common.md, high). Honouring the
variable is a deliberate change, and it is why the compatibility corpus can't
share a cache with `v0.1.0` through that variable (from docs/spec/common.md,
high). The alternative was hard-coding `~/.cache`, as `cacheDir` does, which
lost because XDG is already honoured for config, and splitting the convention
means a machine that relocated its cache still has meowctl writing to the
default, and the record expects nothing to reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-COMMON-021` in `docs/spec/common.md`.
