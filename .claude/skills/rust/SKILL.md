---
name: rust
description: Rust conventions for the meowctl v0.2.0 workspace - crate boundaries, effects behind traits, operations as data, error handling with thiserror and miette, the Starlark bridge, diagnostics through the event stream, testing with nextest and insta, and the lints this repository enforces. Load before writing or reviewing Rust code here.
---

# Rust in this workspace

`docs/design/0.2.0-rust-rewrite.md` decides the crate layout and the execution
model. This skill covers how to write code inside those decisions. When the two
disagree, the design document wins.

## Crate boundaries

Three boundaries carry the whole design, and a change that crosses one is a
defect however small it looks:

- `meowctl-common` depends on no workspace crate and performs no input or
  output. It holds types: `ComponentId`, `Phase`, `ModuleRef`, the error
  taxonomy, and the `Event` vocabulary.
- `meowctl-tui` depends on `meowctl-common` for `Event` and on nothing else in
  the workspace. It does not know the engine exists, which is what lets the
  sinks be tested against a fixture stream with no terminal and no engine.
- Nothing below `meowctl-cli` constructs an effect. `FileSystem` and `Executor`
  arrive as arguments, because a layer that can reach the real filesystem can
  reach it during a dry run.

The Go tree broke the equivalent boundaries: `internal/cli` holds apply
planning and lockfile I/O next to the cobra commands, and `buildRunner` takes a
`tui.Writer`. Do not reproduce either.

Keep the layering checkable. `cargo deny` and a workspace lint enforce it, so a
violation fails CI instead of being found in review.

## Code intelligence

`.claude/settings.json` enables the `rust-analyzer-lsp` plugin, so the `LSP`
tool answers `goToDefinition`, `findReferences`, `hover`, `documentSymbol`, and
the call hierarchy across the workspace. Reach for it before grepping for a
symbol: grep finds every string that looks like the name, and rust-analyzer
finds the one definition that is in scope.

When the tool reports that the server "crashed with exit code 1", the usual
cause is not the code. rustup's shim in `~/.cargo/bin` normally wins on PATH,
and it dispatches on `RUSTUP_TOOLCHAIN`, which mise sets to the concrete
toolchain it resolved. mise installs the `rust-analyzer` component into its own
copy, so the bare command can still fail with "Unknown binary in official
toolchain". Run `mise run setup` once, which adds the component to the active
toolchain too, then check with `rust-analyzer --version`.

## Effects are traits

`FileSystem` and `Executor` are constructed once, in `main`, from the parsed
flags. `--dry-run` selects `DryRunFs`, and that is the entire dry-run
mechanism:

```rust
let fs: Box<dyn FileSystem> = if args.dry_run {
    Box::new(DryRunFs::new(RealFs::new()))   // records intent, writes nothing
} else {
    Box::new(RealFs::new())
};
```

No function below `meowctl-cli` takes a `dry_run: bool` or reads one from a
config. `fix(apply): dry-run claimed work that the runner skips` is the bug
this prevents, and it shipped because the flag was checked in a dozen places
and missed in one.

The same rule makes the engine testable. A unit test constructs `MemFs` and a
scripted `Executor`, runs a phase, and asserts on the resulting tree, without a
temporary directory or a subprocess.

## Operations are data

Every reversible effect is a variant of `meowctl_ops::Op`, not a method:

```rust
pub enum Op {
    WriteFile { path: PathBuf, contents: Vec<u8> },
    Symlink { source: PathBuf, target: PathBuf },
    Mkdir { path: PathBuf, mode: u32 },
    // ...
}

impl Op {
    pub fn inverse(&self, fs: &dyn FileSystem) -> Result<Op, OpsError>;
    pub fn apply(&self, fs: &dyn FileSystem) -> Result<(), OpsError>;
}
```

Adding an effect means adding a variant, and the compiler then asks for its
inverse. In the Go tree a new `ctx` method could ship with no rollback support
and nothing would notice until a failed run left the machine half-configured.

The journal is a log of `Op`s. A property test applies an arbitrary sequence to
`MemFs`, replays the inverses, and asserts the tree matches what it started
with.

## Errors

Libraries return `thiserror` enums. One enum per crate, variants named for what
went wrong rather than for where it happened, and every variant carrying enough
to act on:

```rust
#[derive(Debug, thiserror::Error)]
pub enum ModuleError {
    #[error("integrity mismatch for {module}: expected {expected}, got {actual}")]
    IntegrityMismatch { module: ModuleRef, expected: String, actual: String },

    #[error("no version of {module} satisfies {constraint}")]
    Unsatisfiable { module: ModuleRef, constraint: String },
}
```

