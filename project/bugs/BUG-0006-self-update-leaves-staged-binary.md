---
id: BUG-0006
artifact: bug
status: approved
severity: minor
violates: REQ-3543
found: 2026-09-27
revised: 2026-09-27
issue: 142
---

# `self-update` leaves the staged binary behind when it can't make it executable

`put_in_place` removes the staged file when the rename fails and returns without
removing it when `set_executable` fails.

## Reproduction

The onboarding research inferred this from the code at `main` as #97 left it and
didn't run it. At `main` as #97 left it, make `set_executable` fail on the
staged file `.<name>-<pid>.new`, for example on a filesystem that refuses a mode
change, and run `meowctl self-update`.

## What the system does

A `.meowctl-<pid>.new` file of over 10 MB stays beside the binary
(`crates/meowctl-cli/src/update.rs:108-109`). The test
`the_staged_file_does_not_survive` (`update.rs:165`) says "whichever way it
went" but runs only the success path.

## What it should do, and why

[REQ-3543] says a replacement that fails at any step removes the staged file.

## Triage

A requirement covers it, so the fix enters at implement. It is minor because the
running binary is untouched and the leftover costs only disk space.

## Closed by

Open. The fix adds a `MemFs` test that fails the rename and the mode change.
