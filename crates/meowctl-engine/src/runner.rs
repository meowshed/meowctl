//! Running a plan.
//!
//! Takes a [`Plan`] and the effects and executes exactly it, so a dry run and
//! a real run cannot disagree about what work there is; see [REQ-3003, REQ-3101, REQ-3102].
//!
//! Nothing here holds a renderer, takes one as a field, or formats output.
//! `Runner` in `internal/lifecycle/runner.go` has a `Writer tui.Writer` field
//! and falls back to constructing one, and the cost of that is
//! `SuspendOutput`: a renderer concern threaded into the Starlark layer as a
//! callback; see [REQ-3051, REQ-3117, REQ-3118].

use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use meowctl_common::{ComponentId, Event, Level, Outcome, Phase};
use meowctl_ctx::{Capabilities, Ctx, Effects, Restricted, Surface};
use meowctl_pm::{Call, HandlerFailure, PmError, Registry};
use meowctl_starlark::{Argument, Evaluator, PackageAction, Platform};

use crate::discovery::Component;
use crate::graph::Graph;
use crate::plan::Plan;
use crate::progress::Progress;

/// What the caller is willing to have happen.
#[derive(Debug, Clone)]
pub struct Settings {
    /// `$HOME`.
    pub home: std::path::PathBuf,
    /// Where a component's persistent state goes, one directory per
    /// component; see [REQ-2803].
    pub state_root: std::path::PathBuf,
    /// The machine.
    pub platform: Platform,
    /// The process environment, for `ctx.env`.
    pub environment: BTreeMap<String, String>,
    /// Whether this run writes nothing. A property a hook can read; no method
    /// branches on it; see [REQ-2814].
    pub dry_run: bool,
    /// Whether a failure undoes what the run did; see [REQ-3032, REQ-3110].
    pub rollback: bool,
    /// The shell a runtime hook is contributing to, as `ctx.shell` reports
    /// it.
    ///
    /// `None` everywhere else, which is how a component tests whether it is
    /// being asked to contribute to a shell; see [REQ-2802, REQ-2900, REQ-2901].
    pub shell: Option<String>,
    /// Set when the user has asked the run to stop.
    ///
    /// Read between components and nowhere else. The engine installs no
    /// signal handler: a signal arrives at the process, so whoever owns the
    /// process sets this and the runner reads it; see [REQ-3062, REQ-3121] and
    /// [REQ-3414].
    pub interrupted: Arc<AtomicBool>,
}

/// How a run went.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Report {
    /// Components that finished, in the order they did, with how.
    pub finished: Vec<(Phase, ComponentId, Outcome)>,
    /// The first failure, which is where the run stopped; see [REQ-3030, REQ-3108].
    pub failure: Option<Failure>,
    /// How the rollback went, when one ran.
    ///
    /// `None` when nothing failed, or when the caller turned rollback off.
    pub rolled_back: Option<RolledBack>,
    /// Whether the run stopped because the user asked it to.
    pub interrupted: bool,
    /// What each component declared, for the components whose `install` or
    /// `upgrade` ran, keyed by logical name.
    ///
    /// Reported rather than written, because a lock file is the CLI's to
    /// write: nothing below `meowctl-cli` knows where the configuration
    /// directory is. `pkgsPins` is the same collection; see [REQ-1225, REQ-1309].
    pub packages: BTreeMap<String, Vec<meowctl_starlark::PackageDecl>>,
}

/// How undoing a failed run went.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RolledBack {
    /// Every inverse applied, some, or none; see [REQ-2024].
    pub outcome: meowctl_ops::Outcome,
    /// How many applied.
    pub applied: usize,
    /// What each failed inverse was and why, so a user knows what is left
    /// behind; see [REQ-3061, REQ-3120].
    pub failures: Vec<String>,
}

/// What failed, and where.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Failure {
    /// The phase it was in.
    pub phase: Phase,
    /// The component whose hook failed.
    pub component: ComponentId,
    /// What it said.
    pub reason: String,
}

impl std::fmt::Display for Failure {
    /// Names the component, the phase, and the underlying error; see
    /// [REQ-3060].
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} failed in {}: {}",
            self.component, self.phase, self.reason
        )
    }
}

impl Report {
    /// Whether everything that was planned ran.
    #[must_use]
    pub const fn succeeded(&self) -> bool {
        self.failure.is_none()
    }
}

