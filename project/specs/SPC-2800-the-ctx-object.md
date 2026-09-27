---
id: SPC-2800
artifact: spec
status: live
revised: 2026-09-27
checked-at:
states: [REQ-2801, REQ-2802, REQ-2803, REQ-2810, REQ-2811, REQ-2812, REQ-2813, REQ-2814, REQ-2820, REQ-2821, REQ-2822, REQ-2823, REQ-2824, REQ-2825, REQ-2826, REQ-2827, REQ-2828, REQ-2830, REQ-2831, REQ-2832, REQ-2840, REQ-2841, REQ-2842, REQ-2843, REQ-2844, REQ-2872, REQ-2900, REQ-2901, REQ-2902, REQ-2903, REQ-2904, REQ-2905, REQ-2906, REQ-2907, REQ-2908, REQ-2909, REQ-2910, REQ-2911, REQ-2912, REQ-2940]
---

# The ctx object

## Scope

This component is the `ctx` value passed to every lifecycle hook: the surface a
component author writes against. It is a binding and nothing more. Each method
validates its arguments, builds an `Op` or calls the `Executor`, and returns
(from docs/spec/ctx.md, high). The surface is frozen at what `v0.1.0` exposes,
and a component written for `v0.1.0` runs unchanged (from docs/spec/ctx.md,
high).

It lives in the crate `meowctl-ctx`, its design is
`docs/design/0.2.0-rust-rewrite.md` §3 and defect #3, and its `v0.1.0`
equivalent is `internal/ctx/` (from docs/spec/ctx.md, high).

## Boundary

The attribute names, their signatures, their return values, and which are
available in which phase (from docs/spec/ctx.md, high).

| Phase | Attributes available |
| --- | --- |
| A runtime hook phase, `shell` or `login` | `emit`, `file_exists`, `list_dir`, `platform`, `read_file`, `run`, `shell` and `state_dir` (from docs/spec/ctx.md, high) |
| A read-only phase, as REQ-1012 names them | Every attribute except the mutating methods (from docs/spec/ctx.md, high) |
| Every other phase | The six data properties and the twenty-four methods (from docs/spec/ctx.md, high) |

## Behaviour

### Data properties

`ctx` exposes the six data properties `home`, `dry_run`, `component_dir`,
`state_dir`, `shell` and `platform` [REQ-2801] (from docs/spec/ctx.md, high). In
a runtime hook phase, `shell` names the shell, as the base name of `$SHELL`
[REQ-2802], and in every other phase it is `None` [REQ-2900] (from
docs/spec/ctx.md, high). `platform` is the same struct `platform()` returns
[REQ-2901] (from docs/spec/ctx.md, high). `component_dir` is the component's own
source directory and `state_dir` its persistent per-component directory
[REQ-2803] (from docs/spec/ctx.md, high).

### Methods

`ctx` exposes exactly twenty-four methods: `log`, `env`, `write_file`,
`append_file`, `delete_file`, `copy_file`, `symlink`, `remove_symlink`,
`link_file`, `mkdir`, `read_file`, `file_exists`, `list_dir`, `run`,
`git_clone`, `download`, `defaults_write`, `plist_set`, `prompt`, `emit`,
`add_path`, `render`, `render_file` and `which` [REQ-2810] (from
docs/spec/ctx.md, high). An unknown attribute reports attribute-not-found
[REQ-2811] (from docs/spec/ctx.md, high).

Every method rejects a relative path [REQ-2812] and expands a leading `~`
[REQ-2902] (from docs/spec/ctx.md, high). Every mutating method goes through an
`Op` [REQ-2813], and no method branches on whether this is a dry run, because
the `FileSystem` and `Executor` it was given decide that [REQ-2814] (from
docs/spec/ctx.md, high).

### Process and network

`run(cmd, args = [], env = {}, cwd = None, interactive = False)` returns a value
carrying `stdout`, `stderr` and `exit_code`, under those names [REQ-2820] (from
docs/spec/ctx.md, high). `which(name)` returns the resolved path, or a falsey
value when nothing matches [REQ-2821] (from docs/spec/ctx.md, high).
`git_clone(url, dst, ref = None)` builds a `git` command and runs it through the
`Executor` [REQ-2822] (from docs/spec/ctx.md, high).

`download(url, dst, checksum = None)` fetches over HTTPS [REQ-2823] and journals
a `Download` op [REQ-2903] (from docs/spec/ctx.md, high). When a checksum is
given, `download` verifies it before writing anything [REQ-2904] (from
docs/spec/ctx.md, high).

