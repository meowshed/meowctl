---
id: BUG-0044
artifact: bug
status: approved
severity: major
enters: requirements
found: 2026-10-25
revised: 2026-10-31
---

# `dep sync` keeps the locked version after `deps.mod` pins a different one

When `deps.mod` changes `dep(name = "dotmeow", version = "0.3.32")` to
`0.3.33`, `meowctl dep sync` keeps 0.3.32 in `deps.lock`, so the new pin never
takes effect until the lock is deleted by hand.

## Reproduction

meowctl 0.2.0 at commit `9feca9c`, aarch64-apple-darwin, with a `deps.lock`
that records dotmeow 0.3.32 and a registry that lists 0.3.33. Edit `deps.mod`
to pin 0.3.33, then run:

```bash
meowctl dep sync --format plain
```

## What the system does

```text
  i dotmeow 0.3.32
  i stdlib 0.2.17
```

`deps.lock` still says 0.3.32. `meowctl dep add dotmeow --version 0.3.33`
refuses because dotmeow is already declared, and `meowctl dep upgrade` moves to
the latest the registry lists, which isn't always the version wanted. Moving
`deps.lock` aside and syncing again resolves 0.3.33.

## What it should do, and why

`v0.1.0`'s `syncOneDep` resolves the version `deps.mod` names on every sync, so
a changed pin takes effect. The parity principle in `CLAUDE.md` keeps that.
REQ-2450 says a locked module is used at the locked version and selection runs
only when it is absent, so the code follows a requirement that dropped this
case.

## Triage

The requirement is wrong for an exact pin, so this enters at requirements:
REQ-2454 supersedes REQ-2450. It is major because the documented way to move a
dependency, editing `deps.mod` and syncing, silently does nothing.

## Closed by

`a_changed_pin_moves_the_lock` in `crates/meowctl-module/tests/syncing.rs`
holds the behaviour as a regression check.

## Tasks

- [x] T-001 TSK-0043 re-resolve a module whose pin differs from the lock, in
  `crates/meowctl-module/src/sync.rs` (the pin test passes, and every stage
  exits 0)
