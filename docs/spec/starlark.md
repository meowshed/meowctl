# Starlark evaluation

**Crate:** `meowctl-starlark`
**Design:** `docs/design/0.2.0-rust-rewrite.md` §3 and §5
**v0.1.0 equivalent:** `internal/starlark/`

## Scope

This component evaluates the user's configuration: the predeclared builtins,
the accumulator that collects what they declare, the resolution of `load()`
through the composite loader, and diagnostics that point at a line.

It is the component the parity constraint binds most tightly. The Starlark
surface is what users touch, and `v0.2.0` changes the implementation
underneath it from `go.starlark.net` to `starlark-rust` without changing what a
component file may say.

**M0 gates this spec.** The requirements below state what the evaluator must
do; how it binds to `starlark-rust` is open until the spike answers it, and
findings that contradict a requirement here go through `/amend-spec`.

## Boundary

The predeclared set, the accumulated declarations, the load resolution, the
hook-calling interface, and the diagnostics.

## Predeclared builtins

**[R-STAR-001]** The predeclared set MUST be exactly what
`makePredeclared` in `internal/starlark/builtins.go` provides: `component`,
`pkg`, `unpkg`, `uppkg`, `repo`, `query_pm`, `dep`, `module`, `replace`,
`select`, `platform`, and the `json` module. Nothing MUST be added and nothing
removed.

**[R-STAR-002]** `component(name, after = [], **kwargs)` MUST record a
declaration carrying the name, the ordering hints, and any extra keyword
arguments, in declaration order.

**[R-STAR-003]** `pkg(name, version = "", manager = "", **kwargs)` MUST record
a package declaration. `unpkg` and `uppkg` MUST record the same shape for
removal and update.

**[R-STAR-004]** `repo(...)` MUST record a repository declaration targeting a
named manager.

**[R-STAR-005]** `query_pm(manager)` MUST call the registered handler's
`interrogate` on its own evaluation and return what it returns. It is the one
builtin that runs user code during evaluation; see [R-PM-020].

**[R-STAR-006]** `dep`, `module`, and `replace` MUST record the same
declarations `deps.mod` carries, so that a `deps.mod` is evaluated rather than
parsed by a second grammar; see [R-CONFIG-011].

**[R-STAR-007]** `platform()` MUST return a struct carrying the operating
system, the architecture, and, on Linux, the distribution and its `ID_LIKE`
value. `makePlatformStruct` is the shape.

**[R-STAR-008]** `select(cases)` MUST choose by platform using the matching
rules in `matchesPlatform` and `matchesLinuxDistro`, including matching a
distribution through `ID_LIKE` so a Mint machine matches a `debian` case.

## The accumulator

**[R-STAR-010]** Declarations MUST be collected per evaluation, not in a
global. `v0.1.0` reaches the accumulator through thread-local state, and the
equivalent must be per-evaluation here; a second evaluation must not see the
first one's declarations.

**[R-STAR-011]** The accumulator MUST hold owned data. No Starlark value may
outlive the evaluation that produced it, which is a constraint
`go.starlark.net` did not impose and `starlark-rust` does.

**[R-STAR-012]** Declaration order MUST be preserved. It is the tie-break the
component graph uses when two components have no dependency between them; see
[R-ENGINE-013].

## Loading

**[R-STAR-020]** `load()` MUST resolve through a composite loader covering the
four schemes `internal/starlark/loader/composite.go` handles: a path relative
to the config directory, `self://`, `user://`, `github://owner/repo@ref//path`,
and `@name//path` for a registry module.

**[R-STAR-021]** A registry URL MUST accept an omitted `.star` extension:
`@stdlib//components/apt` and `@stdlib//components/apt.star` MUST resolve to
the same file, and `@name` alone MUST resolve to the module's root
`init.star`.

**[R-STAR-022]** A module file MUST be integrity-checked before it is
evaluated; see [R-MODULE-030]. Evaluating first and checking afterwards means
the code already ran.

**[R-STAR-023]** The same module URL loaded twice within one command MUST be
evaluated once and cached, matching `v0.1.0`, so a component graph that loads a
shared helper from twenty components pays for it once.

## Calling hooks

**[R-STAR-030]** The evaluator MUST call a named global function in an
evaluated file, passing the `ctx` value as its single argument.

**[R-STAR-031]** A hook that is absent MUST be a success, not an error. Hooks
are optional and most components define two or three of the thirteen.

**[R-STAR-032]** A global that is present but not callable MUST be an error
naming the component and the hook.

## Diagnostics

**[R-STAR-040]** An evaluation error MUST carry the file, the line, the column,
and the source text of the offending line, so `meowctl-cli` can render a
diagnostic that points at it; see [R-COMMON-032] and [R-CLI-031].

**[R-STAR-041]** An error raised inside a `load()`ed module MUST carry the
chain of files that led to it, not only the innermost one.

**[R-STAR-042]** A Starlark `fail()` call MUST surface its message as a
configuration error and MUST exit with the configuration code.

## Failure paths

**[R-STAR-050]** A file that does not parse MUST report the syntax error with
its position and MUST NOT partially apply what came before it.

**[R-STAR-051]** A builtin called with wrong or missing arguments MUST name the
builtin, the argument, and what was expected.

**[R-STAR-052]** An evaluation MUST be bounded: a configuration that loops
forever MUST be interruptible. `v0.1.0` inherits `go.starlark.net`'s step
limit; the equivalent here is whatever M0 finds.

**[R-STAR-053]** A `load()` of a module that cannot be fetched MUST exit with
the module code, not the configuration code; see [R-COMMON-030].

## Parity with v0.1.0

The predeclared set, the builtin signatures, the registry URL rules, the
platform struct, and the `select` matching are all reproduced from
`internal/starlark/`. The `json` module comes from the Starlark standard
library in both implementations.

No behaviour is deliberately changed. Where `starlark-rust` and
`go.starlark.net` differ in a way a component can observe, the difference is a
finding for M0 and its resolution is an amendment to this spec, not a silent
divergence.

## Open questions

All four wait on M0, and each can change the design rather than only this spec:

1. How `load()` interception maps onto `starlark-rust`'s loader, and whether
   the composite loader can stay a single trait object.
2. Whether a custom value with attributes can carry the borrowed engine state
   `ctx` needs, or whether `ctx` has to hold handles instead.
3. How per-evaluation state reaches a builtin, and what lifetime it takes.
4. Whether runtime error messages differ in text. They appear in snapshots and
   in user-facing output, so a difference is a parity finding even when the
   behaviour matches.
