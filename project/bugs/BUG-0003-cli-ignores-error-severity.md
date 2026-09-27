---
id: BUG-0003
artifact: bug
status: approved
severity: major
violates: REQ-2253
found: 2026-09-27
revised: 2026-09-27
issue: 139
---

# The CLI ignores the severity lower crates compute, so a module that won't fetch during `apply` exits 3

`CliError` picks its exit code from which crate the error came from, and never
asks the error for its severity, so a network failure reached through `load()`
exits with the configuration code.

## Reproduction

The research ran the debug binary `target/debug/meowctl`, built from revision
`7e8cabe`, on macOS in a scratch directory, with `HOME`, `XDG_CACHE_HOME` and
`--config` pointed there. The research put this line in a component and ran
`meowctl apply`:

```python
load("@nope//x.star", "Y")
```

It printed `meowctl: a: error: loading @nope//x.star: the registry has no module
named nope` and exited 3. A `load()` of
`github://meowshed/meowctl@no-such-ref-xyz//init.star` also exited 3 (see
BUG-0013).

## What the system does

`From<EngineError>`, `From<ConfigError>` and `From<meowctl_common::Error>` all
become `CliError::Configuration`, which exits 3, and `From<FsError>` becomes
`CliError::General` (`crates/meowctl-cli/src/error.rs:62-100`).
`ModuleLoader::load` returns `StarlarkError::Load`
(`crates/meowctl-module/src/loader.rs:212-217`), which `declarations()` and
`ConfigSources::source` turn into a configuration error
(`crates/meowctl-cli/src/run/commands.rs:213-219`, `commands.rs:272-277`). `grep
-rn "\.severity()" crates/*/src` finds only `crates/meowctl-cli/src/run.rs:71`,
so `StarlarkError::severity()` and `EngineError::severity()` are never called.
Only `dep sync` and `dep upgrade` exit 4 for a network failure. A `fail()` in a
hook exits 1 through `commands.rs:497`, where [REQ-2305] states 3.

The test at `crates/meowctl-starlark/tests/evaluation.rs:268` asserts only the
message, so it passes.

## What it should do, and why

[REQ-2253] says a `load()` of a module that can't be fetched exits with the
module code, whichever command triggered it, and [REQ-1030] fixes the codes
scripts branch on. The fix carries the lower crate's `Severity` into `CliError`
in place of choosing the variant by crate.

## Triage

A requirement covers it, so the fix enters at implement. It is major because
exit codes are the contract scripts use to tell a broken configuration from a
broken network, and every command but `dep sync` and `dep upgrade` reports an
outage as a configuration error. No data is lost, which keeps it below critical.

## Closed by

Open. The fix adds a binary-level test citing [REQ-2253] that expects exit 4.
