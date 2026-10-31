---
id: TSK-0043
artifact: task
status: done
revised: 2026-10-31
bug: BUG-0044
closes: [REQ-2454]
---

# Re-resolve a module whose pin differs from the lock

`Syncer::sync` takes a module's locked version only when `deps.mod` doesn't
pin a different exact version, so editing a pin and syncing moves the lock.

## Acceptance criteria

1. Given a lock with stdlib 0.2.16 and a manifest pinning stdlib 0.2.17, when
   an ordinary sync runs, then stdlib resolves to 0.2.17. Closed by: a test in
   `crates/meowctl-module/tests/syncing.rs` naming REQ-2454.
2. Given a lock with stdlib 0.2.16 and a manifest asking for `latest`, when an
   ordinary sync runs, then stdlib stays at 0.2.16. Closed by:
   `a_locked_version_is_kept_even_when_a_newer_one_is_published`, citing
   REQ-2454.

## What to do

In `lock_registry` in `crates/meowctl-module/src/sync.rs`, drop the locked
version when the dependency names an exact version that differs from it. Bring
SPC-2400 and the citations of REQ-2450 in code and tests to REQ-2454.

## Depends on

None.

## Left alone

A transitive module keeps whatever version the root's manifest selects, as it
does today.

## Evidence

Criterion 1 is closed by `a_changed_pin_moves_the_lock`, which failed before the
fix and passes after it, and criterion 2 by
`a_locked_version_is_kept_even_when_a_newer_one_is_published`, which passes on
both sides. `mise run fmt-check`, `mise run check`, `mise run test` (608 passed,
3 skipped) and `mise run build` exit 0. No end-to-end `dep sync` against a real
configuration ran for this change.
