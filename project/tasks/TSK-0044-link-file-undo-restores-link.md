---
id: TSK-0044
artifact: task
status: approved
revised: 2026-10-31
realises: ADR-0052
closes: [REQ-2033]
---

# Journal and replay the prior target of a replaced link

A `link_file` record over a replaced symlink carries `prior_target`, and
reading it back gives an inverse that re-creates that symlink.

## Acceptance criteria

1. Given a `link_file` whose inverse is the prior symlink, when the journal
   writes it, then the record has `"was_backed_up": false` and the prior
   target under `prior_target`. Closed by:
   `every_journalled_variant_writes_the_payload_v0_1_0_reads`, extended.
2. Given that record, when it is read back, then the inverse is `Op::Symlink`
   with the prior target. Closed by: a test in
   `crates/meowctl-ops/tests/journal.rs` naming REQ-2033.
3. Given a `link_file` record with no `prior_target`, as `v0.1.0` writes it,
   when it is read back, then the inverse removes the link. Closed by: the same
   test.

## What to do

In `crates/meowctl-ops/src/journal.rs`, write `prior_target` for a `LinkFile`
whose inverse is `Op::Symlink`, and read it back into `Op::Symlink`. Bring
SPC-2000 and the citations of REQ-2015 to REQ-2033.

## Depends on

None.

## Left alone

BUG-0001, the journal that is never truncated, is a defect of its own.

## Evidence

Not yet.
