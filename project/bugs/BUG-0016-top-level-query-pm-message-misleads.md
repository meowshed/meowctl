---
id: BUG-0016
artifact: bug
status: approved
severity: minor
violates: REQ-2343
found: 2026-09-27
revised: 2026-09-27
issue: 152
---

# A top-level `query_pm` blames the evaluation instead of saying it works only in a hook

`query_pm` at the top level of a component runs against `NoPackageManagers`,
whose message doesn't tell the author where the call does work.

## Reproduction

The research ran the debug binary `target/debug/meowctl`, built from revision
`7e8cabe`, on macOS in a scratch directory, with `HOME`, `XDG_CACHE_HOME` and
`--config` pointed there. The research put `query_pm("brew")` at the top level
of a component and ran `meowctl apply`.

## What the system does

It failed with `query_pm("brew") needs a package-manager registry and this
evaluation has none` and exited 3
(`crates/meowctl-starlark/src/evaluator.rs:62-75`).

## What it should do, and why

[REQ-2343] says the error says that `query_pm` works only inside a hook. The fix
rewords the message in `evaluator.rs:66-70`.

## Triage

A requirement covers it, so the fix enters at implement. It is minor because the
exit code is already right and only the wording misleads.

## Closed by

Open.
