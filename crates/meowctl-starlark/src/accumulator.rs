//! What a configuration declared.
//!
//! Collected per evaluation rather than in a global, so two evaluations in one
//! process cannot see each other's declarations; see [R-STAR-010]. M0 made the
//! other half a fact the compiler enforces: a module's heap is scoped to a
//! closure, so nothing Starlark allocated escapes and every declaration here
//! is owned data; see [R-STAR-011].

use std::cell::RefCell;
use std::collections::BTreeMap;

use allocative::Allocative;
use starlark::values::ProvidesStaticType;

/// A value a declaration carried, reduced to what the engine needs.
///
/// Starlark values cannot leave their evaluation, so a keyword argument is
/// flattened here rather than kept. The four shapes are what a configuration
/// actually passes.
#[derive(Debug, Clone, PartialEq)]
pub enum Argument {
    /// A string.
    String(String),
    /// An integer.
    Integer(i64),
    /// A boolean.
    Boolean(bool),
    /// A list of strings.
    List(Vec<String>),
}

/// A `component()` declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct ComponentDecl {
    /// What it names, in one of the three forms; see [R-COMMON-001].
    pub name: String,
    /// Components this one must run after, without depending on them.
    pub after: Vec<String>,
    /// Anything else the declaration carried.
    pub extra: BTreeMap<String, Argument>,
}

/// A `pkg()`, `unpkg()`, or `uppkg()` declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct PackageDecl {
    /// What to do with it.
    pub action: PackageAction,
    /// The package.
    pub name: String,
    /// The version constraint, when one was given.
    pub version: String,
    /// The manager, when the declaration named one.
    pub manager: String,
    /// Anything else the declaration carried.
    pub extra: BTreeMap<String, Argument>,
}

/// What a package declaration asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageAction {
    /// `pkg()`.
    Install,
    /// `unpkg()`.
    Uninstall,
    /// `uppkg()`.
    Update,
}

/// A `repo()` declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct RepoDecl {
    /// The manager it targets.
    pub manager: String,
    /// Everything else the declaration carried.
    pub arguments: BTreeMap<String, Argument>,
}

/// A `dep()` declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DepDecl {
    /// The module.
    pub name: String,
    /// Its version, for a registry module.
    pub version: String,
    /// Its source, for a GitHub module.
    pub source: String,
}

/// A `replace()` declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplaceDecl {
    /// The module being replaced.
    pub name: String,
    /// A local directory to serve it from.
    pub path: String,
    /// A different remote source to fetch it from.
    pub source: String,
}

/// A `module()` declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleDecl {
    /// The module's own name.
    pub name: String,
    /// Its version.
    pub version: String,
    /// The manifest schema the module was written against.
    ///
    /// Every `MODULE.meow` in the meowshed organisation carries one, and a
    /// reader that rejected it would reject every module there is. Absent in
    /// a `deps.mod`, which meowctl writes itself, and recorded rather than
    /// acted on: nothing reads it yet, and dropping it would mean a module
    /// declaring a newer schema is indistinguishable from one that does not.
    pub compat: Option<i64>,
}

/// Everything one evaluation declared, in declaration order.
///
/// Order is preserved because it is the tie-break the component graph uses
/// between components with no dependency between them; see [R-STAR-012].
#[derive(Debug, Default, ProvidesStaticType, Allocative)]
pub struct Accumulator {
    #[allocative(skip)]
    inner: RefCell<Declarations>,
}

/// The collected declarations.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Declarations {
    /// `component()` declarations.
    pub components: Vec<ComponentDecl>,
    /// `pkg()`, `unpkg()` and `uppkg()` declarations.
    pub packages: Vec<PackageDecl>,
    /// `repo()` declarations.
    pub repos: Vec<RepoDecl>,
    /// `dep()` declarations.
    pub deps: Vec<DepDecl>,
    /// `replace()` declarations.
    pub replaces: Vec<ReplaceDecl>,
    /// The `module()` declaration, when there was one.
    pub module: Option<ModuleDecl>,
}

impl Accumulator {
    /// An empty accumulator.
    #[must_use]
    pub fn new() -> Self {
        Accumulator::default()
    }

    /// Everything declared so far.
    ///
    /// # Panics
    ///
    /// If a builtin panicked while holding the borrow.
    #[must_use]
    pub fn declarations(&self) -> Declarations {
        self.inner.borrow().clone()
    }

    pub(crate) fn push_component(&self, decl: ComponentDecl) {
        self.inner.borrow_mut().components.push(decl);
    }

    pub(crate) fn push_package(&self, decl: PackageDecl) {
        self.inner.borrow_mut().packages.push(decl);
    }

    pub(crate) fn push_repo(&self, decl: RepoDecl) {
        self.inner.borrow_mut().repos.push(decl);
    }

    pub(crate) fn push_dep(&self, decl: DepDecl) {
        self.inner.borrow_mut().deps.push(decl);
    }

    pub(crate) fn push_replace(&self, decl: ReplaceDecl) {
        self.inner.borrow_mut().replaces.push(decl);
    }

    pub(crate) fn set_module(&self, decl: ModuleDecl) {
        self.inner.borrow_mut().module = Some(decl);
    }
}
