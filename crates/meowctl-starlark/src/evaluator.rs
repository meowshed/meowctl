//! Evaluating a file, and calling a hook in it.

use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use meowctl_common::Span;
use starlark::environment::{FrozenModule, Globals, GlobalsBuilder, Module};
use starlark::eval::{Evaluator as StarlarkEvaluator, FileLoader};
use starlark::syntax::{AstModule, Dialect};
use starlark::values::{Heap, Value};

use crate::accumulator::{Accumulator, Declarations};
use crate::builtins::{Context, meowctl_globals};
use crate::platform::Platform;
use crate::{StarlarkError, StarlarkResult};

/// Where `load()` gets its modules.
///
/// The implementation lives in `meowctl-module`, which knows about the
/// registry, GitHub, and integrity. This crate only needs the source text and
/// a name to report in a diagnostic; see [R-STAR-020].
pub trait Loader {
    /// Resolves a module URL to the source of the file it names.
    ///
    /// # Errors
    ///
    /// When the module cannot be found, fetched, or verified.
    fn load(&self, module: &str) -> StarlarkResult<LoadedFile>;
}

/// A file a loader produced.
#[derive(Debug, Clone)]
pub struct LoadedFile {
    /// What to call it in a diagnostic.
    pub name: String,
    /// Its source.
    pub source: String,
}

/// Where `query_pm` gets its answer.
///
/// Interrogating a package manager means calling a function in another
/// component's file, which is an evaluation this crate cannot start from
/// inside a builtin of the evaluation already running. So it arrives as a
/// trait, the way [`Loader`] does, and the engine supplies one; see
/// [R-STAR-005] and [R-PM-020].
pub trait PackageManagers: std::fmt::Debug {
    /// What the manager reports is installed.
    ///
    /// # Errors
    ///
    /// When nothing handles the manager, or when its `interrogate` fails.
    fn interrogate(&self, manager: &str) -> StarlarkResult<Vec<String>>;
}

/// A set of package managers with nothing in it.
///
/// For an evaluation that should not be interrogating anything, and for a
/// test that wants to prove one does not.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoPackageManagers;

impl PackageManagers for NoPackageManagers {
    fn interrogate(&self, manager: &str) -> StarlarkResult<Vec<String>> {
        Err(StarlarkError::Evaluation {
            message: format!(
                "query_pm({manager:?}) needs a package-manager registry and this evaluation has none"
            ),
            span: None,
            load_chain: Vec::new(),
        })
    }
}

/// A loader that refuses everything.
///
/// For a file that should not be loading anything, and for a test that wants
/// to prove one does not.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoLoader;

impl Loader for NoLoader {
    fn load(&self, module: &str) -> StarlarkResult<LoadedFile> {
        Err(StarlarkError::Load {
            module: module.to_owned(),
            reason: "this evaluation has no loader".to_owned(),
        })
    }
}

/// Builds the argument a hook is called with.
///
/// A hook takes `ctx`, which is built by `meowctl-ctx`. That crate depends on
/// this one, so the value cannot be named here; it arrives as something that
/// knows how to allocate itself on the evaluation's heap. The heap is scoped
/// to the evaluation, which is why this is a callback rather than a value.
pub trait HookArgument {
    /// Allocates the argument.
    fn allocate<'v>(&self, heap: Heap<'v>) -> Value<'v>;
}

/// Evaluates configuration files.
pub struct Evaluator<'a> {
    platform: Platform,
    loader: &'a dyn Loader,
    /// Modules already evaluated in this command, by URL.
    ///
    /// A graph that loads one helper from twenty components pays for it once;
    /// see [R-STAR-023].
    cache: RefCell<HashMap<String, FrozenModule>>,
    /// Where `query_pm` sends its question; see [R-STAR-005].
    package_managers: Arc<dyn PackageManagers>,
}

impl std::fmt::Debug for Evaluator<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Evaluator")
            .field("platform", &self.platform)
            .finish_non_exhaustive()
    }
}

/// What one file declared, and what it exported.
#[derive(Debug, Clone, PartialEq)]
pub struct Evaluated {
    /// Everything its builtins collected.
    pub declarations: Declarations,
    /// Its top-level names, sorted.
    pub globals: Vec<String>,
    /// Which of those are functions, so a hook can be found without calling
    /// it; see [R-STAR-032].
    pub callables: Vec<String>,
    /// Those bound to a string, with their values; see [R-STAR-033].
    pub strings: BTreeMap<String, String>,
}