`meowctl-cli` converts to `miette` at the boundary and renders with source
spans. This is how a mistake in `init.star` becomes a diagnostic pointing at
the line, instead of a string that describes it. The Go tree could not do this,
which is why its Starlark errors name a file and nothing more.

Exit codes are a single mapping in `meowctl-cli`, from the error taxonomy in
`meowctl-common`. No other crate knows an exit code exists.

Never use `unwrap` or `expect` outside tests and `build.rs`. When an invariant
truly cannot fail, write the reason:

```rust
// The registry inserted this id one line above, so the lookup cannot miss.
let handler = registry.get(&id).expect("id was just inserted");
```

Never swallow an error. A failed sentinel write that gets logged and ignored
produces a state file that is silently wrong, and the next run makes a decision
from it.

## The Starlark bridge

`starlark-rust` replaces `go.starlark.net`, and the two differ in ways that
matter here. M0 is the spike that establishes exactly how; until it lands,
treat every binding question as open rather than guessing from Go experience.

What the design already commits to:

- `ctx` is a custom `StarlarkValue` with attributes, and its methods are thin.
  Each one validates its arguments, builds an `Op` or calls `Executor`, and
  returns. Logic that decides what to do belongs in `meowctl-ops` or
  `meowctl-engine`, not in the binding.
- `load()` routes through the composite loader, which resolves filesystem,
  registry, and GitHub modules and verifies integrity before evaluation.
- Per-evaluation state reaches builtins through the evaluator's extra value,
  never through a global.

Do not hold a `starlark::Value` beyond the evaluation that produced it. Extract
what you need into owned data at the boundary, the same discipline the Go code
followed by copying declarations into an accumulator.

## No async runtime

The work is a handful of sequential tarball fetches, so `ureq` with `rustls`
does the network and there is no `tokio` in the tree. This is a deliberate
choice, not an omission: `meowctl hook shell` runs on every shell spawn, and
startup cost is the thing that makes or breaks that command.

Adding an async runtime is an architecture change. Route it through
`/amend-spec` against the design document, with a measurement that shows why
the sequential path is not enough.

## Diagnostics

Nothing prints. `println!` and `eprintln!` are denied by lint outside
`meowctl-tui`, because output is the sink's job and a stray write corrupts the
live region.

Code that has something to report emits an `Event`. `Message { severity, text }`
exists for text that genuinely has no structure, and severity decides where it
lands; everything else gets a variant. If you find yourself formatting a string
to describe a state change, the state change wants an event.

`tracing` covers developer-facing instrumentation, off by default. Instrument
at boundaries: a span per phase and per component is useful, a span per
filesystem call is noise that costs more than it tells you.

## Tests

`cargo nextest run --workspace` is the runner. Unit tests live beside the code
in `mod tests`; integration tests live in `tests/`.

Every test that checks a specification requirement names it:

```rust
/// [R-MODULE-031] a replaced module skips integrity verification
#[test]
fn replace_with_local_path_bypasses_integrity() { ... }
```

Use `insta` for anything rendered: sink output, diagnostics, help text, and the
JSON event stream. These change often, and reviewing a snapshot diff is faster
and more honest than maintaining assertions by hand.

Use `proptest` where a round trip should hold: apply-then-undo over `MemFs`,
config parse-then-write, and the Starlark editor preserving comments and
formatting.

Test the failure paths as carefully as the success path. A hook that exits
non-zero mid-phase, a tarball whose hash does not match, a lock file from a
newer schema version, and a module bumped since the last apply are where the
defects live.

Do not test private internals. A test that reaches past a public API makes the
crate hard to change and proves nothing a user could observe.

## Style

Keep the default `rustfmt`; there is no project style beyond it. Clippy runs
with `-D warnings`, and an `#[allow]` needs a comment saying why.

Derive `Debug` on every public type. Prefer `&str` over `String` in arguments,
`impl Trait` in argument position for simple bounds, and a named type in return
position when the caller has to name it.

Name things for what they are in the design document. If the design calls it a
`Plan`, the struct is `Plan`, not `ApplyState`. One term, one meaning, across
prose and code alike, and the Go names carry over where the concept is the
same: a `Phase` is still a phase, a `ComponentId` is still a component id.

## Dependencies

Adding one is a decision. Prefer the standard library, then something already
in the tree, then a well-maintained crate with a licence `cargo deny` accepts.
Say in the pull request why the dependency earns its place.

The design document already picks the load-bearing ones: `starlark`, `clap`,
`thiserror`, `miette`, `toml`, `serde`, `semver`, `ureq`, `rustls`, `sha2`,
`tar`, `flate2`, `anstyle`, and `insta`. Replacing one of those is a design
change, not a dependency bump.

`cargo machete` catches dependencies nobody uses. Run it before opening a pull
request that changes a `Cargo.toml`.
