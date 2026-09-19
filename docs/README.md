# Documentation

Every documentation file in the repository is listed here. Update this index
whenever a file is added, moved, or deleted.

## Design

Architecture-level decisions. These change rarely and always deliberately.

| File | Covers |
| --- | --- |
| [`design/0.2.0-rust-rewrite.md`](design/0.2.0-rust-rewrite.md) | The plan for the Rust rewrite: why it is a rewrite rather than a port, the ten defects it fixes, the crate layout, the terminal output redesign, the Starlark parity risk, and the milestone order |
| [`design/0.2.0-decisions.md`](design/0.2.0-decisions.md) | The architecture decisions and what each costs, and the thirty requirements where a different answer was available |
| [`design/0.2.0-requirement-tradeoffs.md`](design/0.2.0-requirement-tradeoffs.md) | Every one of the 282 requirements, with the alternative that was available, why it lost, and what would reverse it |
| [`design/0.2.0-execution-plan.md`](design/0.2.0-execution-plan.md) | The 282 requirements decomposed into 34 issues with dependencies, sizes, and a coverage check |

## Specifications

Component-level normative behaviour, one file per crate. See
[`spec/README.md`](spec/README.md) for the area prefixes and the template.

The files are listed bottom-up, in dependency order: each one may cite the
requirements above it in this table, and none cites a requirement below it
except where the higher layer constrains the lower on purpose.

| File | Crate | Covers |
| --- | --- | --- |
| [`spec/common.md`](spec/common.md) | `meowctl-common` | Identifiers, phases, paths, the error and exit-code taxonomy, the `Event` vocabulary |
| [`spec/config.md`](spec/config.md) | `meowctl-config` | Every on-disk format, its schema version, and the syntax-aware Starlark editor |
| [`spec/fs.md`](spec/fs.md) | `meowctl-fs` | The `FileSystem` trait and its real, dry-run, and in-memory implementations |
| [`spec/exec.md`](spec/exec.md) | `meowctl-exec` | The `Executor` trait, subprocess output, and terminal hand-off |
| [`spec/ops.md`](spec/ops.md) | `meowctl-ops` | The `Op` enum, inverses, the write-ahead journal, and replay |
| [`spec/starlark.md`](spec/starlark.md) | `meowctl-starlark` | Builtins, the accumulator, `load()` resolution, and diagnostics |
| [`spec/module.md`](spec/module.md) | `meowctl-module` | Version selection, fetching, integrity, the cache, and locking |
| [`spec/pm.md`](spec/pm.md) | `meowctl-pm` | Package-manager handler registration and dispatch |
| [`spec/ctx.md`](spec/ctx.md) | `meowctl-ctx` | The `ctx` object a hook receives, and its restricted forms |
| [`spec/engine.md`](spec/engine.md) | `meowctl-engine` | The staged pipeline, the component graph, phases, staleness, and rollback |
| [`spec/tui.md`](spec/tui.md) | `meowctl-tui` | The three event sinks, the theme, capability detection, and prompts |
| [`spec/cli.md`](spec/cli.md) | `meowctl-cli` | The command surface, exit-code mapping, and the compat corpus |

## Elsewhere in the repository

| File | Covers |
| --- | --- |
| [`../CLAUDE.md`](../CLAUDE.md) | Root policy: principles, architecture index, the workflow, and the skills |
| [`../CHANGELOG.md`](../CHANGELOG.md) | Release history |
| [`../README.md`](../README.md) | What meowctl is, and how to build it |
| [`../Cargo.toml`](../Cargo.toml) | Workspace members, shared package metadata, the lints every crate inherits, and the build profiles |
| [`../deny.toml`](../deny.toml) | Which licences a dependency may carry, and why copyleft is excluded |
