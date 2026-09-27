---
id: REQ-3529
artifact: requirement
topic: cli
class: non-functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3529

`self-update` MUST refuse to proceed when `checksums.sri` is absent.

Treating a release with no checksums as one that needs no verification is the
weakness the self-update requirements exist to close (from docs/spec/cli.md,
high). The alternative was to update from a release with no checksums,
unverified, which lost because a release nothing can be verified against is not
one that needs no verification, and that reading is how the check gets skipped;
the trade-off record names no condition that reverses it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CLI-071` in `docs/spec/cli.md`, its second obligation.
