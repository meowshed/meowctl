---
id: BUG-0017
artifact: bug
status: approved
severity: minor
violates: REQ-2704
found: 2026-09-27
revised: 2026-09-27
issue: 153
---

# A raising `interrogate` doesn't name the asking component in its message

`query_pm` passes the handler's error through unchanged, so its message carries
the handler's traceback and not the component that called `query_pm`.

## Reproduction

The onboarding research inferred this from the code at revision `7e8cabe` and
didn't run it. At revision `7e8cabe`, declare a package-manager component whose
`interrogate` calls `fail("x")`, call `query_pm` on it from another component's
`install` hook, and run `meowctl apply`.

## What the system does

`Interrogator::interrogate` calls `evaluator.call_hook_collecting(...)?`
(`crates/meowctl-engine/src/runner.rs:620-623`), and the builtin flattens the
error with `anyhow::anyhow!("{e}")`
(`crates/meowctl-starlark/src/builtins.rs:197-205`). `call_handler` wraps the
other handler functions in `HandlerFailure` (`runner.rs:540-552`), and this path
skips it. The outer failure line does name the asking component as `a failed in
install`, so the reader gets both names, but not in one message (medium). No
test covers a raising `interrogate`.

## What it should do, and why

[REQ-2704] says the message names both the declaring component and the handler
component. The fix routes the `interrogate` failure through `HandlerFailure`.

## Triage

A requirement covers it, so the fix enters at implement. It is minor because
both names reach the user, in two lines.

## Closed by

Open.
