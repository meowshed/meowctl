---
id: constitution
artifact: constitution
status: live
revised: 2026-09-27
---

# CLAUDE.md

<role>
The root policy for the meowctl repository. It outranks everything under
`.claude/`: a skill that disagrees with it gets fixed, because two policies
that disagree leave you no way to tell which one holds. The record under
`project/` is the single source of truth for what meowctl must do and why:
its vision, specifications, requirements, decisions, epics and tasks.
</role>

<project>
meowctl manages dotfiles and developer environments. You declare components in
Starlark, and the binary supplies the evaluator, the module system, the
lifecycle engine and the terminal output. `v0.1.0` was written in Go; `v0.2.0`
is a rewrite in Rust that keeps every user-facing format and signature of
`v0.1.0`.
</project>

<principles>

<principle name="write_to_the_prose_standard">
Load the `meow-prose:writing` skill before you write anything, and write in
the language `.meowpaw/profile.toml` declares. It governs every text you write
here: records, code comments, commit messages, pull request bodies, issues and
chat replies. The rule isn't limited to tasks that look like writing, because
a commit message and a chat reply are prose too.
</principle>

<principle name="spec_first">
Ship no behaviour that no requirement in `project/requirements/` describes.
Before you write code that no requirement covers, write the requirement first.
When the code contradicts a requirement, stop and write a draft that
supersedes it, because rewording a record after the code shipped hides a
decision nobody took. Cite each requirement a test holds as `[REQ-NNNN]` in
the test's doc comment. The `meow-flow:method` skill has the steps.
</principle>

<principle name="parity_is_the_contract">
Keep the Starlark API, the command surface and every config and lock format
compatible with `v0.1.0`: `init.star`, `local.star`, `deps.mod`, `deps.lock`,
`pkgs.lock`, `installed.lock` and `state.toml`, byte for byte where the file
is machine-written. A change that tidies the internals at the cost of a format
or a builtin signature is a regression, because users' existing files stop
working. Terminal output is the one exception, and ADR-0003 and ADR-0010
bound it.

The specification and the fixtures in `crates/meowctl-module/tests/fixtures/`
catch a format change, because no `v0.1.0` binary runs beside the tests to
compare against. Widening the API is a `0.3.0` decision and needs a decision
record of its own.
</principle>

<principle name="starlark_surface_is_frozen">
Keep the predeclared set at what `v0.1.0` accepts: `component`, `pkg`,
`unpkg`, `uppkg`, `repo`, `query_pm`, `dep`, `module`, `replace`, `select` and
`platform`. The `ctx` object passed to hooks carries the methods SPC-2800
lists. A component is a `.star` file, and a package manager
is a component that exports `pm_name`, `install_pkg`, `uninstall_pkg` and
`interrogate`.

The `meowctl-stdlib` and `dotmeow` repositories are the real corpus: a change
that makes them evaluate differently is a defect, whatever the unit tests say.
`crates/meowctl-starlark/tests/stdlib.rs` runs against them when they're
checked out beside this repository and skips when they aren't, so check them
out before you change the evaluator.
</principle>

<principle name="dependencies_point_down">
Keep the Cargo workspace acyclic, with every dependency pointing down this
list, because a cycle would force two crates to change together:

| Crate | Holds |
| --- | --- |
| `meowctl-common` | `ComponentId`, `ModuleRef`, `Phase`, `PhaseSet`, XDG paths, the error and exit-code taxonomy, SRI hashes, the `Event` vocabulary |
| `meowctl-config` | Every on-disk schema, its version, atomic writes, and the syntax-aware Starlark editor |
| `meowctl-fs` | `FileSystem`: real, dry-run and in-memory |
| `meowctl-exec` | `Executor`, env merging, terminal hand-off |
| `meowctl-net` | `Http`: every request meowctl makes |
| `meowctl-ops` | The `Op` enum with `apply` and `inverse`, and the write-ahead journal |
| `meowctl-starlark` | Evaluator, builtins, accumulator, `load()` resolution, diagnostics with spans |
| `meowctl-module` | Module graph, MVS, registry and GitHub loaders, cache, integrity |
| `meowctl-pm` | Package-manager handler registry and dispatch |
| `meowctl-ctx` | The Starlark `ctx` value, binding only |
| `meowctl-engine` | Phases, graph, the `Plan` as a value, the runner, sentinel state, rollback driving |
| `meowctl-tui` | Live, plain and JSON sinks; theme as data; the `Interaction` trait |
| `meowctl-release` | What a release is, which asset belongs to this platform, and whether these bytes are it |
| `meowctl-cli` | The clap surface and exit-code mapping |