/// Runs a plan.
pub struct Runner<'a> {
    graph: &'a Graph,
    registry: &'a Registry,
    effects: Effects,
    settings: Settings,
    evaluator: Evaluator<'a>,
    /// What has been done, written as it is done; see [REQ-3041].
    ///
    /// Absent when the caller is not tracking it, which is what a dry run
    /// and a one-off `verify` do.
    progress: Option<Progress>,
}

impl std::fmt::Debug for Runner<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Runner")
            .field("components", &self.graph.components().len())
            .field("managers", &self.registry.managers())
            .finish_non_exhaustive()
    }
}

impl<'a> Runner<'a> {
    /// A runner over this graph, with these effects.
    #[must_use]
    pub fn new(
        graph: &'a Graph,
        registry: &'a Registry,
        evaluator: Evaluator<'a>,
        effects: Effects,
        settings: Settings,
    ) -> Self {
        Runner {
            graph,
            registry,
            effects,
            settings,
            evaluator,
            progress: None,
        }
    }

    /// Records what finishes, in this file.
    #[must_use]
    pub fn recording(mut self, progress: Progress) -> Self {
        self.progress = Some(progress);
        self
    }

    /// What was recorded, for a caller that wants to write something beside
    /// it.
    #[must_use]
    pub const fn progress(&self) -> Option<&Progress> {
        self.progress.as_ref()
    }

    /// Runs the plan, stopping at the first component that fails.
    ///
    /// A phase set runs its phases in order and stops at the first failed
    /// phase, and a phase stops at the first failed component: a component
    /// that fails is usually one that everything after it needs, and fifty
    /// errors about a tool that never installed bury the one that matters;
    /// see [REQ-3030, REQ-3108] and [REQ-3031, REQ-3109].
    pub fn run(&mut self, plan: &Plan) -> Report {
        self.effects.emit(Event::PlanComputed {
            phase_set: plan.phase_set,
            steps: plan.steps.clone(),
        });

        let mut report = Report::default();
        let by_name: BTreeMap<&str, &Component> = self
            .graph
            .components()
            .iter()
            .map(|c| (c.logical_name(), c))
            .collect();

        for phase in plan.phase_set.phases().iter().copied() {
            // Whether a command runs during a dry run depends on whether the
            // phase is read-only, and this is the only place that knows both
            // the flag and the phase; see [REQ-3034].
            let phase_effects = self.effects_for(phase);
            self.effects.emit(Event::PhaseStarted {
                phase,
                total: plan.running_in(phase),
            });

            let mut failed = 0;
            for step in plan.steps.iter().filter(|s| s.phase == phase) {
                // Between components, which is the whole of what an
                // interrupt can stop: a hook halfway through `brew install`
                // is not ours to interrupt, and the terminal has already sent
                // the subprocess its own; see [REQ-3062, REQ-3121].
                if self.settings.interrupted.load(Ordering::Relaxed) {
                    report.interrupted = true;
                    break;
                }
                if let Some(reason) = &step.skipped {
                    self.effects.emit(Event::ComponentSkipped {
                        component: step.component.clone(),
                        phase,
                        reason: reason.clone(),
                    });
                    continue;
                }
                let Some(component) = by_name.get(step.component.logical_name()) else {
                    continue;
                };

                self.effects.emit(Event::ComponentStarted {
                    component: component.id.clone(),
                    phase,
                });
                let outcome = self.run_component(&phase_effects, component, phase);
                self.effects.emit(Event::ComponentFinished {
                    component: component.id.clone(),
                    phase,
                    outcome: outcome.clone(),
                });
                report
                    .finished
                    .push((phase, component.id.clone(), outcome.clone()));

                // What a component declared, once its install or upgrade has
                // run. Only those two phases, and only a component that has
                // declarations, which is what `pkgsPins` records; see
                // [REQ-1225, REQ-1309].
                if matches!(phase, Phase::Install | Phase::Upgrade)
                    && !matches!(outcome, Outcome::Failed { .. })
                    && !component.evaluated.declarations.packages.is_empty()
                {
                    report
                        .packages
                        .entry(component.logical_name().to_owned())
                        .or_default()
                        .extend(component.evaluated.declarations.packages.iter().cloned());
                }

                // Recorded as soon as it succeeds, so a run interrupted at
                // the next component resumes here rather than at the start of
                // the phase; see [REQ-3041].
                if !matches!(outcome, Outcome::Failed { .. })
                    && let Some(progress) = self.progress.as_mut()
                    && let Err(e) = progress.record(
                        self.effects.fs.as_ref(),
                        phase.as_str(),
                        component.logical_name(),
                    )
                {
                    self.effects.emit(Event::Message {
                        level: Level::Warn,
                        text: format!("could not record what finished: {e}"),
                    });
                }

                if let Outcome::Failed { error } = outcome {
                    failed = 1;
                    report.failure = Some(Failure {
                        phase,
                        component: component.id.clone(),
                        reason: error,
                    });
                    break;
                }
            }

            self.effects.emit(Event::PhaseFinished { phase, failed });
            // A phase set stops at the first failed phase; see
            // [REQ-3031, REQ-3109].
            if report.failure.is_some() || report.interrupted {
                break;
            }
        }

        // An interrupted run keeps its journal for the next run to find and
        // report. Undoing work the user stopped is not what stopping asked
        // for; see [REQ-3064].
        if report.failure.is_some() && !report.interrupted && self.settings.rollback {
            report.rolled_back = self.roll_back();
        }
        report
    }

