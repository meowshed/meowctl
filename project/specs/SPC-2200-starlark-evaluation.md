---
id: SPC-2200
artifact: spec
status: live
revised: 2026-09-27
checked-at:
states: [REQ-2201, REQ-2202, REQ-2203, REQ-2204, REQ-2205, REQ-2206, REQ-2207, REQ-2208, REQ-2210, REQ-2211, REQ-2212, REQ-2220, REQ-2221, REQ-2222, REQ-2223, REQ-2230, REQ-2231, REQ-2232, REQ-2233, REQ-2234, REQ-2240, REQ-2241, REQ-2242, REQ-2250, REQ-2251, REQ-2252, REQ-2253, REQ-2300, REQ-2301, REQ-2302, REQ-2303, REQ-2304, REQ-2305, REQ-2306, REQ-2307, REQ-2340, REQ-2341, REQ-2342, REQ-2343, REQ-2370]
---

# Starlark evaluation

## Scope

This component evaluates the user's configuration: the predeclared builtins, the
accumulator that collects what they declare, the resolution of `load()` through
the composite loader, and diagnostics that point at a line (from
docs/spec/starlark.md, high). It lives in the `meowctl-starlark` crate, its
design is in `docs/design/0.2.0-rust-rewrite.md` §3 and §5, and its `v0.1.0`
equivalent is `internal/starlark/` (from docs/spec/starlark.md, high).

It is the component the parity constraint binds most tightly: the Starlark
surface is what users touch, and `v0.2.0` changes the implementation underneath
it from `go.starlark.net` to `starlark-rust` without changing what a component
file may say (from docs/spec/starlark.md, high).

## Boundary

The predeclared set, the accumulated declarations, the load resolution, the
hook-calling interface, and the diagnostics (from docs/spec/starlark.md, high).

## Behaviour

### Predeclared builtins

The predeclared set is exactly what `makePredeclared` in
`internal/starlark/builtins.go` provides: `component`, `pkg`, `unpkg`, `uppkg`,
`repo`, `query_pm`, `dep`, `module`, `replace`, `select`, `platform` and the
`json` module [REQ-2201], with nothing added and nothing removed [REQ-2300]
(from docs/spec/starlark.md, high). The `json` module comes from the Starlark
standard library (from docs/spec/starlark.md, high).

`component(name, after = [], **kwargs)` records a declaration carrying the name,
the ordering hints and any extra keyword arguments, in declaration order; `name`
is required and is positional or named, and `after` is named only and is a list
of strings [REQ-2202] (from docs/spec/starlark.md, high).

`pkg(manager, name, version = "", **kwargs)` records a package declaration
[REQ-2203], and `unpkg` and `uppkg` take the same shape for removal and update
[REQ-2301] (from docs/spec/starlark.md, high). `manager` and `name` are both
required, each is given positionally or by keyword, and supplying one both ways
is an error [REQ-2302] (from docs/spec/starlark.md, high). The positional order
is `manager` before `name`, so `pkg("brew", "git")` installs git with Homebrew
[REQ-2203] (from docs/spec/starlark.md, high).

`repo(...)` records a repository declaration targeting a named manager
[REQ-2204] (from docs/spec/starlark.md, high).

`query_pm(manager)` calls the registered handler's `interrogate` on its own
evaluation and returns what that returns, which makes it the one builtin that
runs user code during evaluation [REQ-2205] (from docs/spec/starlark.md, high).

`dep`, `module` and `replace` read every manifest meowctl meets, a `deps.mod`
and a `MODULE.meow`, with one grammar [REQ-2206] (from docs/spec/starlark.md,
high). `module(name, version = "", compat = None)` accepts `compat` and records
it without acting on it [REQ-2303] (from docs/spec/starlark.md, high).

`platform()` returns a struct in the shape of `makePlatformStruct`, carrying the
operating system, the architecture and, on Linux, the distribution and its
`ID_LIKE` value [REQ-2207] (from docs/spec/starlark.md, high).

On Linux, the distribution fields hold the `ID`, `ID_LIKE` and `VERSION_ID`
values of `/etc/os-release` [REQ-2341].

`select(cases)` chooses by platform with the rules in `matchesPlatform` and
`matchesLinuxDistro`, and matches a distribution through `ID_LIKE`, so a Mint
machine matches a `debian` case [REQ-2208] (from docs/spec/starlark.md, high).

### The accumulator

Each evaluation collects its declarations in its own accumulator, so a second
evaluation doesn't see the first one's [REQ-2210] (from docs/spec/starlark.md,
high). A builtin reaches that per-evaluation state through `Evaluator::extra`,
which it downcasts [REQ-2210] (from docs/spec/starlark.md, high).

The accumulator holds owned data [REQ-2211] (from docs/spec/starlark.md, high).
`Module::with_temp_heap` scopes a module and its heap to a closure, so no
Starlark value outlives the evaluation that produced it, and declarations become
owned data at the evaluation boundary [REQ-2211] (from docs/spec/starlark.md,
high).