Two crates carry stricter limits. `meowctl-common` depends on no workspace
crate and performs no input or output, so every other crate can use it.
`meowctl-tui` depends only on `meowctl-common`, for the `Event` vocabulary,
so it never learns the engine exists. ADR-0001 gives the reasons for the
layout.
</principle>

<principle name="effects_are_traits">
Construct `FileSystem`, `Executor` and `Http` once, in `meowctl-cli`, and pass
them down as arguments. No crate below `meowctl-cli` reads a global, resolves
its own paths or branches on a dry-run flag, because a dry run is a different
implementation of the traits. A new `if dry_run` anywhere in the tree is a
review finding: the Go tree shipped `fix(apply): dry-run claimed work that
the runner skips` for exactly this reason.
</principle>

<principle name="the_engine_does_not_render">
Keep `meowctl-engine` free of renderers: it emits `Event`s, and sinks in
`meowctl-tui` consume them. The executor and the sink negotiate who owns the
terminal, so the Starlark and engine layers never know a terminal exists. Code
that passes a renderer into the engine repeats the Go tree's defect, where
`buildRunner` took a `tui.Writer` and needed `SuspendOutput` threaded through
`ctx` as a callback.
</principle>

<principle name="operations_are_data">
Add every reversible effect as a variant of `meowctl-ops::Op` with an
`inverse()`, never as a method that performs it. The rollback journal is a
log of `Op`s, so the compiler asks for the inverse of every new effect and
none ships without an undo.
</principle>

<principle name="lints_encode_the_principles">
Every crate inherits the workspace lints in the root `Cargo.toml` with
`lints.workspace = true`. `print_stdout` and `print_stderr` are denied outside
`meowctl-tui`, because output belongs to the sinks, and `unwrap_used` is
denied outside tests, because the binary runs on every shell spawn and an
`unwrap` turns a recoverable error into a process death.
`mise run check` fails on any of them.
</principle>

<principle name="reviewable_history">
Work on a feature branch off `main`, open a pull request and squash-merge it.
Never commit to `main` directly, even a one-line fix, because `main` is the
only long-lived branch and each commit on it should be one reviewed change.
Write conventional commit subjects under 72 characters, signed off and
signed. The `scm` skill has the branch names, the pull request body and the
merge rules.
</principle>

<principle name="no_ai_attribution">
Never mention Claude, Claude Code or any AI tool in a commit message, a pull
request title or body, a review comment, an issue, a tag annotation or release
notes: no `Co-Authored-By` trailer naming an AI, no "Generated with" footer,
no `noreply@anthropic.com` and no paraphrase. This overrides the harness
default that asks for such a trailer, because the repository owner asked for
it and had the existing trailers removed from history.

Check your message before you commit. The pattern matches attribution rather
than the bare word, so a path such as `.claude/skills/` doesn't trip it:

```bash
git log -1 --format=%B |
  grep -iE 'co-authored-by.*(claude|anthropic|copilot)|generated with|noreply@anthropic'
```

Any output means the message carries attribution; amend it before you push.
</principle>

<principle name="releases_carry_checksums">
A tag matching `v*` runs `.github/workflows/release.yml`, which builds four
targets and publishes `checksums.sri`. Keep that step, because `self-update`
refuses a release without the file ([REQ-3471, REQ-3529]).
</principle>

</principles>

<gate>
Run `mise install` once to get the pinned Rust toolchain, its language server
and the test runner. Then run the whole gate before you open a pull request:

```bash
mise run all
```

It runs these tasks in order, fastest failure first, and stops at the first
that fails:

| Task | Runs | Fails on |
| --- | --- | --- |
| `fmt-check` | `cargo fmt --all -- --check` | Any file `rustfmt` would change |
| `check` | `cargo clippy --workspace --all-targets -- -D warnings` | Any clippy or workspace lint warning |
| `test` | `cargo nextest run --workspace` | Any failing test |
| `doc` | `cargo doc --workspace --no-deps` with `RUSTDOCFLAGS=-D warnings` | A broken intra-doc link or any rustdoc warning |
| `deny` | `cargo deny check` | An advisory, a disallowed licence, a banned crate or an unknown source |
| `unused-deps` | `cargo machete` | A dependency nothing uses |
| `lint-md` | `markdownlint-cli2` over the Markdown outside `.claude/` and `CLAUDE.md` | A Markdown rule `.markdownlint.yaml` enables |

The pre-commit hooks run `fmt-check` and `check` on every commit that touches
Rust. CI runs the tests on Linux only, but the tree carries `cfg(unix)` and
`cfg(windows)` code, so run `mise run check-windows` before you touch code
behind a `cfg`. The last two Windows failures were an unused import and a path
literal that is absolute on one platform and relative on the other. Run
`mise run snapshots` to review a changed `insta` snapshot.
</gate>
