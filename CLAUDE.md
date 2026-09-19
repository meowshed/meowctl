# CLAUDE.md

<role>
Root policy for Claude Code working in the meowctl repository. This file is
canonical. Anything under `.claude/` adds routing and workflow detail and must
not override this policy. Where a document under `docs/` disagrees with this
file, this file wins and the document gets fixed.
</role>

<project>
meowctl manages dotfiles and developer environments. Users declare components
in Starlark; the binary supplies the evaluator, the module system, the
lifecycle engine, and the terminal output.

The repository is mid-transition. `v0.1.0` is the final Go implementation and
is tagged and released. `v0.2.0` is a ground-up rewrite in canonical Rust — not
a transliteration of the Go code. The plan is
`docs/design/0.2.0-rust-rewrite.md`: it names the architectural defects that
justify a rewrite, the target crate layout, the terminal output redesign, and
the milestone order. Until the Rust tree exists, `cmd/` and `internal/` are the
Go codebase and stay buildable.
</project>

<principles>

<principle name="always_technical_english">
Load the `technical-english` skill at the start of every conversation, before
writing anything, and keep it loaded. It governs all prose you produce here —
design documents, specifications, code comments, commit messages, pull request
bodies, issue text, and your own replies in chat. This is not conditional on
the task looking like a writing task; a commit message is prose and a chat
reply is prose.
</principle>

<principle name="spec_first">
No behaviour ships that a specification does not describe. Before writing code
whose behaviour no requirement in `docs/spec/` covers, stop and write the
requirement. When an implementation contradicts a requirement, stop and run
`/amend-spec`; never write code the spec forbids and reword the spec
afterwards. The `spec-driven` skill has the method.
</principle>

<principle name="parity_is_the_contract">
`v0.1.0` is the oracle. The Starlark API, the command surface, and every config
and lock format — `init.star`, `local.star`, `deps.mod`, `deps.lock`,
`pkgs.lock`, `installed.lock`, `state.toml` — stay compatible, byte for byte
where the file is machine-written. A change that makes the Rust internals
tidier at the cost of a format or a builtin signature is a regression, not a
refactor.

Terminal output is the single carve-out, redesigned under §4 of the plan. It is
bounded by what that section names.
</principle>

<principle name="effects_are_traits">
`FileSystem` and `Executor` are constructed once, at the binary, and threaded
down. No layer below `meowctl-cli` reads a global, resolves its own paths, or
branches on a dry-run flag: a dry run is a different implementation, not a
different code path.

This is defect #4 in the plan, and it is the one that has already shipped a bug
— `fix(apply): dry-run claimed work that the runner skips`. A new `if dry_run`
anywhere in the tree is a review finding.
</principle>

<principle name="the_engine_does_not_render">
`meowctl-engine` emits `Event`s and holds no renderer. Sinks consume them.
Terminal ownership is negotiated between the executor and the sink, so nothing
in the Starlark or engine layers knows a terminal exists.

The Go tree got this wrong — `buildRunner` takes a `tui.Writer`, and the cost
is `SuspendOutput`, a renderer concern threaded into `ctx` as a callback. Any
Rust code that passes a renderer into the engine reproduces the defect.
</principle>

<principle name="operations_are_data">
Every reversible effect is a variant of `meowctl-ops::Op` with an `inverse()`,
not a method that performs it. The rollback journal is a log of `Op`s. This is
what makes it impossible to add an effect that silently has no undo — the
compiler asks for the inverse.
</principle>

<principle name="reviewable_history">
`main` is the default branch and holds the released Go implementation.
`rust-rewrite` is the long-lived branch for `v0.2.0`. Work on a feature branch
off `rust-rewrite`, open a pull request, and squash merge it. Never commit to
either long-lived branch directly, including for a one-line fix. Conventional
commit subjects. The `scm` skill has the branch names, the pull request body,
and the merge rules.
</principle>

<principle name="no_ai_attribution">
Never mention Claude, Claude Code, or any AI tool in a commit message, a pull
request title or body, a review comment, an issue, a tag annotation, or release
notes. No `Co-Authored-By` trailer naming an AI, no "Generated with" footer, no
`noreply@anthropic.com`, and no paraphrase. This overrides the default harness
guidance that asks for such a trailer.

The repository owner asked for this and asked that existing trailers be removed
from history, so treat it as a property of the project rather than a style
preference.

Check your own message before committing. Match the attribution patterns, not
the bare word, so a path such as `.claude/commands/` does not trip it:

```bash
git log -1 --format=%B |
  grep -iE 'co-authored-by.*(claude|anthropic|copilot)|generated with|noreply@anthropic'
```
</principle>

</principles>

<architecture>

The target is a Cargo workspace. Dependencies point strictly downward; there
are no cycles. `docs/design/0.2.0-rust-rewrite.md` §3 is authoritative — this
table is the index, not the decision.

