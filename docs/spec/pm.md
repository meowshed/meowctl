# Package managers

**Crate:** `meowctl-pm`
**Design:** `docs/design/0.2.0-rust-rewrite.md` §3
**v0.1.0 equivalent:** `internal/pkg/`

## Scope

This component owns the registry of package-manager handlers and the dispatch
of package declarations to them. A package manager is not code in the binary:
it is a component that exports a handful of functions, which is what lets
Homebrew, apt, and everything else live in the standard library rather than in
a match statement here.

## Boundary

Handler registration, the dispatch calls, and the errors they produce.

## Registration

**[R-PM-001]** A component MUST be registered as a handler when it exports all
four of `pm_name`, `install_pkg`, `uninstall_pkg`, and `interrogate`. A
component exporting some but not all of them MUST NOT be registered, and the
omission MUST be warned about, because it is nearly always a mistake.

**[R-PM-002]** `update_pkg` and `add_repo` MUST be optional.

**[R-PM-003]** Registration MUST happen during the first evaluation pass,
before any hook runs, so that a hook in the first component can declare a
package handled by the last. See [R-ENGINE-020].

**[R-PM-004]** Two components exporting the same `pm_name` MUST be reported
rather than silently resolved. `v0.1.0` overwrites the earlier handler, which
makes which one wins depend on evaluation order.

## Dispatch

**[R-PM-010]** A `pkg()` declaration MUST dispatch to `install_pkg(ctx, name,
version, **kwargs)` on the handler for its manager.

**[R-PM-011]** An `unpkg()` declaration MUST dispatch to `uninstall_pkg`, and a
`uppkg()` declaration to `update_pkg`.

**[R-PM-012]** When a handler exports no `update_pkg`, `uppkg()` MUST fall back
to `install_pkg(ctx, name, "latest", **kwargs)`. `DispatchUpdate` in
`internal/pkg/registry.go` is the fallback, and removing it would break every
handler that never defined an update path.

**[R-PM-013]** A `repo()` declaration MUST dispatch to `add_repo(ctx,
**kwargs)`, and MUST fail naming the manager when the handler exports none.

**[R-PM-014]** Dispatch MUST pass the calling component's `ctx`, so the
handler's effects are journaled against the component that asked for the
package rather than against the handler.

**[R-PM-015]** Declarations MUST be dispatched in declaration order within a
component, and components MUST be processed in the graph order; see
[R-ENGINE-013].

## Interrogation

**[R-PM-020]** `query_pm(manager)` MUST call the handler's `interrogate(ctx)`
and return its result to the caller. It runs during evaluation, on its own
evaluation context, which is what `builtinQueryPM` does with a separate thread.

**[R-PM-021]** `interrogate` MUST be callable during a dry run, because it
reads rather than writes; see [R-EXEC-011].

## Failure paths

**[R-PM-030]** A `pkg()` naming a manager with no registered handler MUST fail
naming the manager and listing the managers that are registered. A typo in a
manager name is the common cause and the list is what makes it obvious.

**[R-PM-031]** A handler function that raises MUST fail the component that
declared the package, not the handler component, and the message MUST name
both.

**[R-PM-032]** A handler returning a value of an unexpected type MUST be
reported as a handler defect naming the function, rather than being coerced.

## Parity with v0.1.0

The four required exports, the two optional ones, the `update_pkg` fallback,
and the separate evaluation for `interrogate` all come from
`internal/pkg/registry.go`.

[R-PM-004] changes behaviour: `v0.1.0`'s `Register` overwrites an existing
entry, so two components claiming `brew` produce whichever evaluated last.
Reporting it is better than a configuration whose meaning depends on graph
order.