impl Evaluated {
    /// Whether the file exports a callable hook of this name.
    #[must_use]
    pub fn has_hook(&self, name: &str) -> bool {
        self.callables.iter().any(|c| c == name)
    }
}

impl<'a> Evaluator<'a> {
    /// An evaluator for this machine, loading through this loader.
    #[must_use]
    pub fn new(platform: Platform, loader: &'a dyn Loader) -> Self {
        Evaluator {
            platform,
            loader,
            cache: RefCell::new(HashMap::new()),
            package_managers: Arc::new(NoPackageManagers),
        }
    }

    /// Sends `query_pm` to this registry.
    ///
    /// Without it `query_pm` refuses, which is right for an evaluation that
    /// has no components to ask.
    #[must_use]
    pub fn with_package_managers(mut self, managers: Arc<dyn PackageManagers>) -> Self {
        self.package_managers = managers;
        self
    }

    /// Evaluates a file and reports what it declared.
    ///
    /// # Errors
    ///
    /// [`StarlarkError::Evaluation`] when it does not parse or fails while
    /// running, or [`StarlarkError::Load`] when a `load()` cannot be resolved.
    pub fn evaluate(&self, name: &str, source: &str) -> StarlarkResult<Evaluated> {
        self.run(name, source, None::<&dyn HookArgument>, None)
            .map(|(evaluated, _)| evaluated)
    }

    /// Evaluates a file and calls one of its functions with `argument`.
    ///
    /// A hook that is absent is a success with nothing to do, because most
    /// components define three of the thirteen phases; see [R-STAR-031].
    ///
    /// # Errors
    ///
    /// Whatever [`Evaluator::evaluate`] fails with, plus
    /// [`StarlarkError::NotCallable`] when the name is taken by something that
    /// is not a function, and [`StarlarkError::Evaluation`] when the hook
    /// itself fails.
    pub fn call_hook(
        &self,
        name: &str,
        source: &str,
        hook: &str,
        argument: &dyn HookArgument,
    ) -> StarlarkResult<bool> {
        let (_, called) = self.run(name, source, Some(argument), Some(hook))?;
        Ok(called)
    }

    /// Evaluates, and optionally calls a hook, inside one heap.
    ///
    /// One function because a module's heap is scoped to a closure: the hook
    /// has to be called while the module that defines it is still alive.
    fn run(
        &self,
        name: &str,
        source: &str,
        argument: Option<&dyn HookArgument>,
        hook: Option<&str>,
    ) -> StarlarkResult<(Evaluated, bool)> {
        let globals = build_globals();
        let context = Context {
            accumulator: Accumulator::new(),
            platform: self.platform.clone(),
            package_managers: Arc::clone(&self.package_managers),
        };
        let loader = CachingLoader {
            inner: self.loader,
            cache: &self.cache,
            globals: &globals,
            platform: &self.platform,
            package_managers: &self.package_managers,
            chain: RefCell::new(Vec::new()),
        };

        let ast = parse(name, source)?;

        Module::with_temp_heap(|module| {
            {
                let mut eval = StarlarkEvaluator::new(&module);
                eval.set_loader(&loader);
                eval.extra = Some(&context);
                eval.eval_module(ast, &globals)
                    .map_err(|e| evaluation_error(&e, &loader.chain.borrow()))?;
            }

            let mut globals_found: Vec<String> =
                module.names().map(|n| n.as_str().to_owned()).collect();
            globals_found.sort();

            let callables: Vec<String> = globals_found
                .iter()
                .filter(|n| module.get(n).is_some_and(is_callable))
                .cloned()
                .collect();

            // [R-STAR-033]: the value of every top-level string, because
            // `pm_name` is what decides whether a component handles a package
            // manager, and which one.
            let strings: BTreeMap<String, String> = globals_found
                .iter()
                .filter_map(|name| {
                    let value = module.get(name)?;
                    let text = value.unpack_str()?;
                    Some((name.clone(), text.to_owned()))
                })
                .collect();

            let evaluated = Evaluated {
                declarations: context.accumulator.declarations(),
                globals: globals_found,
                callables,
                strings,
            };

            let Some(hook) = hook else {
                return Ok((evaluated, false));
            };
            let Some(function) = module.get(hook) else {
                // Absent is a success with nothing to do.
                return Ok((evaluated, false));
            };
            if !evaluated.has_hook(hook) {
                return Err(StarlarkError::NotCallable {
                    component: name.to_owned(),
                    hook: hook.to_owned(),
                    found: function.get_type().to_owned(),
                });
            }

            let arg = argument.map(|a| a.allocate(module.heap()));
            let mut eval = StarlarkEvaluator::new(&module);
            eval.set_loader(&loader);
            eval.extra = Some(&context);
            eval.eval_function(function, arg.as_slice(), &[])
                .map_err(|e| evaluation_error(&e, &loader.chain.borrow()))?;

            Ok((
                Evaluated {
                    declarations: context.accumulator.declarations(),
                    ..evaluated
                },
                true,
            ))
        })
    }
}