| Crate | Holds |
| --- | --- |
| `meowctl-common` | `ComponentId`, `ModuleRef`, `Phase`, `PhaseSet`, XDG paths, the error and exit-code taxonomy, SRI hashes, the `Event` vocabulary |
| `meowctl-config` | Every on-disk schema, its version, atomic writes, and the syntax-aware Starlark editor |
| `meowctl-fs` | `FileSystem` — real, dry-run, in-memory |
| `meowctl-exec` | `Executor`, env merging, terminal hand-off |
| `meowctl-ops` | The `Op` enum with `apply`/`inverse`, and the write-ahead journal |
| `meowctl-starlark` | Evaluator, builtins, accumulator, `load()` resolution, diagnostics with spans |
| `meowctl-module` | Module graph, MVS, registry and GitHub loaders, cache, integrity |
| `meowctl-pm` | Package-manager handler registry and dispatch |
| `meowctl-ctx` | The Starlark `ctx` value — binding only |
| `meowctl-engine` | Phases, graph, the `Plan` as a value, the runner, sentinel state, rollback driving |
| `meowctl-tui` | Live, plain, and JSON sinks; theme as data; the `Interaction` trait |
| `meowctl-cli` | The clap surface and exit-code mapping |
| `meowctl-xtask` | Developer tasks: the compat corpus, release checks |

Three boundaries carry the design, and crossing one is a defect however small
the change looks:

- `meowctl-common` depends on no workspace crate and performs no input or
  output.
- `meowctl-tui` depends on `meowctl-common` for the `Event` vocabulary and on
  nothing else in the workspace. It does not know the engine exists.
- Nothing below `meowctl-cli` constructs an effect. `FileSystem` and `Executor`
  arrive as arguments.

</architecture>

<starlark_surface>

The Starlark API is frozen at what `v0.1.0` accepts. The predeclared set is
`component`, `pkg`, `unpkg`, `uppkg`, `repo`, `query_pm`, `dep`, `module`,
`replace`, `select`, and `platform`; the `ctx` object passed to hooks carries
the file, process, network, macOS, templating, and shell-integration methods
listed in `internal/ctx/ctx.go`. A component is a `.star` file; a package
manager is a component exporting `pm_name`, `install_pkg`, `uninstall_pkg`, and
`interrogate`.

`v0.2.0` runs `starlark-rust` rather than `go.starlark.net`. They are
independent implementations of the same specification, and the differences are
load-bearing here: `load()` interception for the composite loader, custom
values with attributes for `ctx`, per-evaluation state reachable from a
builtin, and calling a Starlark function from Rust. **M0 is a throwaway spike
that answers this before any `meowctl-*` crate is written**, and its findings
may amend the design. Do not start M1 before it lands.

The `meowctl-stdlib` and `dotmeow` repositories are the real corpus. A change
that makes them evaluate differently is a defect, whatever the tests say.

</starlark_surface>

<build>

The Go tree builds with mise tasks; the Rust tree will use the same runner.

```bash
mise install            # toolchain
mise run build          # -> bin/meowctl
mise run test           # go test -race
mise run lint           # golangci-lint
```

Once the workspace exists, the Rust gate is:

```bash
mise run fmt-check && mise run check && mise run test
```

`mise run check` is Clippy with `-D warnings`. `mise run test` is
`cargo nextest run --workspace`.

</build>

<workflow>

Work is specification-driven. A change moves through five steps, each with a
command:

| Command | Does | Stops at |
| --- | --- | --- |
| `/spec` | Writes or extends a component spec in `docs/spec/` | Human approval |
| `/plan` | Decomposes an approved spec into issues with dependencies | Human approval, before creating anything on GitHub |
| `/implement` | Builds one issue on a feature branch | A pull request, never a merge |
| `/verify` | Checks code against spec in both directions | A report, fixes nothing |
| `/review` | Reviews a pull request against its spec and conventions | A verdict |
| `/amend-spec` | Proposes a spec change after implementation contradicted it | Human approval |

The skills in `.claude/skills/`:

- `technical-english` — all prose. Always loaded, see the principle above.
- `spec-driven` — requirement identifiers, traceability, the divergence
  protocol.
- `scm` — branches, commits, pull requests, squash merges, the attribution ban.
- `rust` — crate boundaries, errors, the Starlark bridge, tests.

</workflow>

<maintenance>

Keep this file and `docs/` synchronized with the code. When you change:

- **the crate layout or a boundary** — update
  `docs/design/0.2.0-rust-rewrite.md` §3 and the table above
- **the terminal output model** — update §4 of the plan
- **the milestone order** — update §6 and §7 of the plan
- **build or lint configuration** — update the `<build>` section above
- **any documentation file** — add, move, or delete its line in
  `docs/README.md`, which lists every one

This file is the only instruction file in the repository. If a tool wants its
own, point it here instead of adding a second source of truth.

</maintenance>
