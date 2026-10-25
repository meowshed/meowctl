---
id: TSK-0041
artifact: task
status: done
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

Criteria 1 and 2 are closed by `a_registry_file_reports_its_place_in_the_cache`
and `a_replaced_file_reports_its_place_in_the_checkout` in
`crates/meowctl-module/tests/fetching.rs`, and criterion 3 by
`a_module_component_lives_where_its_file_was_read` in
`crates/meowctl-cli/src/run/commands.rs`. All three failed before the fix and
pass after it. `mise run fmt-check`, `mise run check`, `mise run test` (603
passed, 3 skipped) and `mise run build` exit 0. Built together with BUG-0042's
fix, `meowctl upgrade @dotmeow//components/git-config` linked
`~/.config/git/config` into the dotmeow 0.3.32 cache directory.
