# Lifecycle engine

**Crate:** `meowctl-engine`
**Design:** `docs/design/0.2.0-rust-rewrite.md` §3, defects #1, #2, #9 and #11
**v0.1.0 equivalent:** `internal/lifecycle/` plus the orchestration half of `internal/cli`

## Scope

This component decides what happens and makes it happen: it discovers
components, builds their graph, computes a plan, runs the phases, tracks what
completed, and drives rollback when something fails. It emits
[`Event`](common.md)s and holds no renderer.

Most of it lives in `internal/cli` in `v0.1.0`, next to the cobra commands.
`lifecycle.go` is 962 lines and `apply.go` is 832, and between them they hold
the staleness computation, the sentinel bookkeeping, the two evaluation passes,
and the phase running. Defects #1 and #2 are that arrangement.

## Boundary

The staged pipeline, the `Plan`, the phase execution, the sentinel updates, and
the event stream.

## Stages

**[R-ENGINE-001]** Discovery, resolution, planning, and execution MUST be
distinct types, so a stage cannot be entered before the one it depends on has
produced its value. `v0.1.0` expresses the same ordering in comments, which is
how "pass one registers PM handlers" became something a caller can forget.

**[R-ENGINE-002]** `Plan` MUST be a value computed from the configuration, the
lock state, and the sentinel, without performing an effect. It MUST name the
phases to run, the components in each, and which are skipped and why.

**[R-ENGINE-003]** Execution MUST take a `Plan` and the effects. A dry run MUST
render the plan and MUST NOT execute it, which is how [R-FS-011] and
[R-CTX-014] add up to a guarantee rather than a convention.

## Discovery and the graph

**[R-ENGINE-010]** Components MUST be discovered by evaluating `init.star`,
then merging `local.star` when it exists, with a component declared in both
counting once.

**[R-ENGINE-011]** A component's dependencies MUST be expanded transitively, so
declaring an aggregate brings in what it needs. A component reached this way
and not declared by the configuration MUST be marked as such, because a
`meowctl remove` of one tool must not run the uninstall hook of the package
manager it was reached through; see [R-ENGINE-032].

**[R-ENGINE-012]** A component whose platform or distribution guard does not
match the current machine MUST be dropped from the graph.

The guards are top-level globals in the component's own file, not arguments to
`component()`: a `platforms` list of operating-system names, and a `distros`
list matched against the machine's distribution or its `ID_LIKE` value. A
component declaring neither runs everywhere, and a value that is not a list of
strings is ignored rather than refused, which is what `platformMatches` and
`distroMatches` do.

This is not the matching [R-STAR-008] describes. `select()` takes
`//platform:linux-debian` conditions and matches `ID_LIKE` by substring; a
guard takes a bare `linux` and matches by equality. An earlier draft of this
requirement said they were the same and they are not.

**[R-ENGINE-013]** Execution order MUST be a topological sort of the dependency
graph, with declaration order as the tie-break between components that have no
dependency between them. `TopoSort` in `internal/lifecycle/toposort.go` takes
the declaration-order map for exactly this.

**[R-ENGINE-014]** A component's `after` list MUST be read from both the
`component()` declaration and the component file's own top-level `after`
global, and a name in it that the configuration does not declare MUST be
pulled into the graph.

An earlier draft of this requirement said the opposite: that `after` orders
without depending, and that an unknown name is ignored. Neither is what
`expandWithFileDeps` does, and the comment above it says why — a component
declaring `after = ["@stdlib//components/apt"]` is relying on apt being
installed, and requiring it to be named twice would make every component file
a change to `init.star` as well.

**[R-ENGINE-015]** A cycle MUST be reported with the components on it, and MUST
exit with the configuration code. `TopoSort` reports only that there is one,
which leaves a user with a hundred components and no way to find the two that
point at each other.

**[R-ENGINE-016]** A filter naming components MUST restrict the run to those
components and their dependencies, and a name matching nothing MUST be an
error rather than an empty run.

## Two passes

**[R-ENGINE-020]** Every component MUST be evaluated once to collect its
globals and register package-manager handlers, before any hook runs. See
[R-PM-003].

**[R-ENGINE-021]** The second pass MUST call hooks in graph order, reusing the
globals from the first rather than re-evaluating.

## Phases

**[R-ENGINE-030]** A phase MUST run its hook for every component in order, and
MUST stop at the first component that fails.