    /// Runs one runtime hook phase over every component, in graph order.
    ///
    /// Not `run`, and not given a [`Plan`], because the runtime hook phases
    /// belong to no phase set and there is nothing to plan: `shell` runs on
    /// every shell spawn, so skipping a component because it ran last time
    /// would mean a shell without its integration. Nothing is recorded,
    /// nothing is journalled, and nothing is rolled back; see
    /// [REQ-3035, REQ-3112, REQ-3113, REQ-3114, REQ-3115].
    ///
    /// The only events it emits are the ones a hook produces, so a sink
    /// writing shell code sees shell code and nothing else; see [REQ-3461, REQ-3521].
    pub fn run_hook(&mut self, phase: Phase) -> Report {
        let mut report = Report::default();
        let effects = self.effects.clone();

        for component in self.graph.components() {
            let outcome = self.run_component(&effects, component, phase);
            report
                .finished
                .push((phase, component.id.clone(), outcome.clone()));

            if let Outcome::Failed { error } = outcome {
                report.failure = Some(Failure {
                    phase,
                    component: component.id.clone(),
                    reason: error,
                });
                break;
            }
        }
        report
    }

    /// Undoes what the run did.
    ///
    /// Reported rather than returned as an error: a rollback that partially
    /// succeeded still leaves the original failure as the thing that went
    /// wrong, and the user needs to know both; see [REQ-3061, REQ-3120].
    fn roll_back(&mut self) -> Option<RolledBack> {
        let journal = self.effects.journal.clone()?;
        let journal = journal.lock().ok()?;

        let mut sink = |event| self.effects.emit(event);
        let replay = meowctl_ops::replay(
            &journal,
            self.effects.fs.as_ref(),
            self.effects.exec.as_ref(),
            &mut sink,
        )
        .ok()?;

        let failures: Vec<String> = replay
            .failures
            .iter()
            .map(|(record, error)| format!("{} {}: {error}", record.kind, record.component))
            .collect();
        for failure in &failures {
            self.effects.emit(Event::Message {
                level: Level::Warn,
                text: format!("could not undo {failure}"),
            });
        }

        Some(RolledBack {
            outcome: replay.outcome,
            applied: replay.applied,
            failures,
        })
    }

    /// The effects a phase runs against.
    ///
    /// The same effects, with an executor built for this phase when the run
    /// writes nothing; see [REQ-3034].
    fn effects_for(&self, phase: Phase) -> Effects {
        if !self.settings.dry_run {
            return self.effects.clone();
        }
        Effects {
            exec: std::sync::Arc::new(meowctl_exec::DryRunExecutor::new(
                Box::new(std::sync::Arc::clone(&self.effects.exec)),
                phase,
            )),
            ..self.effects.clone()
        }
    }

