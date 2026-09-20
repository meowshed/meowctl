//! The order components run in.
//!
//! A topological sort of the `after` edges, with declaration order as the
//! tie-break between components that have no edge between them. Two runs of
//! the same configuration produce the same order, which is what makes a plan
//! worth reading; see [R-ENGINE-013].

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use meowctl_common::ComponentId;

use crate::discovery::{Component, Discovered};
use crate::{EngineError, EngineResult};

/// The components to run, in the order to run them.
#[derive(Debug, Clone)]
pub struct Graph {
    order: Vec<Component>,
}

impl Graph {
    /// Orders what discovery found.
    ///
    /// # Errors
    ///
    /// [`EngineError::Cycle`], naming the components that could not be
    /// ordered; see [R-ENGINE-015].
    pub fn build(discovered: &Discovered) -> EngineResult<Graph> {
        let components = &discovered.components;

        // Declaration order is the tie-break, so it is the priority: a
        // component declared earlier goes first among those that are ready at
        // the same time; see [R-ENGINE-013].
        let priority: BTreeMap<&str, usize> = components
            .iter()
            .enumerate()
            .map(|(i, c)| (c.logical_name(), i))
            .collect();

        // An edge points from a component to what it must run after. A name
        // no component in the graph answers to is dropped rather than
        // refused: a guard may have removed it, and a machine that does not
        // run apt should not fail because something is ordered after it.
        let mut waiting_for: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
        for component in components {
            let depends: BTreeSet<&str> = component
                .after
                .iter()
                .map(|name| ComponentId::logical_of(name))
                .filter(|name| priority.contains_key(name))
                .filter(|name| *name != component.logical_name())
                .collect();
            waiting_for.insert(component.logical_name(), depends);
        }

        let mut ready: VecDeque<&str> = ordered_by_priority(
            waiting_for
                .iter()
                .filter(|(_, on)| on.is_empty())
                .map(|(name, _)| *name),
            &priority,
        )
        .into();

        let mut placed: Vec<&str> = Vec::with_capacity(components.len());
        let mut done: BTreeSet<&str> = BTreeSet::new();
        while let Some(name) = ready.pop_front() {
            placed.push(name);
            done.insert(name);

            // Whatever was waiting only for this one is ready now. The batch
            // is sorted before it is appended, so the traversal is stable
            // across identical inputs.
            let freed = waiting_for
                .iter()
                .filter(|(held, on)| !done.contains(*held) && on.contains(&name))
                .filter(|(_, on)| on.iter().all(|n| done.contains(n) || *n == name))
                .map(|(held, _)| *held)
                .collect::<Vec<_>>();
            for held in ordered_by_priority(freed.into_iter(), &priority) {
                if !ready.contains(&held) {
                    ready.push_back(held);
                }
            }
        }

        if placed.len() != components.len() {
            let on_it: Vec<ComponentId> = components
                .iter()
                .filter(|c| !done.contains(c.logical_name()))
                .map(|c| c.id.clone())
                .collect();
            return Err(EngineError::Cycle { on_it });
        }

        let by_name: BTreeMap<&str, &Component> =
            components.iter().map(|c| (c.logical_name(), c)).collect();
        Ok(Graph {
            order: placed
                .into_iter()
                .filter_map(|name| by_name.get(name).map(|c| (*c).clone()))
                .collect(),
        })
    }

    /// The components, in execution order.
    #[must_use]
    pub fn components(&self) -> &[Component] {
        &self.order
    }

    /// The order, by logical name, for a message or a test.
    #[must_use]
    pub fn names(&self) -> Vec<&str> {
        self.order.iter().map(Component::logical_name).collect()
    }

    /// The same graph restricted to these components and what they need.
    ///
    /// A name matching nothing is an error rather than an empty run, because
    /// an empty run reports success; see [R-ENGINE-016].
    ///
    /// # Errors
    ///
    /// [`EngineError::NoSuchComponent`], naming what was asked for.
    pub fn restricted_to(&self, names: &[String]) -> EngineResult<Graph> {
        if names.is_empty() {
            return Ok(self.clone());
        }

        let by_name: BTreeMap<&str, &Component> =
            self.order.iter().map(|c| (c.logical_name(), c)).collect();

        let mut wanted: BTreeSet<&str> = BTreeSet::new();
        let mut pending: VecDeque<&str> = VecDeque::new();
        for name in names {
            let logical = ComponentId::logical_of(name);
            let found = by_name
                .keys()
                .find(|held| **held == logical)
                .ok_or_else(|| EngineError::NoSuchComponent { name: name.clone() })?;
            pending.push_back(found);
        }

        // Naming a component brings in what it depends on, or the run fails
        // on a tool that was never installed; see [R-ENGINE-016].
        while let Some(name) = pending.pop_front() {
            if !wanted.insert(name) {
                continue;
            }
            let Some(component) = by_name.get(name) else {
                continue;
            };
            for after in &component.after {
                let logical = ComponentId::logical_of(after);
                if let Some(held) = by_name.keys().find(|k| **k == logical) {
                    pending.push_back(held);
                }
            }
        }

        Ok(Graph {
            order: self
                .order
                .iter()
                .filter(|c| wanted.contains(c.logical_name()))
                .cloned()
                .collect(),
        })
    }
}

/// Sorts a batch the way `sortByPriority` does: by declaration order, then
/// alphabetically for anything the map does not hold.
fn ordered_by_priority<'a>(
    names: impl Iterator<Item = &'a str>,
    priority: &BTreeMap<&str, usize>,
) -> Vec<&'a str> {
    let mut batch: Vec<&str> = names.collect();
    batch.sort_by(|a, b| match (priority.get(a), priority.get(b)) {
        (Some(x), Some(y)) => x.cmp(y),
        // Known before unknown, and alphabetical between two unknowns.
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => a.cmp(b),
    });
    batch
}