An earlier draft of this requirement said the opposite: collect every failure
and report them together. `RunPhase` used to do that, and
`fix(lifecycle): fail-fast on component failure and trigger rollback` changed
it, because a component failing early makes every component that depended on
it fail too -- fifty tools reporting "mise: executable file not found" when
mise is what failed. `PhaseError` still carries a list because one failure is
a list of one.

**[R-ENGINE-031]** A phase set MUST run its phases in the order
[R-COMMON-011] gives, and MUST stop at the first failed phase.

**[R-ENGINE-032]** A failed phase set MUST trigger rollback unless the caller
disabled it, and the outcome MUST be recorded; see [R-OPS-024].

**[R-ENGINE-033]** A component whose hook is absent MUST be recorded as
completed, not skipped. Absence means the component has nothing to do in this
phase, which is a success; see [R-STAR-031].

It MUST also be reported as finishing with nothing to do rather than as
succeeding, so a reader can tell a component that did work from one that had
none.

**[R-ENGINE-034]** During a dry run the engine MUST give each phase an
executor built for that phase, because whether a command runs depends on
whether the phase is read-only; see [R-EXEC-011]. Choosing an implementation
per phase is not the same as a method branching on a flag: no code below
`meowctl-cli` asks whether this is a dry run while it is doing something, and
[R-CTX-014] still holds.

The engine is the only place that knows both the flag and the phase. The
command line knows the flag and not the phases; the executor knows the phase
it was built for and not the flag.

## Sentinel and staleness

**[R-ENGINE-040]** A component already recorded as completed for a phase MUST
be skipped, unless the caller forced a re-run.

**[R-ENGINE-041]** A component MUST be recorded as completed immediately after
its hook succeeds, not at the end of the phase, so an interrupted run resumes
where it stopped.

**[R-ENGINE-042]** A non-empty rollback journal at startup MUST be reported as
an interrupted previous run before anything else happens; see [R-OPS-025].

**[R-ENGINE-043]** A component whose module fingerprint differs from what
`installed.lock` recorded MUST have its completion records cleared, so the next
run re-links it. Every component belonging to the changed module MUST be
cleared, including transitive ones `installed.lock` does not name directly.
`computeStaleComponents` in `internal/cli/apply.go` is the computation, and
`fix: correct module updates` is why it exists.

**[R-ENGINE-044]** Staleness MUST be computed from the fingerprint in
[R-CONFIG-033], so a GitHub module re-synced to a new commit invalidates even
though no version changed.

## Events

**[R-ENGINE-050]** The engine MUST emit `PlanComputed` before execution,
`PhaseStarted` and `PhaseFinished` around each phase, and `ComponentStarted`,
`ComponentSkipped` or `ComponentFinished` around each component.

**[R-ENGINE-051]** The engine MUST NOT hold a renderer, MUST NOT take one as a
field, and MUST NOT format output. `Runner` in `internal/lifecycle/runner.go`
has a `Writer tui.Writer` field and falls back to constructing one; neither MUST
exist here. This is defect #11.

**[R-ENGINE-052]** Every skip MUST carry its reason: already completed,
filtered out, or platform mismatch. An absent hook is not a skip; see
[R-ENGINE-033]. A skip a user cannot explain is what made `v0.1.0`'s dry run
misleading.

## Failure paths

**[R-ENGINE-060]** A hook failing MUST name the component, the phase, and the
underlying error.

**[R-ENGINE-061]** A rollback that partially succeeds MUST report which
inverses failed and MUST record the `partial` outcome.

**[R-ENGINE-062]** A run interrupted by a signal MUST stop before starting the
next component, flush the sentinel, and leave the journal intact for the next
run to find.

**[R-ENGINE-063]** A component that fails MUST NOT prevent the phase from
reporting the components that already succeeded.

## Parity with v0.1.0

The phase sets, the topological sort with declaration-order tie-break, the
sentinel semantics, the staleness computation, and the rollback triggering all
come from `internal/lifecycle/` and `internal/cli/apply.go`.

Two requirements change behaviour deliberately. [R-ENGINE-016] makes a filter
that matches nothing an error; `filterDecls` returns an error already, and this
states it as an obligation rather than an implementation detail.
[R-ENGINE-052] requires a reason on every skip, which `v0.1.0` does not carry,
and which the plan needs in order to be worth printing.
