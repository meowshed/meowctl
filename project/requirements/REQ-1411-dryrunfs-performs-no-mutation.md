---
id: REQ-1411
artifact: requirement
topic: fs
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1411

`DryRunFs` MUST perform no mutation.

A hook that reads a file it just wrote in a dry run has to see plausible content
(from docs/spec/fs.md, high). In `v0.1.0` a dry run is `if c.caps.DryRun`
repeated through every effectful method, so the guarantee that a dry run writes
nothing holds only as long as nobody forgets a branch, and somebody did (from
docs/spec/fs.md, high). The alternative was skipping effectful calls at the call
site, which lost because that is `v0.1.0`, and the branch was forgotten once
already, and the record expects nothing to reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-FS-011` in `docs/spec/fs.md`, its first obligation.
