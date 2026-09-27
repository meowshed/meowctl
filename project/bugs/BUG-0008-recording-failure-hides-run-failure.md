---
id: BUG-0008
artifact: bug
status: approved
severity: minor
found: 2026-09-27
revised: 2026-09-27
issue: 144
---

# A failure recording packages after a failed run replaces the run's failure

After a failed run, `apply` records local declarations and packages before it
checks the run's failure, so an error there replaces the run's failure and
changes the exit code from 1 to 3.

## Reproduction

The onboarding research inferred this from the code at revision `7e8cabe` and
didn't run it. At revision `7e8cabe`, make a hook fail and make `pkgs.lock`
unwritable, then run `meowctl apply`.

## What the system does

`local_declarations(session, &loader)?` and `record_packages(...)?` run before
`report.failure` is checked (`crates/meowctl-cli/src/run/commands.rs:492-498`).
If either fails, its error is what `apply` returns.

## What it should do, and why

The run's failure should be reported first and the recording failure beside it,
keeping the general code. No requirement covers the order yet, which is a gap
for the requirements step.

## Triage

No requirement covers it, so it routes to the requirements step. It is minor
because the component's own failure is still shown by its `ComponentFinished`
event; only the final line and the exit code change.

## Closed by

Open.
