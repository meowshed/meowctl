---
id: REQ-1030
artifact: requirement
topic: common
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1030

The error taxonomy MUST map onto the exit codes `v0.1.0` defines in
`internal/cli/errors.go`, and no others: 0 for success, 1 for a general error, 2
for a usage error, 3 for a configuration error (a malformed `init.star`, a
missing component, a legacy layout), and 4 for a module error (a fetch that
failed, an integrity hash that did not match).

The alternative was richer exit codes, which lost because scripts already branch
on these five, and adding codes changes behaviour silently; revisit it if a
caller needs to distinguish a case these merge (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-COMMON-030` in `docs/spec/common.md`.
