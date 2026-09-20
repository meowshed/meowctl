# The ctx object

**Crate:** `meowctl-ctx`
**Design:** `docs/design/0.2.0-rust-rewrite.md` §3, defect #3
**v0.1.0 equivalent:** `internal/ctx/`

## Scope

This component is the `ctx` value passed to every lifecycle hook: the surface a
component author actually writes against. It is a binding and nothing more.
Each method validates its arguments, builds an [`Op`](ops.md) or calls the
[`Executor`](exec.md), and returns.

It exists as its own crate because of defect #3. `internal/ctx/methods.go` is
1206 lines mixing Starlark glue, filesystem mutation, process execution, and
rollback journaling, which is why a new method could ship without an inverse
and why dry-run checks had to be repeated in every one of them.

The surface is frozen at what `v0.1.0` exposes. A component written for
`v0.1.0` must run unchanged.

## Boundary

The attribute names, their signatures, their return values, and which are
available in which phase.

## Data properties

**[R-CTX-001]** `ctx` MUST expose the six data properties `New` in
`internal/ctx/ctx.go` registers: `home`, `dry_run`, `component_dir`,
`state_dir`, `shell`, and `platform`.

**[R-CTX-002]** `shell` MUST be `None` outside `shell.star` evaluation, and
`platform` MUST be the same struct `platform()` returns; see [R-STAR-007].

**[R-CTX-003]** `component_dir` MUST be the component's own source directory
and `state_dir` its persistent per-component directory. A component writes
state it wants to survive into the second.

## Methods

**[R-CTX-010]** `ctx` MUST expose exactly the twenty-four methods `New`
registers, under those names: `log`, `env`, `write_file`, `append_file`,
`delete_file`, `copy_file`, `symlink`, `remove_symlink`, `link_file`, `mkdir`,
`read_file`, `file_exists`, `list_dir`, `run`, `git_clone`, `download`,
`defaults_write`, `plist_set`, `prompt`, `emit`, `add_path`, `render`,
`render_file`, and `which`.

**[R-CTX-011]** An unknown attribute MUST report attribute-not-found rather
than raise an internal error, matching `Attr` returning `(nil, nil)`.

**[R-CTX-012]** Every method MUST reject a relative path and MUST expand a
leading `~`; see [R-COMMON-022].

**[R-CTX-013]** Every mutating method MUST go through an `Op`. No method may
call `FileSystem` directly, because an effect that is not an `Op` has no
inverse; see [R-OPS-002].

**[R-CTX-014]** No method MUST branch on whether this is a dry run. The
`FileSystem` and `Executor` it was given decide that; see [R-FS-011].

## Process and network

**[R-CTX-020]** `run(cmd, args = [], env = {}, cwd = None, interactive = False)`
MUST return a value carrying `stdout`, `stderr`, and `exit_code`, under those
names. Component code branches on `exit_code`, so the shape is part of the API.

**[R-CTX-021]** `which(name)` MUST return the resolved path or a falsey value
when nothing matches; see [R-EXEC-032].

**[R-CTX-022]** `git_clone(url, dst, ref = None)` MUST build a `git` command
and run it through the `Executor`, rather than implementing a fetch.

**[R-CTX-023]** `download(url, dst, checksum = None)` MUST fetch over HTTPS and
MUST journal a `Download` op, so an interrupted run restores what was at `dst`;
see [R-OPS-016]. When a checksum is given it MUST be verified before anything
is written, for the reason [R-MODULE-030] gives.

## Shell integration

**[R-CTX-024]** `emit(text)` MUST write to stdout only during the `shell` and
`login` phases, and MUST be a no-op otherwise. This is how a component
contributes to the shell environment, and stdout in any other phase would
corrupt a piped run; see [R-COMMON-013].

**[R-CTX-025]** `add_path(dir)` MUST prepend the directory to the `PATH` every
later `run` in this process sees, and MUST do nothing when it is already
first. It emits nothing.

