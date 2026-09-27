---
id: BUG-0007
artifact: bug
status: approved
severity: major
violates: REQ-3140
found: 2026-09-27
revised: 2026-09-27
issue: 143
---

# An `init.star` or `local.star` that exists but can't be read is skipped as if absent

`declarations()` treats every read error as absence, so a run goes ahead without
the machine's local components when `local.star` can't be read.

## Reproduction

The onboarding research inferred this from the code at `main` as #97 left it and
didn't run it. At `main` as #97 left it, run `chmod 000 local.star` in a
configuration and run `meowctl apply --dry-run`.

## What the system does

`let Ok(bytes) = session.fs.read(&path) else { continue; }`
(`crates/meowctl-cli/src/run/commands.rs:264-266`) skips the file, and
`local_declarations` repeats the pattern (`commands.rs:299-304`). The plan omits
every component `local.star` declares, and nothing is printed.

## What it should do, and why

[REQ-3140] says a `local.star` that exists and can't be read stops the command
before any phase runs. The fix treats only `FsError::NotFound` as absence.

## Triage

A requirement covers it, so the fix enters at implement. It is major because the
shared configuration is applied without the machine overlay, and nothing tells
the user.

## Closed by

Open.
