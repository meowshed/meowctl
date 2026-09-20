# Changelog

All notable changes to meowctl will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Changed

- Rewritten in Rust. `v0.2.0` is a ground-up rewrite, not a port: the Starlark
  evaluator is `starlark-rust`, effects sit behind `FileSystem`, `Executor`
  and `Http` traits constructed once at the binary, every reversible effect is
  an `Op` with an inverse, and the engine emits an event stream that four
  sinks render. `docs/design/0.2.0-rust-rewrite.md` says why each of those is
  a rewrite rather than a refactor.
- Terminal output redesigned. A live region renders what is running, the
  finished work is committed above it, and `--format json` emits the same
  event stream on every command rather than only on `doctor`.
- `--dry-run` no longer claims work the runner skips. A dry run is a
  `FileSystem` and an `Executor` that cannot write, so it predicts exactly
  what a real run would do.
- An interrupt stops the run rather than killing the process. The first one
  asks it to stop before the next component and leaves the journal for the
  next run to find; the second stops the process at once.
- `self-update` verifies what it downloaded before replacing the running
  binary. `v0.1.0` renamed an unverified download over it; a release now
  publishes `checksums.sri` and a release without one is refused.
- A `deps.mod` naming both a version and a source for one dependency is
  refused when it is read, not only when `meowctl dep add` writes it.
- A lock file naming an integrity hash that is not a W3C SRI one fails to
  parse, rather than failing later as a hash mismatch.

The Starlark API, the command surface, and every config and lock format are
unchanged. A configuration that `v0.1.0` applied applies here.

### Added

- `theme.toml` in the configuration directory: a table per role over the
  built-in palette. A role it leaves out keeps its default; a malformed file
  warns and falls back rather than stopping the command.
- A release workflow that builds four targets and publishes the checksums
  `self-update` verifies against.

### Removed

- The Go implementation, and the compatibility corpus that compared the two
  binaries. The corpus existed to prove the rewrite does what `v0.1.0` did,
  and that proof had an end date.

### Fixed

- `meowctl hook shell` and `meowctl hook login` run again. They were a stub
  through most of the rewrite, which would have left a shell without its
  `PATH`, its `mise`, its `direnv` and every variable its components
  contribute.
- A component inside a module can name its siblings without the module
  prefix, which is what `@dotmeow`'s components do.
- `pkgs.lock` and `pkgs.local.lock` are written again after an apply.
- Every lifecycle command takes the flags `v0.1.0` gives it. Five were
  missing `--no-rollback`, two `--force`, and `verify` `--dry-run`.
- The home directory is found on Windows, where it comes from
  `%USERPROFILE%` rather than `$HOME`.

## [0.1.0] - 2026-09-19

First tagged release. The Go implementation is feature-complete for the
dotfiles workflow it was designed around: declare components in Starlark,
resolve modules, run lifecycle phases, roll back on failure.

### Added

#### Configuration language

- Starlark evaluation engine (`go.starlark.net`) with a fixed predeclared set:
  `component`, `pkg`, `unpkg`, `uppkg`, `repo`, `query_pm`, `dep`, `module`,
  `replace`, `select`, `platform`.
- Declaration accumulator — builtins collect declarations per evaluation
  instead of mutating global state.
- `select()` and `platform()` for OS, arch, and Linux distro branching
  (including `ID_LIKE` matching).
- Config layout: `init.star` (user-owned), `local.star` (machine-local,
  gitignored), `components/*.star`, `hooks/`.
- `ctx` object passed to every hook: file operations (`write_file`,
  `append_file`, `delete_file`, `copy_file`, `symlink`, `remove_symlink`,
  `link_file`, `mkdir`, `read_file`, `file_exists`, `list_dir`), process
  execution (`run`, `which`, `git_clone`), network (`download`), macOS
  integration (`defaults_write`, `plist_set`), templating (`render`,
  `render_file`), and shell integration (`emit`, `add_path`, `prompt`, `log`,
  `env`).
- Restricted `ctx` for read-only phases — check and verify phases cannot
  mutate the system.

#### Module system

- `deps.mod` / `deps.local.mod` machine-managed manifests with `module()`,
  `dep()`, and `replace()`.
- Minimal Version Selection over the dependency graph.
- Module sources: registry modules resolved through the meowctl-registry
  index, and `github://owner/repo@ref//path` modules resolved to a commit SHA.
- `deps.lock` / `deps.local.lock`: resolved versions, tarball source, W3C SRI
  integrity, per-file hashes, and commit SHAs for reproducible fetches.
- Local path and remote fork overrides via `replace()`.
- Module cache under `~/.cache/meowctl/modules`, with per-file integrity
  verification on load.
- Composite loader: filesystem, registry, and GitHub loaders behind one
  `load()` resolution path.

#### Lifecycle engine

- Thirteen command-scoped phases across install, update, upgrade, uninstall,
  hook (`shell`, `login`), and verify phase sets.
- Component dependency graph with topological ordering, derived from explicit
  `deps` plus `after` ordering hints.
- Two-pass component evaluation: pass one registers package-manager handlers
  and metadata, pass two runs hooks.
- Package-manager handler registry — any component exporting `pm_name`,
  `install_pkg`, `uninstall_pkg`, and `interrogate` becomes a handler;
  `pkg()` declarations dispatch to it. Optional `update_pkg` and `add_repo`.
- `pkgs.lock` / `pkgs.local.lock` recording resolved package installations.
- Write-ahead rollback journal: every reversible operation records its inverse
  before executing, and a failed run replays the stack in reverse.
- Sentinel state at `state.toml` — current run, last run, per-component
  completion records, interrupted-run detection, and rollback outcome.
- Stale-component detection: a module bump invalidates the completion records
  of every component that resolves from it, so `apply` re-links them.

#### Commands

- `init` — scaffold a config directory, or bootstrap a machine from a public
  dotfiles repository over pure-Go HTTPS (no `git` subprocess).
- `apply`, `upgrade`, `verify`, `update` — the lifecycle phase sets.
- `add` / `remove` — component declarations, with surgical edits to
  `init.star`.
- `dep list` / `add` / `remove` / `upgrade` / `sync` / `tidy` — module
  management.
- `hook <phase>` — runtime shell and login hooks; `shell <shell>` emits the
  integration snippet for zsh, fish, bash, or POSIX sh.
- `status`, `doctor`, `check <dir>`, `version`, `self-update`, plus shell
  completions.
- Stable exit-code taxonomy distinguishing config errors from runtime failure.

#### Terminal output

- One design system behind both renderers: `Writer` for lifecycle progress,
  `Printer` for plans, results, tables, and diagnostics.
- Capability detection — motion, colour, and Unicode glyphs degrade
  independently for pipes, CI logs, `NO_COLOR`, and non-UTF-8 locales, with no
  loss of information.
- The live renderer never takes raw mode: it stands down and erases its region
  while a hook's subprocess owns the terminal, then reclaims it.
- `--dry-run` planning output across every mutating command.

#### Project infrastructure

- CI: lint (golangci-lint), unit tests with coverage, live-registry
  integration tests, a five-target cross-platform build matrix, gosec,
  govulncheck, trufflehog secret scanning, and CycloneDX SBOM generation.
- Pre-commit hooks mirroring the CI lint and security gates.
- mise-driven toolchain and task definitions; Renovate dependency updates.

[Unreleased]: https://github.com/meowshed/meowctl/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/meowshed/meowctl/releases/tag/v0.1.0