The name suggests a shell statement and it is not one. A component that
installs a tool whose binary is not yet on `PATH` calls it so the next
`ctx.run` can find that binary, which is what `starAddPath` does by setting the
process environment. A component wanting a shell to see the directory calls
`emit` itself. Reproduced because the standard library depends on it, and
stated here because the name does not say it.

**[R-CTX-026]** `prompt(question)` MUST go through the `Interaction` trait, not
read stdin directly; see [R-TUI-060].

## Templating

**[R-CTX-027]** `render(template_str, vars)` MUST substitute `{{name}}` for
each key in `vars` and return the result, and `render_file(src, vars)` MUST
read `src` relative to `component_dir`, render it the same way, and return the
result. Neither writes anything: a component that wants the rendered text on
disk passes it to `write_file`.

`vars` MUST be a mapping of strings to strings, and anything else MUST be an
error rather than being formatted. `renderTemplate` refuses it, and a
component that passed an integer would otherwise get a substitution that
depends on Starlark's formatting.

**[R-CTX-028]** `append_file(dst, content, marker = None)` MUST accept a
caller-supplied marker and MUST generate one when it is absent. A component
that re-runs with the same marker replaces its own block rather than appending
a second copy, which is why the argument exists; see [R-OPS-011].

## Restricted contexts

**[R-CTX-030]** In a read-only phase, `ctx` MUST expose no mutating method. The
read-only phases are in [R-COMMON-012], and a hook that tries to write in one
MUST get attribute-not-found rather than a silent no-op.

**[R-CTX-031]** During `shell.star` evaluation, `ctx` MUST expose exactly the
eight attributes `ShellCtxAllowList` names: `emit`, `file_exists`, `list_dir`,
`platform`, `read_file`, `run`, `shell`, and `state_dir`. A shell hook runs on
every shell spawn and must have no persistent effect beyond what it emits.

**[R-CTX-032]** The restriction MUST be enforced by the value the hook receives,
not by a check inside each method.

## Failure paths

**[R-CTX-040]** A method called with a wrong argument type MUST name the
method, the argument, and the expected type.

**[R-CTX-041]** A failed effect MUST propagate as a hook failure, which fails
the component and triggers rollback; see [R-ENGINE-030].

**[R-CTX-042]** `read_file` on a missing file MUST fail, while `file_exists`
MUST return false. The two are how a component tests and then reads, and
conflating them turns a test into an error.

**[R-CTX-043]** `run` MUST NOT fail on a non-zero exit; see [R-EXEC-004].

**[R-CTX-044]** `remove_symlink` on a path that is not a symlink MUST fail; see
[R-FS-022].

## Parity with v0.1.0

The property list, the method list, the `run` result shape, the shell
allow-list, and the read-only phase set are all read from `internal/ctx/`. The
argument names and defaults of each method are part of the surface and are
reproduced from the same file.

[R-CTX-030] and [R-CTX-031] are new. `v0.1.0` has the machinery -- a
`RestrictedCtxValue` and a `ShellCtxAllowList` -- and wires neither up:
`NewRestricted` is called from nowhere outside its own package, so every hook
in every phase gets the full `ctx`. `validCheckPhases`, which [R-COMMON-012]
cites, does not exist either; the four read-only phases are named here and
nowhere in `v0.1.0`.

Neither restriction moves anything that exists. No `verify`, `install_check`,
`upgrade_check` or `uninstall_check` hook in `meowctl-stdlib` or in `dotmeow`
calls a mutating method, which was checked rather than assumed.

Three statements above were wrong about `v0.1.0` when they were written, and
are corrected rather than implemented as written: `add_path` sets the process
`PATH` and emits nothing, `render_file` returns a string and writes nothing,
and `download` takes a `checksum`. Each is what `internal/ctx/methods.go`
does, and a component written against `v0.1.0` depends on all three.

Two things change, both structural rather than observable. `SuspendOutput`
leaves `Capabilities` entirely, because terminal hand-off is negotiated through
events; see [R-EXEC-022]. And the dry-run branches leave every method, because
the effect implementations decide; see [R-CTX-014]. Neither changes what a
component file can do.
