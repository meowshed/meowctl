---
id: REQ-1002
artifact: requirement
topic: common
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1002

A `ComponentId` MUST expose the module key its source belongs to: `dotmeow` for
`@dotmeow//path`, `github.com/o/r` for `github.com/o/r//path`, and nothing for a
bare name.

This is the key used to look an entry up in a lock file, and
`internal/cli/apply.go`'s `moduleKeyFromComponentURL` is the behaviour to
reproduce (from docs/spec/common.md, high). The alternative was computing the
module key at each call site, which lost because `apply.go` computes it once and
four callers re-derive it, and each is a place to get `//` splitting wrong;
revisit it if the lock stops being keyed by module (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-COMMON-002` in `docs/spec/common.md`.
