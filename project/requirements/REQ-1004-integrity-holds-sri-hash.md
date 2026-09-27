---
id: REQ-1004
artifact: requirement
topic: common
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1004

An `Integrity` MUST hold a W3C Subresource Integrity hash in the form
`sha384-<base64>`.

This is what `deps.lock` stores; see [REQ-1220] (from docs/spec/common.md,
high). The alternative was storing integrity as a plain string, which lost
because an unvalidated hash compares unequal forever and looks like tampering;
revisit it if Subresource Integrity is replaced by another hash format (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-COMMON-004` in `docs/spec/common.md`, its first obligation.
