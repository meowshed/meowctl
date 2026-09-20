//! Registration and dispatch.

use std::collections::BTreeMap;

use meowctl_starlark::{Argument, Evaluated, PackageAction, PackageDecl, RepoDecl};

use crate::{PmError, PmResult};

/// The global naming the manager a component handles.
const PM_NAME: &str = "pm_name";

/// The functions a handler must export; see [R-PM-001].
const REQUIRED: [&str; 3] = ["install_pkg", "uninstall_pkg", "interrogate"];

/// The optional ones; see [R-PM-002].
const UPDATE_PKG: &str = "update_pkg";
const ADD_REPO: &str = "add_repo";

/// What a component exports, once it is known to handle a manager.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Handler {
    /// The manager it handles, from `pm_name`.
    pub manager: String,
    /// The component that exports it.
    pub component: String,
    /// Whether it exports `update_pkg`, which decides whether `uppkg()` falls
    /// back to installing `latest`; see [R-PM-012].
    pub has_update: bool,
    /// Whether it exports `add_repo`, without which a `repo()` has nowhere to
    /// go; see [R-PM-013].
    pub has_add_repo: bool,
}

/// What scanning a component's exports found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Registration {
    /// It handles a manager.
    Handler(Handler),
    /// It names a manager and does not export everything a handler needs.
    ///
    /// Reported rather than ignored, because it is nearly always a mistake;
    /// see [R-PM-001].
    Incomplete {
        /// The manager it named.
        manager: String,
        /// What it did not export, in the order [R-PM-001] lists them.
        missing: Vec<String>,
    },
    /// It names no manager, which is what almost every component does.
    NotAHandler,
}

/// Reads a component's exports for a handler.
///
/// `ScanGlobals` in `internal/pkg/registry.go` is the same reading, against
/// the live globals rather than against what the evaluation reported.
#[must_use]
pub fn scan(component: &str, evaluated: &Evaluated) -> Registration {
    let Some(manager) = evaluated.strings.get(PM_NAME) else {
        // A `pm_name` that is not a string is not a manager name, and
        // `v0.1.0` warns and moves on. Here it is indistinguishable from a
        // component with no `pm_name` at all, which is the same outcome.
        return Registration::NotAHandler;
    };

    let missing: Vec<String> = REQUIRED
        .iter()
        .filter(|name| !evaluated.has_hook(name))
        .map(|name| (*name).to_owned())
        .collect();
    if !missing.is_empty() {
        return Registration::Incomplete {
            manager: manager.clone(),
            missing,
        };
    }

    Registration::Handler(Handler {
        manager: manager.clone(),
        component: component.to_owned(),
        has_update: evaluated.has_hook(UPDATE_PKG),
        has_add_repo: evaluated.has_hook(ADD_REPO),
    })
}

/// One call to make on a handler.
///
/// Data rather than an invocation, so the engine can make it with the `ctx` of
/// the component that declared the package; see [R-PM-014].
#[derive(Debug, Clone, PartialEq)]
pub struct Call {
    /// The component whose file exports the function.
    pub component: String,
    /// The manager, for a message.
    pub manager: String,
    /// The function to call.
    pub function: &'static str,
    /// The positional arguments after `ctx`.
    pub positional: Vec<String>,
    /// The keyword arguments the declaration carried.
    pub keyword: BTreeMap<String, Argument>,
}

/// Which component handles which manager.
#[derive(Debug, Clone, Default)]
pub struct Registry {
    handlers: BTreeMap<String, Handler>,
}

impl Registry {
    /// An empty registry.
    #[must_use]
    pub fn new() -> Self {
        Registry::default()
    }

    /// Records a handler.
    ///
    /// # Errors
    ///
    /// [`PmError::DuplicateHandler`] when another component already handles
    /// the manager. `v0.1.0` overwrites instead, which makes the winner depend
    /// on the order components happened to evaluate in; see [R-PM-004].
    pub fn register(&mut self, handler: Handler) -> PmResult<()> {
        if let Some(existing) = self.handlers.get(&handler.manager) {
            // Registering the same component twice is not a conflict: an
            // evaluation that ran again found the same thing.
            if existing.component != handler.component {
                return Err(PmError::DuplicateHandler {
                    manager: handler.manager,
                    first: existing.component.clone(),
                    second: handler.component,
                });
            }
            return Ok(());
        }
        self.handlers.insert(handler.manager.clone(), handler);
        Ok(())
    }

    /// The handler for a manager.
    ///
    /// # Errors
    ///
    /// [`PmError::NoHandler`], listing what is registered; see [R-PM-030].
    pub fn handler(&self, manager: &str) -> PmResult<&Handler> {
        self.handlers
            .get(manager)
            .ok_or_else(|| PmError::NoHandler {
                manager: manager.to_owned(),
                registered: self.managers(),
            })
    }

    /// Every registered manager, sorted.
    #[must_use]
    pub fn managers(&self) -> Vec<String> {
        self.handlers.keys().cloned().collect()
    }

    /// Whether anything handles this manager.
    #[must_use]
    pub fn has(&self, manager: &str) -> bool {
        self.handlers.contains_key(manager)
    }

    /// What to call for a package declaration.
    ///
    /// # Errors
    ///
    /// [`PmError::NoHandler`] when nothing handles the manager.
    pub fn call_for(&self, declaration: &PackageDecl) -> PmResult<Call> {
        let handler = self.handler(&declaration.manager)?;
        let (function, positional) = match declaration.action {
            PackageAction::Install => (
                "install_pkg",
                vec![declaration.name.clone(), declaration.version.clone()],
            ),
            PackageAction::Uninstall => (
                "uninstall_pkg",
                vec![declaration.name.clone(), declaration.version.clone()],
            ),
            // A handler with no `update_pkg` installs `latest` instead.
            // Removing the fallback would break every handler that never
            // defined an update path; see [R-PM-012].
            PackageAction::Update if handler.has_update => {
                ("update_pkg", vec![declaration.name.clone()])
            }
            PackageAction::Update => (
                "install_pkg",
                vec![declaration.name.clone(), "latest".to_owned()],
            ),
        };
        Ok(Call {
            component: handler.component.clone(),
            manager: handler.manager.clone(),
            function,
            positional,
            keyword: declaration.extra.clone(),
        })
    }

    /// What to call for a repository declaration.
    ///
    /// # Errors
    ///
    /// [`PmError::NoHandler`] when nothing handles the manager, and
    /// [`PmError::NoAddRepo`] when the handler exports none; see [R-PM-013].
    pub fn call_for_repo(&self, declaration: &RepoDecl) -> PmResult<Call> {
        let handler = self.handler(&declaration.manager)?;
        if !handler.has_add_repo {
            return Err(PmError::NoAddRepo {
                manager: handler.manager.clone(),
                component: handler.component.clone(),
            });
        }
        Ok(Call {
            component: handler.component.clone(),
            manager: handler.manager.clone(),
            function: ADD_REPO,
            positional: Vec::new(),
            keyword: declaration.arguments.clone(),
        })
    }

    /// What to call to interrogate a manager.
    ///
    /// # Errors
    ///
    /// [`PmError::NoHandler`] when nothing handles it.
    pub fn call_for_interrogate(&self, manager: &str) -> PmResult<Call> {
        let handler = self.handler(manager)?;
        Ok(Call {
            component: handler.component.clone(),
            manager: handler.manager.clone(),
            function: "interrogate",
            positional: Vec::new(),
            keyword: BTreeMap::new(),
        })
    }
}