The accumulator keeps declaration order, which the component graph uses as the
tie-break between two components with no dependency between them [REQ-2212]
(from docs/spec/starlark.md, high).

### Loading

`load()` resolves through a composite loader covering five URL forms: a path
relative to the config directory, `self://`, `user://`,
`github://owner/repo@ref//path`, and `@name//path` for a registry module
[REQ-2220] (from docs/spec/starlark.md, high). The evaluator installs the loader
as a `FileLoader` through `Evaluator::set_loader`, and the composite loader is
one type that dispatches on the URL scheme [REQ-2220] (from
docs/spec/starlark.md, high).

A registry URL whose final segment holds no `.` resolves to `<path>/init.star`
inside the module, and a final segment holding a `.` is taken as written
[REQ-2221] (from docs/spec/starlark.md, high). So `@stdlib//components/apt`
loads `components/apt/init.star` and `@stdlib//components/apt.star` loads that
file (from docs/spec/starlark.md, high). `@name` alone resolves to the module's
root `init.star` [REQ-2304] (from docs/spec/starlark.md, high).

A module file is integrity-checked before it is evaluated [REQ-2222] (from
docs/spec/starlark.md, high).

A module URL loaded twice within one command is evaluated once and served from a
cache after that [REQ-2223] (from docs/spec/starlark.md, high).

### Calling hooks

The evaluator calls a named global function in an evaluated file, passing the
`ctx` value first and whatever else the caller supplies after it, positionally
and by keyword [REQ-2230] (from docs/spec/starlark.md, high). A lifecycle hook
takes `ctx` alone, and a package-manager handler takes
`install_pkg(ctx, name, version, **kwargs)` [REQ-2230] (from
docs/spec/starlark.md, high). The call goes through `Evaluator::eval_function`
[REQ-2230] (from docs/spec/starlark.md, high). A value passed to a hook that
isn't `Send + Sync` needs `alloc_complex_no_freeze`; `ctx` is made
`Send + Sync` so that it doesn't (from
docs/spec/starlark.md:218-225 and crates/meowctl-ctx/src/value.rs, high).

A hook that is absent is a success [REQ-2231] (from docs/spec/starlark.md,
high).

An evaluation reports the value of every top-level name bound to a string, as a
map, and `pm_name` is one of those names [REQ-2233] (from docs/spec/starlark.md,
high). An evaluation also reports the value of every top-level name bound to a
list of strings, `platforms` and `distros` among them [REQ-2234] (from
docs/spec/starlark.md, high).

## Failure paths

### Hooks

A global that is present but not callable is an error naming the component and
the hook [REQ-2232] (from docs/spec/starlark.md, high).

### Diagnostics

An evaluation error carries the file, the line, the column and the source text
of the offending line [REQ-2240] (from docs/spec/starlark.md, high). The library
supplies these: `Error::span()` returns a `FileSpan` such as
`broken.star:2:1-12`, and the `Display` form renders a traceback with a caret
under the offending source [REQ-2240] (from docs/spec/starlark.md, high).

A runtime error reports the same mistake `v0.1.0` reports, in the wording
`starlark-rust` produces [REQ-2370], because the wording isn't an interface a
component depends on (from docs/spec/starlark.md:200-203, high).

An error raised inside a `load()`ed module carries the chain of files that led
to it, not only the innermost one [REQ-2241] (from docs/spec/starlark.md, high).

A Starlark `fail()` call surfaces its message as a configuration error
[REQ-2242] and exits with the configuration code [REQ-2305] (from
docs/spec/starlark.md, high).

### Evaluation failures

A file that doesn't parse reports the syntax error with its position [REQ-2250]
and applies nothing of what came before it [REQ-2306] (from
docs/spec/starlark.md, high).

A builtin called with wrong or missing arguments names the builtin, the argument
and what it expected [REQ-2251] (from docs/spec/starlark.md, high).

A `select` whose cases match nothing and that carries no `//conditions:default`
fails at the call with a configuration error [REQ-2340] (from
crates/meowctl-starlark/src/builtins.rs:275-303, high). A condition string
`select` doesn't know matches nothing and isn't an error (from
crates/meowctl-starlark/src/platform.rs:52-70, high).

On Linux with no readable `/etc/os-release`, `platform()` returns empty
distribution fields and doesn't fail, so a distribution-specific case falls
through to its default [REQ-2342].

A `query_pm` call outside a hook fails with a configuration error saying that
`query_pm` works only inside a hook [REQ-2343].

Nothing inside the evaluation bounds a configuration that loops forever, and
nothing claims to [REQ-2307] (from docs/spec/starlark.md, high). The process
stops it from outside [REQ-2252]: the first interrupt sets a flag the engine
reads between components, which a looping evaluation never reaches, and the
second interrupt kills the process (from docs/spec/starlark.md, high).

A `load()` of a module that can't be fetched exits with the module code, not the
configuration code [REQ-2253] (from docs/spec/starlark.md, high). A load the
loader refuses surfaces as an evaluation error carrying the loader's message
[REQ-2253] (from docs/spec/starlark.md, high).