    /// Runs one component's hook for one phase.
    fn run_component(&self, effects: &Effects, component: &Component, phase: Phase) -> Outcome {
        let hook = phase.as_str();
        // `query_pm` inside this hook runs a handler with this component's
        // `ctx`, so the registry it reaches is installed per component rather
        // than once for the run; see [REQ-2620].
        self.evaluator
            .set_package_managers(std::sync::Arc::new(Interrogator {
                handlers: self.handler_sources(),
                effects: effects.clone(),
                capabilities: self.capabilities(component, phase),
                platform: self.settings.platform.clone(),
            }));

        let ctx = Ctx::new(self.capabilities(component, phase), effects.clone());
        let argument = Restricted::new(ctx, Surface::for_phase(phase));

        let (evaluated, called) = match self.evaluator.call_hook_collecting(
            component.id.as_str(),
            &component.source,
            hook,
            &argument,
        ) {
            Ok(result) => result,
            Err(e) => {
                return Outcome::Failed {
                    error: e.to_string(),
                };
            }
        };

        // Declaration order within a component, and the graph's order between
        // them, which is the order the caller is iterating in; see
        // [REQ-2615, REQ-2703].
        for declaration in &evaluated.declarations.packages {
            if let Err(e) = self.dispatch(effects, component, phase, declaration) {
                return Outcome::Failed {
                    error: e.to_string(),
                };
            }
        }
        for declaration in &evaluated.declarations.repos {
            let call = match self.registry.call_for_repo(declaration) {
                Ok(call) => call,
                Err(e) => {
                    return Outcome::Failed {
                        error: e.to_string(),
                    };
                }
            };
            if let Err(e) =
                self.call_handler(effects, component, phase, &call, &declaration.manager)
            {
                return Outcome::Failed {
                    error: e.to_string(),
                };
            }
        }

        // An absent hook is a success with nothing to do, not a skip:
        // absence means the component has nothing to do in this phase; see
        // [REQ-3033, REQ-3111] and [REQ-2231].
        if called {
            Outcome::Succeeded
        } else {
            Outcome::NothingToDo
        }
    }

    /// Sends one package declaration to whatever handles its manager.
    fn dispatch(
        &self,
        effects: &Effects,
        component: &Component,
        phase: Phase,
        declaration: &meowctl_starlark::PackageDecl,
    ) -> Result<(), PmError> {
        // An uninstall declaration in an install phase is not this phase's
        // business, which is what `Dispatch`'s phase switch decides.
        let wanted = match declaration.action {
            PackageAction::Install => matches!(
                phase,
                Phase::Install | Phase::InstallConfigure | Phase::Upgrade
            ),
            PackageAction::Uninstall => {
                matches!(phase, Phase::Uninstall | Phase::UninstallCleanup)
            }
            PackageAction::Update => matches!(phase, Phase::Update | Phase::Upgrade),
        };
        if !wanted {
            return Ok(());
        }

        let call = self.registry.call_for(declaration)?;
        self.call_handler(effects, component, phase, &call, &declaration.name)
    }

    /// Calls one handler function.
    ///
    /// The handler's file is evaluated and its function called with the `ctx`
    /// of the component that asked for the package, not of the handler: the
    /// handler's effects belong to whoever wanted them; see [REQ-2614].
    fn call_handler(
        &self,
        effects: &Effects,
        component: &Component,
        phase: Phase,
        call: &Call,
        subject: &str,
    ) -> Result<(), PmError> {
        let Some(handler) = self
            .graph
            .components()
            .iter()
            .find(|c| c.logical_name() == call.component)
        else {
            return Err(PmError::NoHandler {
                manager: call.manager.clone(),
                registered: self.registry.managers(),
            });
        };

        let ctx = Ctx::new(self.capabilities(component, phase), effects.clone());
        let argument = HandlerCall {
            ctx: Restricted::new(ctx, Surface::for_phase(phase)),
            positional: call.positional.clone(),
            keyword: call.keyword.clone(),
        };

        self.evaluator
            .call_hook(
                handler.id.as_str(),
                &handler.source,
                call.function,
                &argument,
            )
            .map(|_| ())
            .map_err(|e| {
                PmError::from(HandlerFailure {
                    manager: call.manager.clone(),
                    package: subject.to_owned(),
                    component: component.logical_name().to_owned(),
                    handler: call.component.clone(),
                    function: call.function.to_owned(),
                    reason: e.to_string(),
                })
            })
    }

    /// Which component holds each manager's handler, and its source.
    fn handler_sources(&self) -> BTreeMap<String, (String, String)> {
        self.registry
            .managers()
            .into_iter()
            .filter_map(|manager| {
                let call = self.registry.call_for_interrogate(&manager).ok()?;
                let handler = self
                    .graph
                    .components()
                    .iter()
                    .find(|c| c.logical_name() == call.component)?;
                Some((
                    manager,
                    (handler.id.as_str().to_owned(), handler.source.clone()),
                ))
            })
            .collect()
    }

