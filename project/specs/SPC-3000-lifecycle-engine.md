---
id: SPC-3000
artifact: spec
status: live
revised: 2026-09-27
checked-at:
states: [REQ-3001, REQ-3002, REQ-3003, REQ-3010, REQ-3011, REQ-3012, REQ-3013, REQ-3014, REQ-3015, REQ-3016, REQ-3017, REQ-3018, REQ-3020, REQ-3021, REQ-3030, REQ-3031, REQ-3032, REQ-3033, REQ-3034, REQ-3035, REQ-3040, REQ-3041, REQ-3042, REQ-3043, REQ-3044, REQ-3050, REQ-3051, REQ-3052, REQ-3060, REQ-3061, REQ-3062, REQ-3063, REQ-3064, REQ-3100, REQ-3101, REQ-3102, REQ-3103, REQ-3104, REQ-3105, REQ-3106, REQ-3107, REQ-3108, REQ-3109, REQ-3110, REQ-3111, REQ-3112, REQ-3113, REQ-3114, REQ-3115, REQ-3116, REQ-3117, REQ-3118, REQ-3120, REQ-3121, REQ-3140, REQ-3141, REQ-3142, REQ-3172]
---

# Lifecycle engine

## Scope

The lifecycle engine lives in the `meowctl-engine` crate, its design is
`docs/design/0.2.0-rust-rewrite.md` §3 with defects #1, #2, #9 and #11, and its
`v0.1.0` equivalent is `internal/lifecycle/` plus the orchestration half of
`internal/cli` (from docs/spec/engine.md, high).

This component decides what happens and makes it happen: it discovers
components, builds their graph, computes a plan, runs the phases, tracks what
completed, and drives rollback when something fails (from docs/spec/engine.md,
high). It emits `Event`s and holds no renderer; the event vocabulary belongs to
the common specification, and rendering belongs to the terminal output
specification (from docs/spec/engine.md, high).

Most of it lives in `internal/cli` in `v0.1.0`, next to the cobra commands:
`lifecycle.go` is 962 lines and `apply.go` is 832, and between them they hold
the staleness computation, the sentinel bookkeeping, the two evaluation passes
and the phase running (from docs/spec/engine.md, high). Defects #1 and #2 are
that arrangement (from docs/spec/engine.md, high).

## Boundary

The engine exposes the staged pipeline, the `Plan`, the phase execution, the
sentinel updates, and the event stream (from docs/spec/engine.md, high).

| Surface | What is observable |
| --- | --- |
| Staged pipeline | Discovery, the graph, the plan and the report as distinct types, `Discovered`, `Graph`, `Plan` and `Report` [REQ-3001] (from docs/spec/engine.md and crates/meowctl-engine/src, high) |
| `Plan` | The phases to run, the components in each, and each skip with its reason [REQ-3100] (from docs/spec/engine.md, high) |
| Sentinel | Completion records in the `completed_components` array of `state.toml` in the configuration directory, written through `Progress::record` as each component completes [REQ-3041] [REQ-1240] [REQ-1313] (from crates/meowctl-config/src/layout.rs:86-90 and crates/meowctl-engine/src/progress.rs:63-71, high) |
| Event stream | `PlanComputed`, `PhaseStarted`, `PhaseFinished`, `ComponentStarted`, `ComponentSkipped` and `ComponentFinished` [REQ-3050] (from docs/spec/engine.md, high) |

## Behaviour

### Stages

Discovery, resolution, planning and execution are distinct types, so a stage
cannot be entered before the one it depends on has produced its value [REQ-3001]
(from docs/spec/engine.md, high).

`Plan` is a value computed from the configuration, the lock state and the
sentinel without performing an effect [REQ-3002], and it names the phases to
run, the components in each, and which are skipped and why [REQ-3100] (from
docs/spec/engine.md, high).

Execution takes a `Plan` and the effects [REQ-3003]. A dry run renders the plan
[REQ-3101] and does not execute it [REQ-3102] (from docs/spec/engine.md, high).

### Discovery and the graph

The engine discovers components by evaluating `init.star`, then merging
`local.star` when it exists, and a component declared in both counts once
[REQ-3010] (from docs/spec/engine.md, high).

A component's dependencies are expanded transitively, so declaring an aggregate
brings in what it needs [REQ-3011]. A component reached this way and not
declared by the configuration is marked as such [REQ-3103] (from
docs/spec/engine.md, high).

A component whose platform or distribution guard does not match the current
machine is dropped from the graph [REQ-3012] (from docs/spec/engine.md, high).
The guards are top-level globals in the component's own file, not arguments to
`component()`: a `platforms` list of operating-system names, and a `distros`
list matched against the machine's distribution or its `ID_LIKE` value
[REQ-3012] (from docs/spec/engine.md, high). A component declaring neither runs
everywhere, and a value that is not a list of strings is ignored rather than
refused [REQ-3012] (from docs/spec/engine.md, high). A guard takes a bare
`linux` and matches by equality, unlike `select()`, which takes
`//platform:linux-debian` conditions and matches `ID_LIKE` by substring
[REQ-3012] (from docs/spec/engine.md, high).

Execution order is a topological sort of the dependency graph, with declaration
order as the tie-break between components that have no dependency between them
[REQ-3013] (from docs/spec/engine.md, high).

The engine reads a component's `after` list from both the `component()`
declaration and the component file's own top-level `after` global [REQ-3014],
and pulls into the graph any name in it that the configuration does not declare
[REQ-3104] (from docs/spec/engine.md, high).

A bare name in the `after` list of a module's component resolves inside that
module, as `<module>//components/<name>` [REQ-3018]. A bare name in the
configuration's own component resolves in the configuration [REQ-3018] (from
docs/spec/engine.md, high).

