---
id: BUG-0015
artifact: bug
status: approved
severity: major
violates: REQ-2207
found: 2026-09-27
revised: 2026-09-27
issue:
---

# `platform()` carries no architecture attribute

`PlatformValue` exposes no architecture, though [REQ-2207] says `platform()`
carries one.

## Reproduction

The onboarding research inferred this from the code at revision `7e8cabe` and
didn't run it. At revision `7e8cabe`, read
`crates/meowctl-starlark/src/platform.rs:111-128`, or evaluate a component that
reads the architecture from `platform()`.

## What the system does

The struct has no architecture field, so reading it fails with
attribute-not-found at evaluation.

## What it should do, and why

[REQ-2207] says `platform()` returns the operating system, the architecture and,
on Linux, the distribution. The fix either adds the field or, if the owner
decides `v0.1.0` never had one, withdraws that part of [REQ-2207].

## Triage

A requirement covers it, so the fix enters at implement. It is major because the
Starlark surface is a parity contract, and a component that reads the field
fails to evaluate.

## Closed by

Open.