### Shell integration

`emit(text)` writes to stdout only during the `shell` and `login` phases
[REQ-2824], and does nothing in any other phase [REQ-2905] (from
docs/spec/ctx.md, high). `add_path(dir)` prepends the directory to the `PATH`
every later `run` in this process sees [REQ-2825], does nothing when the
directory is already first [REQ-2906], and emits nothing (from docs/spec/ctx.md,
high). `prompt(question)` goes through the `Interaction` trait and never reads
stdin directly [REQ-2826] (from docs/spec/ctx.md, high).

### Templating

`render(template_str, vars)` substitutes `{{name}}` for each key in `vars` and
returns the result [REQ-2827] (from docs/spec/ctx.md, high).
`render_file(src, vars)` reads `src` relative to `component_dir`, renders it the
same way and returns the result [REQ-2907] (from docs/spec/ctx.md, high).
Neither writes anything (from docs/spec/ctx.md, high). `vars` is a mapping of
strings to strings [REQ-2908] (from docs/spec/ctx.md, high).

`append_file(dst, content, marker = None)` accepts a caller-supplied marker
[REQ-2828] and generates one when it is absent [REQ-2910] (from
docs/spec/ctx.md, high).

### Restricted contexts

In a read-only phase, `ctx` exposes no mutating method [REQ-2830] (from
docs/spec/ctx.md, high). In a runtime hook phase, `ctx` exposes exactly the
eight attributes `ShellCtxAllowList` names [REQ-2831] (from docs/spec/ctx.md,
high). The value the hook receives enforces both restrictions, with no check
inside each method [REQ-2832] (from docs/spec/ctx.md, high).

### The value

`Ctx`, `ReadOnlyCtx` and `ShellCtx` are simple Starlark values over one shared
`Arc<Core>`, and every mutable part of `Core` sits behind `Arc<Mutex<..>>` or a
`Send + Sync` trait object (from crates/meowctl-ctx/src/value.rs:27-39 and
crates/meowctl-ctx/src/state.rs:83-95, high). So a hook's `ctx` is allocated
with `alloc_simple` and holds no reference into the Starlark heap (from
crates/meowctl-ctx/src/value.rs:81-90, high). The M0 finding that `ctx` needs
`alloc_complex_no_freeze` applies only to a value that isn't `Send + Sync`, and
`ctx` is made `Send + Sync` so that it doesn't (from
docs/spec/starlark.md:218-225, high). No pull request records the switch;
keeping the state in `Core` behind `Mutex` is the inferred reason (from
https://github.com/meowshed/meowctl/pull/70, low).

`download` allows a transfer up to 5 minutes end to end [REQ-2872] (from
project/requirements/REQ-2872-download-bounded-at-five-minutes.md, high).

## Failure paths

| Condition | What happens |
| --- | --- |
| A method gets an argument of the wrong type | The error names the method, the argument and the expected type [REQ-2840] (from docs/spec/ctx.md, high) |
| A method gets a relative path | The method rejects it [REQ-2812] (from docs/spec/ctx.md, high) |
| A hook reads an unknown attribute | It gets attribute-not-found, not an internal error [REQ-2811] (from docs/spec/ctx.md, high) |
| A hook in a read-only phase calls a mutating method | It gets attribute-not-found, not a silent no-op [REQ-2911] (from docs/spec/ctx.md, high) |
| An effect fails | The failure propagates as a hook failure, which fails the component and triggers rollback [REQ-2841] (from docs/spec/ctx.md, high) |
| `read_file` gets a missing file | It fails [REQ-2842] (from docs/spec/ctx.md, high) |
| `file_exists` gets a missing file | It returns false [REQ-2912] (from docs/spec/ctx.md, high) |
| `run` sees a non-zero exit | It doesn't fail, and the result carries the exit code [REQ-2843] (from docs/spec/ctx.md, high) |
| `download` receives bytes that don't match `checksum` | The method fails, naming the URL, the expected hash and the actual one, and writes and journals nothing [REQ-2823] [REQ-2940] (from crates/meowctl-ctx/src/value.rs:664-704, high) |
| `remove_symlink` gets a path that isn't a symlink | It fails [REQ-2844] (from docs/spec/ctx.md, high) |
| `render` or `render_file` gets `vars` that isn't a mapping of strings to strings | It is an error, not a formatted substitution [REQ-2909] (from docs/spec/ctx.md, high) |
| `which` finds nothing | It returns a falsey value [REQ-2821] (from docs/spec/ctx.md, high) |
