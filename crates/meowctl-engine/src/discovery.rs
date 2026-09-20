//! What a configuration declares, and what each component exports.
//!
//! One pass over every component file, before any hook runs. It collects the
//! declarations, follows `after` to whatever they reach, reads the guards, and
//! registers the package-manager handlers, so a hook in the first component
//! can declare a package the last one handles; see [R-ENGINE-020] and
//! [R-PM-003].

use std::collections::{BTreeMap, VecDeque};

use meowctl_common::ComponentId;
use meowctl_pm::{Registration, Registry, scan};
use meowctl_starlark::{Evaluated, Evaluator, Loader, Platform};

use crate::{EngineError, EngineResult};

/// The global naming the operating systems a component runs on.
const PLATFORMS: &str = "platforms";

/// The global naming the distributions it runs on.
const DISTROS: &str = "distros";

/// The global a component file uses to name what it must run after.
const AFTER: &str = "after";

/// One component, as the first pass found it.
#[derive(Debug, Clone)]
pub struct Component {
    /// What the configuration called it.
    pub id: ComponentId,
    /// Components it must run after, as written.
    pub after: Vec<String>,
    /// What the first pass found in its file.
    ///
    /// Kept so the second pass calls a hook without evaluating the file
    /// again; see [R-ENGINE-021].
    pub evaluated: Evaluated,
    /// Whether the configuration declared it, or something reached it
    /// through an `after` list.
    ///
    /// A `meowctl remove` of one tool must not run the uninstall hook of the
    /// package manager it was reached through; see [R-ENGINE-011].
    pub declared: bool,
}

impl Component {
    /// The name an `after` list refers to it by; see [R-COMMON-006].
    #[must_use]
    pub fn logical_name(&self) -> &str {
        self.id.logical_name()
    }

    /// Whether this component runs on this machine.
    ///
    /// The guards are top-level globals in the component's own file. A
    /// component declaring neither runs everywhere, and a value that is not a
    /// list of strings is ignored rather than refused, which is what
    /// `platformMatches` and `distroMatches` do; see [R-ENGINE-012].
    #[must_use]
    pub fn runs_on(&self, platform: &Platform) -> bool {
        let platforms = self.evaluated.lists.get(PLATFORMS);
        let distros = self.evaluated.lists.get(DISTROS);

        let platform_ok = platforms.is_none_or(|names| names.iter().any(|n| n == &platform.os));
        // By equality against either the distribution or its `ID_LIKE`, and
        // not by the substring match `select()` uses; see [R-ENGINE-012].
        let distro_ok = distros.is_none_or(|names| {
            names
                .iter()
                .any(|n| n == &platform.distro || n == &platform.distro_like)
        });
        platform_ok && distro_ok
    }

    /// The guards it declares, for the message that says why it was dropped.
    #[must_use]
    pub fn guard(&self) -> String {
        let mut parts = Vec::new();
        if let Some(names) = self.evaluated.lists.get(PLATFORMS) {
            parts.push(format!("platforms = {}", names.join(", ")));
        }
        if let Some(names) = self.evaluated.lists.get(DISTROS) {
            parts.push(format!("distros = {}", names.join(", ")));
        }
        parts.join("; ")
    }
}

/// Everything the first pass produced.
#[derive(Debug)]
pub struct Discovered {
    /// Every component in the graph, in the order they were reached, with the
    /// guards already applied.
    pub components: Vec<Component>,
    /// Which component a guard dropped, and which guard.
    ///
    /// Kept so the plan can say why rather than leaving a component missing;
    /// see [R-ENGINE-052].
    pub excluded: Vec<(ComponentId, String)>,
    /// The package-manager handlers the components registered.
    pub registry: Registry,
}

/// Where a component's file comes from.
///
/// A trait rather than the module loader itself, so discovery is testable
/// with a map and this crate does not depend on the one that fetches.
pub trait Sources {
    /// The `component()` declarations, in declaration order, `init.star`
    /// first and then `local.star`; see [R-ENGINE-010].
    ///
    /// # Errors
    ///
    /// When an entry point cannot be read or evaluated.
    fn declared(&self) -> EngineResult<Vec<Declaration>>;

    /// The source of one component's file.
    ///
    /// # Errors
    ///
    /// When it cannot be read.
    fn source(&self, id: &ComponentId) -> EngineResult<String>;
}

/// One `component()` call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Declaration {
    /// The name as written, in one of the three forms.
    pub name: String,
    /// What the call said it must run after.
    pub after: Vec<String>,
}

impl Declaration {
    /// A declaration with no ordering hints.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Declaration {
            name: name.into(),
            after: Vec::new(),
        }
    }

    /// The same declaration, ordered after these.
    #[must_use]
    pub fn after(mut self, names: &[&str]) -> Self {
        self.after = names.iter().map(|n| (*n).to_owned()).collect();
        self
    }
}

/// Runs the first pass.
///
/// # Errors
///
/// [`EngineError::UnusableName`] when a declaration names something that is
/// not a component, [`EngineError::Configuration`] when a component file does
/// not evaluate, and [`EngineError::PackageManager`] when two components claim
/// one manager; see [R-PM-004].
pub fn discover(
    sources: &dyn Sources,
    loader: &dyn Loader,
    platform: &Platform,
) -> EngineResult<Discovered> {
    let evaluator = Evaluator::new(platform.clone(), loader);

    let mut components = Vec::new();
    let mut excluded = Vec::new();
    let mut registry = Registry::new();
    // Keyed by logical name, because a component declared in both
    // `init.star` and `local.star` counts once; see [R-ENGINE-010].
    let mut seen: BTreeMap<String, ()> = BTreeMap::new();

    let mut pending: VecDeque<(Declaration, bool)> = sources
        .declared()?
        .into_iter()
        .map(|decl| (decl, true))
        .collect();

    while let Some((declaration, declared)) = pending.pop_front() {
        let id: ComponentId = declaration
            .name
            .parse()
            .map_err(|e| EngineError::UnusableName {
                name: declaration.name.clone(),
                reason: format!("{e}"),
            })?;
        if seen.insert(id.logical_name().to_owned(), ()).is_some() {
            continue;
        }

        let source = sources.source(&id)?;
        let evaluated = evaluator
            .evaluate(&declaration.name, &source)
            .map_err(|e| EngineError::Configuration {
                path: declaration.name.clone(),
                reason: e.to_string(),
            })?;

        // Registration happens in this pass, so a hook in the first component
        // can declare a package the last one handles; see [R-PM-003].
        if let Registration::Handler(handler) = scan(id.logical_name(), &evaluated) {
            registry.register(handler)?;
        }

        // Both halves: what the `component()` call said, and what the file
        // says about itself; see [R-ENGINE-014].
        let mut after = declaration.after;
        if let Some(own) = evaluated.lists.get(AFTER) {
            for name in own {
                if !after.contains(name) {
                    after.push(name.clone());
                }
            }
        }

        let component = Component {
            id: id.clone(),
            after: after.clone(),
            evaluated,
            declared,
        };

        // A name the configuration does not declare is pulled in rather than
        // ignored; see [R-ENGINE-014].
        for name in &after {
            let logical = ComponentId::logical_of(name);
            if !seen.contains_key(logical) {
                pending.push_back((Declaration::new(name.clone()), false));
            }
        }

        if component.runs_on(platform) {
            components.push(component);
        } else {
            excluded.push((id, component.guard()));
        }
    }

    Ok(Discovered {
        components,
        excluded,
        registry,
    })
}