/// Whether a global can be called as a hook.
///
/// By type name rather than by a trait probe, because the answer has to match
/// what a user is told when it is wrong: the message names the type, and a
/// check that disagreed with the message would be worse than no check.
fn is_callable(value: Value<'_>) -> bool {
    matches!(value.get_type(), "function" | "builtin_function_or_method")
}

/// The globals every evaluation gets.
fn build_globals() -> Globals {
    // `standard()` rather than `extended()`: the extended set adds builtins
    // `v0.1.0` does not provide, and a component using one would not run on
    // the Go binary; see [R-STAR-001].
    GlobalsBuilder::standard().with(meowctl_globals).build()
}

/// The dialect every configuration is parsed with.
///
/// Named here and again in `meowctl-config`'s editor, which parses the same
/// files to change one declaration without disturbing the rest. A file one
/// accepts and the other rejects would mean `meowctl add` succeeding on a
/// configuration that then fails to apply, and a test in that crate runs one
/// file through both to keep the two honest.
fn dialect() -> Dialect {
    Dialect::Extended
}

/// Parses a file, turning a syntax error into a diagnostic with its position.
fn parse(name: &str, source: &str) -> StarlarkResult<AstModule> {
    AstModule::parse(name, source.to_owned(), &dialect()).map_err(|e| evaluation_error(&e, &[]))
}

/// Turns a `starlark` failure into one of ours, keeping the span.
fn evaluation_error(error: &starlark::Error, chain: &[String]) -> StarlarkError {
    let span = error.span().map(|s| Span {
        file: s.file.filename().to_owned(),
        line: s.resolve_span().begin.line as u32 + 1,
        column: s.resolve_span().begin.column as u32 + 1,
        source_line: s.file.source_line_at_pos(s.span.begin()).to_owned().into(),
    });

    StarlarkError::Evaluation {
        message: error.to_string(),
        span,
        load_chain: chain.to_vec(),
    }
}

/// Resolves `load()` through the caller's loader, evaluating and caching what
/// it produces.
struct CachingLoader<'a> {
    inner: &'a dyn Loader,
    cache: &'a RefCell<HashMap<String, FrozenModule>>,
    globals: &'a Globals,
    platform: &'a Platform,
    /// Where a `query_pm` inside a loaded file sends its question.
    package_managers: &'a Arc<dyn PackageManagers>,
    /// The files being loaded, outermost first, for a diagnostic.
    chain: RefCell<Vec<String>>,
}

impl FileLoader for CachingLoader<'_> {
    fn load(&self, path: &str) -> starlark::Result<FrozenModule> {
        if let Some(cached) = self.cache.borrow().get(path) {
            return Ok(cached.clone());
        }

        let file = self
            .inner
            .load(path)
            .map_err(|e| starlark::Error::new_other(anyhow::anyhow!("{e}")))?;

        self.chain.borrow_mut().push(file.name.clone());

        let ast = AstModule::parse(&file.name, file.source, &dialect())?;
        let context = Context {
            accumulator: Accumulator::new(),
            platform: self.platform.clone(),
            package_managers: Arc::clone(self.package_managers),
        };

        let frozen = Module::with_temp_heap(|module| {
            {
                let mut eval = StarlarkEvaluator::new(&module);
                eval.set_loader(self);
                eval.extra = Some(&context);
                eval.eval_module(ast, self.globals)?;
            }
            starlark::Result::Ok(module.freeze()?)
        })?;

        self.chain.borrow_mut().pop();
        self.cache
            .borrow_mut()
            .insert(path.to_owned(), frozen.clone());
        Ok(frozen)
    }
}
