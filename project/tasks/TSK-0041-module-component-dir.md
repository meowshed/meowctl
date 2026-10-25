---
id: TSK-0041
artifact: task
status: approved
revised: 2026-10-25
bug: BUG-0043
closes: []
---

# Take a module component's directory from where the loader read it

A loaded file carries the path it was read from, when it has one, and the
CLI gives a module component the directory holding that file as its
`component_dir`.

## Acceptance criteria

1. Given a registry module in the cache, when the module loader loads
   `@stdlib//components/apt`, then the file reports
   `<cache>/stdlib/0.2.17/components/apt/init.star` as its path. Closed by: a
   test in `crates/meowctl-module/tests/fetching.rs`.
2. Given a replaced module, when the loader loads a component from it, then
   the file reports its path in the checkout. Closed by: a test in
   `crates/meowctl-module/tests/fetching.rs`.
3. Given a module component whose file the loader read from a path, when the
   CLI builds its source, then its directory is that file's directory. Closed
   by: a test in `crates/meowctl-cli/src/run/commands.rs` naming REQ-2803.

## What to do

Add the on-disk path to `LoadedFile` in `meowctl-starlark` as an optional
field, fill it in `ModuleLoader` for local, registry, replaced and GitHub
files, and use its parent in `ConfigSources::source`. A loader that reports no
path keeps today's directory, because the engine's own test loaders have none.

## Depends on

None.

## Left alone

Bare-name components keep the REQ-3107 layout, because their directory is
already the one their file sits in.

## Evidence

Not yet.
