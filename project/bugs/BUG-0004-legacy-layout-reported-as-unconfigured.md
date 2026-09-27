---
id: BUG-0004
artifact: bug
status: approved
severity: major
violates: REQ-1204
found: 2026-09-27
revised: 2026-09-27
issue: 140
---

# `require_configured` hides the legacy-layout message and I/O errors behind "run `meowctl init`"

Every error from `Layout::check` becomes `CliError::NotConfigured`, so a user
with the pre-rename layout is told to run `meowctl init` in a directory that
holds a configuration.

## Reproduction

The onboarding research inferred this from the code at revision `7e8cabe` and
didn't run it. At revision `7e8cabe`, put `meowctl.star` and no `init.star` in
the configuration directory and run `meowctl apply`.

## What the system does

`require_configured` maps every `Layout::check` error to `NotConfigured`
(`crates/meowctl-cli/src/run/commands.rs:150-156`). That includes
`ConfigError::LegacyLayout`, whose message carries the three `mv` commands that
migrate the layout (`crates/meowctl-config/src/layout.rs:132-152`), and any I/O
error.

## What it should do, and why

[REQ-1204] says the pre-rename layout is reported with the three `mv` commands.
The fix maps only the missing entry file to `NotConfigured` and passes
`LegacyLayout` and I/O errors through.

## Triage

A requirement covers it, so the fix enters at implement. It is major because
every `v0.1.0` user still on the pre-rename layout gets advice that doesn't
apply and never sees the commands that would fix it.

## Closed by

Open.