A filter naming components restricts the run to those components and their
dependencies [REQ-3016] (from docs/spec/engine.md, high).

The engine looks for a component named by a bare name as
`components/<name>.star` and then as `components/<name>/init.star` [REQ-3017].
Its `component_dir` is `components/<name>/` when that directory exists and
`components/` otherwise [REQ-3107] (from docs/spec/engine.md, high).

### Two passes

The first pass evaluates every component once to collect its globals and
register package-manager handlers, before any hook runs [REQ-3020] (from
docs/spec/engine.md, high).

The second pass calls hooks in graph order and reuses the globals from the first
pass, without re-evaluating [REQ-3021] (from docs/spec/engine.md, high).

### Phases

A phase runs its hook for every component in order [REQ-3030] and stops at the
first component that fails [REQ-3108] (from docs/spec/engine.md, high).

A phase set runs its phases in the order REQ-1011 gives [REQ-3031] and stops at
the first failed phase [REQ-3109] (from docs/spec/engine.md, high).

A failed phase set triggers rollback unless the caller disabled it [REQ-3032],
and the rollback outcome is recorded [REQ-3110] (from docs/spec/engine.md,
high).

A component whose hook is absent is recorded as completed, not skipped
[REQ-3033], and is reported as finishing with nothing to do rather than as
succeeding [REQ-3111] (from docs/spec/engine.md, high).

During a dry run the engine gives each phase an executor built for that phase,
because whether a command runs depends on whether the phase is read-only
[REQ-3034] (from docs/spec/engine.md, high).

A runtime hook phase runs over every component in graph order without a `Plan`
[REQ-3035] (from docs/spec/engine.md, high). It does not consult the sentinel
[REQ-3112], record what finished [REQ-3113], journal [REQ-3114] or roll back
[REQ-3115] (from docs/spec/engine.md, high).

### Sentinel and staleness

A component already recorded as completed for a phase is skipped, unless the
caller forced a re-run [REQ-3040] (from docs/spec/engine.md, high).

A component is recorded as completed immediately after its hook succeeds, not at
the end of the phase, so an interrupted run resumes where it stopped [REQ-3041]
(from docs/spec/engine.md, high).

A non-empty rollback journal at startup is reported as an interrupted previous
run before anything else happens [REQ-3042] (from docs/spec/engine.md, high).

A component whose module fingerprint differs from what `installed.lock` recorded
has its completion records cleared, so the next run re-links it [REQ-3043] (from
docs/spec/engine.md, high). Every component belonging to the changed module is
cleared, including transitive ones `installed.lock` does not name directly
[REQ-3116] (from docs/spec/engine.md, high).

Staleness is computed from the fingerprint in REQ-1233, so a GitHub module
re-synced to a new commit invalidates even though no version changed [REQ-3044]
(from docs/spec/engine.md, high).

### Events

The engine emits `PlanComputed` before execution, `PhaseStarted` and
`PhaseFinished` around each phase, and `ComponentStarted`, `ComponentSkipped` or
`ComponentFinished` around each component [REQ-3050] (from docs/spec/engine.md,
high).

The engine holds no renderer [REQ-3051], takes none as a field [REQ-3117] and
formats no output [REQ-3118], so it has no writer field and no fallback that
constructs a writer (from docs/spec/engine.md, high).

Every skip carries its reason: already completed, filtered out, or platform
mismatch [REQ-3052] (from docs/spec/engine.md, high).

### Dry runs

A dry run renders the plan and then runs each planned hook against the dry-run
effects [REQ-3172] (from
project/requirements/REQ-3172-dry-run-runs-hooks-against-dry-run-effects.md,
high).

## Failure paths

| Condition | What happens |
| --- | --- |
| The dependency graph has a cycle | The engine reports the cycle with the components on it [REQ-3015] and exits with the configuration code [REQ-3105] (from docs/spec/engine.md, high). |
| A filter name matches nothing | The run is an error, not an empty run [REQ-3106] (from docs/spec/engine.md, high). |
| A hook fails | The error names the component, the phase and the underlying error [REQ-3060] (from docs/spec/engine.md, high). The phase stops at that component [REQ-3108] and still reports the components that already succeeded [REQ-3063] (from docs/spec/engine.md, high). |
| A phase in a phase set fails | The phase set stops [REQ-3109] and triggers rollback unless the caller disabled it [REQ-3032] (from docs/spec/engine.md, high). |
| A rollback partially succeeds | The engine reports which inverses failed [REQ-3061] and records the `partial` outcome [REQ-3120] (from docs/spec/engine.md, high). |
| A signal interrupts the run | The run stops before starting the next component [REQ-3062], leaves the journal intact for the next run to find [REQ-3121], and does not roll back [REQ-3064] (from docs/spec/engine.md, high). The engine learns of the interrupt through a flag the caller owns and the runner reads, and the sentinel needs no flush [REQ-3062] (from docs/spec/engine.md, high). |
| A previous run left a non-empty journal | The engine reports an interrupted previous run before anything else happens [REQ-3042] (from docs/spec/engine.md, high). |
| A component fails to evaluate during discovery | The command stops before any phase and names the component [REQ-3141]. The exit code is the configuration code, or the module code when the cause is a module that couldn't be fetched [REQ-2253] (from crates/meowctl-engine/src/discovery.rs:205-265, high). |
| `local.star` fails to evaluate, or exists and can't be read | The command stops before any phase, with the configuration code and the file's diagnostic [REQ-3140]. |
| A hook fails during a dry run | The run reports the predicted failure, naming the component and the phase, and exits with the general code [REQ-3142]. A dry run runs hooks against the dry-run effects [REQ-3172]. |