    /// What this component's `ctx` knows.
    fn capabilities(&self, component: &Component, phase: Phase) -> Capabilities {
        Capabilities {
            home: self.settings.home.clone(),
            dry_run: self.settings.dry_run,
            component_dir: component.directory.clone(),
            state_dir: self.settings.state_root.join(component.logical_name()),
            // Set only in a runtime hook phase, which is how a component
            // tests whether it is being asked to contribute to a shell; see
            // [REQ-2802, REQ-2900, REQ-2901].
            shell: phase
                .is_runtime_hook()
                .then(|| self.settings.shell.clone())
                .flatten(),
            platform: self.settings.platform.clone(),
            environment: self.settings.environment.clone(),
            phase,
            component: component.logical_name().to_owned(),
        }
    }
}

/// Answers `query_pm` by running a handler's `interrogate`.
///
/// It carries the asking component's capabilities, because `v0.1.0` passes
/// the caller's `ctx` to `interrogate` and a handler that reads
/// `ctx.component_dir` would otherwise see the handler's own; see
/// [REQ-2620].
///
/// A `query_pm` inside an `interrogate` refuses rather than recursing: an
/// interrogation that interrogates is a loop, and the evaluation it would need
/// is the one already running.
#[derive(Debug)]
struct Interrogator {
    handlers: BTreeMap<String, (String, String)>,
    effects: Effects,
    capabilities: Capabilities,
    platform: Platform,
}

impl meowctl_starlark::PackageManagers for Interrogator {
    fn interrogate(&self, manager: &str) -> meowctl_starlark::StarlarkResult<Vec<String>> {
        let refused = |reason: String| meowctl_starlark::StarlarkError::Evaluation {
            message: reason,
            span: None,
            load_chain: Vec::new(),
        };

        let Some((name, source)) = self.handlers.get(manager) else {
            return Err(refused(format!(
                "no component handles the package manager {manager}; registered: {}",
                if self.handlers.is_empty() {
                    "none".to_owned()
                } else {
                    self.handlers.keys().cloned().collect::<Vec<_>>().join(", ")
                }
            )));
        };

        let ctx = Ctx::new(self.capabilities.clone(), self.effects.clone());
        let argument = Restricted::new(ctx, Surface::for_phase(self.capabilities.phase));
        let loader = meowctl_starlark::NoLoader;
        let evaluator = Evaluator::new(self.platform.clone(), &loader);
        let (evaluated, called) =
            evaluator.call_hook_collecting(name, source, "interrogate", &argument)?;
        if !called {
            return Err(refused(format!("{name} exports no interrogate")));
        }

        // A handler returning something that is not a list of strings is a
        // defect in the handler rather than in the configuration that used
        // it, and coercing it would hide which; see [REQ-2632].
        evaluated.returned.ok_or_else(|| {
            refused(format!(
                "{name}'s interrogate returned something that is not a list of strings"
            ))
        })
    }
}

/// A handler call: the `ctx`, then the positional arguments, then the
/// keywords.
///
/// The hook-argument trait passes one value, and a handler takes several, so
/// the extras ride along here and the evaluator allocates them together.
#[derive(Debug)]
struct HandlerCall {
    ctx: Restricted,
    positional: Vec<String>,
    keyword: BTreeMap<String, Argument>,
}

impl meowctl_starlark::HookArgument for HandlerCall {
    fn allocate<'v>(&self, heap: starlark::values::Heap<'v>) -> starlark::values::Value<'v> {
        self.ctx.allocate(heap)
    }

    fn positional<'v>(&self, heap: starlark::values::Heap<'v>) -> Vec<starlark::values::Value<'v>> {
        self.positional
            .iter()
            .map(|text| heap.alloc(text.as_str()))
            .collect()
    }

    fn keywords<'v>(
        &self,
        heap: starlark::values::Heap<'v>,
    ) -> Vec<(String, starlark::values::Value<'v>)> {
        self.keyword
            .iter()
            .map(|(name, argument)| (name.clone(), allocate_argument(argument, heap)))
            .collect()
    }
}

/// Puts a flattened keyword argument back on a heap.
///
/// The accumulator copies a value out of the evaluation that made it, because
/// a Starlark value cannot leave its heap; see [REQ-2211]. A handler runs in
/// a different evaluation, so it has to be copied back in.
fn allocate_argument<'v>(
    argument: &Argument,
    heap: starlark::values::Heap<'v>,
) -> starlark::values::Value<'v> {
    match argument {
        Argument::String(text) => heap.alloc(text.as_str()),
        Argument::Integer(number) => heap.alloc(*number),
        Argument::Boolean(flag) => starlark::values::Value::new_bool(*flag),
        Argument::List(items) => heap.alloc(items.clone()),
    }
}
