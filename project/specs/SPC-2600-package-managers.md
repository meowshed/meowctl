---
id: SPC-2600
artifact: spec
status: live
revised: 2026-09-27
checked-at:
states: [REQ-2601, REQ-2602, REQ-2603, REQ-2604, REQ-2610, REQ-2611, REQ-2612, REQ-2613, REQ-2614, REQ-2615, REQ-2620, REQ-2621, REQ-2630, REQ-2631, REQ-2632, REQ-2700, REQ-2701, REQ-2702, REQ-2703, REQ-2704]
---

# Package managers

## Scope

This component owns the registry of package-manager handlers and the dispatch of
package declarations to them. A package manager isn't code in the binary: it's a
component that exports a handful of functions, which is what lets Homebrew, apt
and everything else live in the standard library (from docs/spec/pm.md, high).

It lives in the crate `meowctl-pm`, its design is
`docs/design/0.2.0-rust-rewrite.md` §3, and its `v0.1.0` equivalent is
`internal/pkg/` (from docs/spec/pm.md, high).

## Boundary

Handler registration, the dispatch calls, and the errors they produce (from
docs/spec/pm.md, high).

| Export | Required | Called for |
| --- | --- | --- |
| `pm_name` | Yes | Naming the manager (from docs/spec/pm.md, high) |
| `install_pkg(ctx, name, version, **kwargs)` | Yes | `pkg()`, and `uppkg()` when `update_pkg` is absent (from docs/spec/pm.md, high) |
| `uninstall_pkg` | Yes | `unpkg()` (from docs/spec/pm.md, high) |
| `interrogate(ctx)` | Yes | `query_pm(manager)` (from docs/spec/pm.md, high) |
| `update_pkg` | No | `uppkg()` (from docs/spec/pm.md, high) |
| `add_repo(ctx, **kwargs)` | No | `repo()` (from docs/spec/pm.md, high) |

## Behaviour

### Registration

A component that exports all four of `pm_name`, `install_pkg`, `uninstall_pkg`
and `interrogate` is registered as a handler [REQ-2601] (from docs/spec/pm.md,
high). `update_pkg` and `add_repo` are optional [REQ-2602] (from
docs/spec/pm.md, high). Registration happens during the first evaluation pass,
before any hook runs, so a hook in the first component can declare a package
handled by the last [REQ-2603] (from docs/spec/pm.md, high).

### Dispatch

A `pkg()` declaration dispatches to `install_pkg(ctx, name, version, **kwargs)`
on the handler for its manager [REQ-2610] (from docs/spec/pm.md, high). An
`unpkg()` declaration dispatches to `uninstall_pkg`, and a `uppkg()` declaration
to `update_pkg` [REQ-2611] (from docs/spec/pm.md, high). When a handler exports
no `update_pkg`, `uppkg()` falls back to
`install_pkg(ctx, name, "latest", **kwargs)` [REQ-2612] (from docs/spec/pm.md,
high). A `repo()` declaration dispatches to `add_repo(ctx, **kwargs)` [REQ-2613]
(from docs/spec/pm.md, high).

Dispatch passes the calling component's `ctx`, so the handler's effects are
journaled against the component that asked for the package [REQ-2614] (from
docs/spec/pm.md, high). Declarations are dispatched in declaration order within
a component [REQ-2615], and components are processed in the graph order REQ-3013
defines [REQ-2703] (from docs/spec/pm.md, high).

### Interrogation

`query_pm(manager)` calls the handler's `interrogate(ctx)` during evaluation, on
its own evaluation context, and returns the result to the caller [REQ-2620]
(from docs/spec/pm.md, high). `interrogate` is callable during a dry run
[REQ-2621] (from docs/spec/pm.md, high).

## Failure paths

| Condition | What happens |
| --- | --- |
| A component exports some but not all of the four required functions | It isn't registered [REQ-2700], and the omission is warned about [REQ-2701] (from docs/spec/pm.md, high) |
| Two components export the same `pm_name` | The clash is reported, not silently resolved [REQ-2604] (from docs/spec/pm.md, high) |
| A `repo()` names a handler that exports no `add_repo` | The declaration fails, naming the manager [REQ-2702] (from docs/spec/pm.md, high) |
| A `pkg()` names a manager with no registered handler | It fails, naming the manager and listing the registered managers [REQ-2630] (from docs/spec/pm.md, high) |
| A `repo()` names a manager with no registered handler | The declaring component fails in the phase the declaration is dispatched in, naming the manager and listing the registered managers [REQ-2630] (from crates/meowctl-pm/src/registry.rs:145-215, high) |
| A `query_pm` in a hook names a manager with no registered handler | The calling hook raises, naming the manager and listing the registered managers [REQ-2630] (from crates/meowctl-engine/src/runner.rs:603-640, high) |
| A handler function raises | The component that declared the package fails, not the handler component [REQ-2631], and the message names both [REQ-2704] (from docs/spec/pm.md, high) |
| `interrogate` raises during `query_pm` | The calling hook fails, and one message names the manager, the handler component and the calling component [REQ-2631] [REQ-2704] |
| A handler returns a value of an unexpected type | It is reported as a handler defect naming the function, without coercion [REQ-2632] (from docs/spec/pm.md, high) |
