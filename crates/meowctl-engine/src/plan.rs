//! What a run will do, as a value.
//!
//! Computed from the graph, the sentinel, and the filter, and performing no
//! effect. A dry run renders it and stops; a real run executes exactly it, so
//! the two cannot disagree about what work there is. `v0.1.0` printed one
//! thing and ran another, which is what `fix(apply): dry-run claimed work
//! that the runner skips` closed; see [R-ENGINE-002] and [R-ENGINE-003].

use meowctl_common::{ComponentId, Phase, PhaseSet, PlannedStep, SkipReason};
use meowctl_config::Sentinel;

use crate::discovery::Component;
use crate::graph::Graph;

/// What a run will do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    /// The set being run.
    pub phase_set: PhaseSet,
    /// Every step, in execution order: the phases in the set's order, and the
    /// components in graph order within each.
    pub steps: Vec<PlannedStep>,
}

/// What a plan is computed from.
///
/// A struct rather than five arguments, because four of them are optional and
/// a call site with four `None`s says nothing about which is which.
#[derive(Debug, Clone, Default)]
pub struct Inputs<'a> {
    /// What completed before, so a component is not run twice.
    pub sentinel: Option<&'a Sentinel>,
    /// The components the caller named, if any.
    pub filter: &'a [String],
    /// Components a guard dropped, with the guard, so the plan can say why
    /// rather than leaving them out; see [R-ENGINE-052].
    pub excluded: &'a [(ComponentId, String)],
    /// Whether to run everything regardless of what completed before.
    pub force: bool,
}

impl Plan {
    /// Computes the plan.
    ///
    /// Nothing here reads a file or runs a command: the plan is a function of
    /// what it is given, which is what makes a dry run a prediction rather
    /// than a description; see [R-ENGINE-002].
    #[must_use]
    pub fn compute(graph: &Graph, phase_set: PhaseSet, inputs: &Inputs<'_>) -> Plan {
        let in_scope = scope(graph, inputs.filter);
        let mut steps = Vec::new();

        for phase in phase_set.phases().iter().copied() {
            // A component a guard dropped is named once per phase, the same
            // as every other skip, so a reader counting lines gets the same
            // number either way; see [R-ENGINE-052].
            for (component, guard) in inputs.excluded {
                steps.push(PlannedStep {
                    phase,
                    component: component.clone(),
                    skipped: Some(SkipReason::PlatformMismatch {
                        guard: guard.clone(),
                    }),
                });
            }

            for component in graph.components() {
                steps.push(PlannedStep {
                    phase,
                    component: component.id.clone(),
                    skipped: skip_reason(component, phase, &in_scope, inputs),
                });
            }
        }

        Plan { phase_set, steps }
    }

    /// The steps that will actually run.
    pub fn running(&self) -> impl Iterator<Item = &PlannedStep> {
        self.steps.iter().filter(|s| s.skipped.is_none())
    }

    /// How many components will run in a phase.
    #[must_use]
    pub fn running_in(&self, phase: Phase) -> usize {
        self.running().filter(|s| s.phase == phase).count()
    }

    /// Whether there is anything to do.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.running().next().is_none()
    }
}

/// Which components the filter leaves in, by logical name.
///
/// `None` when there is no filter, which is not the same as "all of them":
/// a filter that matched nothing is refused before this, by
/// [`Graph::restricted_to`].
fn scope(graph: &Graph, filter: &[String]) -> Option<Vec<String>> {
    if filter.is_empty() {
        return None;
    }
    graph
        .restricted_to(filter)
        .ok()
        .map(|scoped| scoped.names().into_iter().map(str::to_owned).collect())
}

/// Why this component will not run in this phase, if it will not.
fn skip_reason(
    component: &Component,
    phase: Phase,
    in_scope: &Option<Vec<String>>,
    inputs: &Inputs<'_>,
) -> Option<SkipReason> {
    if let Some(names) = in_scope
        && !names.iter().any(|n| n == component.logical_name())
    {
        return Some(SkipReason::FilteredOut);
    }

    // `--force` re-runs everything, which is what `Runner.Force` does.
    if inputs.force {
        return None;
    }

    // A component whose phase is already recorded is skipped, and the plan
    // says so rather than listing it as pending work. Listing it is what made
    // the dry run report 120 components before and after a successful apply
    // alike; see [R-ENGINE-002].
    let completed = inputs
        .sentinel
        .is_some_and(|s| s.is_completed(phase.as_str(), component.logical_name()));
    completed.then_some(SkipReason::AlreadyCompleted)
}
