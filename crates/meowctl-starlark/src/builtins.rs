//! The predeclared set.
//!
//! Exactly what `makePredeclared` provides in
//! `internal/starlark/builtins.go`, under the same names and with the same
//! signatures. Adding one is surface `v0.1.0` cannot evaluate, so a component
//! using it would stop working on the old binary; see [R-STAR-001].

use std::collections::BTreeMap;

use starlark::collections::SmallMap;
use starlark::environment::GlobalsBuilder;
use starlark::eval::Evaluator;
use starlark::starlark_module;
use starlark::values::Value;
use starlark::values::dict::DictRef;
use starlark::values::none::NoneType;

use crate::accumulator::{
    Accumulator, Argument, ComponentDecl, DepDecl, ModuleDecl, PackageAction, PackageDecl,
    ReplaceDecl, RepoDecl,
};
use crate::json::JsonModule;
use crate::platform::{Platform, PlatformValue};

/// What a builtin needs from the evaluation it is running inside.
///
/// Reached through `Evaluator::extra`, which M0 established as the equivalent
/// of the thread-local `v0.1.0` uses, and which is per evaluation rather than
/// global; see [R-STAR-010].
#[derive(Debug, starlark::values::ProvidesStaticType)]
pub(crate) struct Context {
    /// Where declarations go.
    pub(crate) accumulator: Accumulator,
    /// The machine, for `platform()` and `select()`.
    pub(crate) platform: Platform,
}

/// The context of the evaluation a builtin is running in.
fn context<'a>(eval: &'a Evaluator<'_, '_, '_>) -> anyhow::Result<&'a Context> {
    eval.extra
        .ok_or_else(|| anyhow::anyhow!("this builtin needs an evaluation context and has none"))?
        .downcast_ref::<Context>()
        .ok_or_else(|| anyhow::anyhow!("the evaluation context is not a meowctl one"))
}

/// Flattens a keyword argument into something that outlives the evaluation.
///
/// A Starlark value cannot leave the heap it was allocated on, so anything
/// kept has to be copied out here; see [R-STAR-011].
fn flatten(value: Value<'_>) -> Option<Argument> {
    if let Some(s) = value.unpack_str() {
        return Some(Argument::String(s.to_owned()));
    }
    if let Some(b) = value.unpack_bool() {
        return Some(Argument::Boolean(b));
    }
    if let Some(n) = value.unpack_i32() {
        return Some(Argument::Integer(i64::from(n)));
    }
    if let Some(list) = starlark::values::list::ListRef::from_value(value) {
        let items: Option<Vec<String>> = list
            .iter()
            .map(|v| v.unpack_str().map(str::to_owned))
            .collect();
        return items.map(Argument::List);
    }
    None
}

/// Flattens keyword arguments into owned data.
fn flatten_kwargs(kwargs: &SmallMap<String, Value<'_>>) -> BTreeMap<String, Argument> {
    kwargs
        .iter()
        .filter_map(|(key, value)| flatten(*value).map(|v| (key.clone(), v)))
        .collect()
}

/// A list of strings, for `after`.
fn string_list(value: Option<Value<'_>>) -> Vec<String> {
    let Some(list) = value.and_then(starlark::values::list::ListRef::from_value) else {
        return Vec::new();
    };
    list.iter()
        .filter_map(|v| v.unpack_str().map(str::to_owned))
        .collect()
}

#[starlark_module]
pub(crate) fn meowctl_globals(builder: &mut GlobalsBuilder) {
    /// Reading and writing JSON.
    ///
    /// A module rather than a function, because that is what
    /// `go.starlark.net/lib/json` gives and what the standard library's
    /// components call: `json.decode(result.stdout)`.
    const json: JsonModule = JsonModule;

    /// Declares a component.
    ///
    /// `name` is positional or named, and `after` is named only, which is what
    /// `builtinComponent` accepts.
    fn component<'v>(
        name: String,
        after: Option<Value<'v>>,
        #[starlark(kwargs)] kwargs: SmallMap<String, Value<'v>>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> anyhow::Result<NoneType> {
        context(eval)?.accumulator.push_component(ComponentDecl {
            name,
            after: string_list(after),
            extra: flatten_kwargs(&kwargs),
        });
        Ok(NoneType)
    }

    /// Declares a package to install.
    ///
    /// The positional order is `manager` then `name`, and both are required.
    /// That is `parsePkgArgs`'s order, and it is not the order a reader
    /// expects: `pkg("brew", "git")` installs git with Homebrew. Components in
    /// the standard library write it both ways round by keyword, so the order
    /// only shows when they do not.
    fn pkg<'v>(
        manager: String,
        name: String,
        version: Option<String>,
        #[starlark(kwargs)] kwargs: SmallMap<String, Value<'v>>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> anyhow::Result<NoneType> {
        context(eval)?.accumulator.push_package(PackageDecl {
            action: PackageAction::Install,
            name,
            version: version.unwrap_or_default(),
            manager,
            extra: flatten_kwargs(&kwargs),
        });
        Ok(NoneType)
    }

    /// Declares a package to remove.
    fn unpkg<'v>(
        manager: String,
        name: String,
        version: Option<String>,
        #[starlark(kwargs)] kwargs: SmallMap<String, Value<'v>>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> anyhow::Result<NoneType> {
        context(eval)?.accumulator.push_package(PackageDecl {
            action: PackageAction::Uninstall,
            name,
            version: version.unwrap_or_default(),
            manager,
            extra: flatten_kwargs(&kwargs),
        });
        Ok(NoneType)
    }

    /// Declares a package to update.
    fn uppkg<'v>(
        manager: String,
        name: String,
        version: Option<String>,
        #[starlark(kwargs)] kwargs: SmallMap<String, Value<'v>>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> anyhow::Result<NoneType> {
        context(eval)?.accumulator.push_package(PackageDecl {
            action: PackageAction::Update,
            name,
            version: version.unwrap_or_default(),
            manager,
            extra: flatten_kwargs(&kwargs),
        });
        Ok(NoneType)
    }

    /// Declares a repository for a package manager.
    fn repo<'v>(
        manager: Option<String>,
        #[starlark(kwargs)] kwargs: SmallMap<String, Value<'v>>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> anyhow::Result<NoneType> {
        context(eval)?.accumulator.push_repo(RepoDecl {
            manager: manager.unwrap_or_default(),
            arguments: flatten_kwargs(&kwargs),
        });
        Ok(NoneType)
    }

    /// Declares a module dependency.
    fn dep<'v>(
        name: String,
        version: Option<String>,
        source: Option<String>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> anyhow::Result<NoneType> {
        context(eval)?.accumulator.push_dep(DepDecl {
            name,
            version: version.unwrap_or_default(),
            source: source.unwrap_or_default(),
        });
        Ok(NoneType)
    }

    /// Declares this configuration's own module identity.
    fn module<'v>(
        name: String,
        version: Option<String>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> anyhow::Result<NoneType> {
        context(eval)?.accumulator.set_module(ModuleDecl {
            name,
            version: version.unwrap_or_default(),
        });
        Ok(NoneType)
    }

    /// Points a module somewhere else.
    fn replace<'v>(
        name: String,
        path: Option<String>,
        source: Option<String>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> anyhow::Result<NoneType> {
        context(eval)?.accumulator.push_replace(ReplaceDecl {
            name,
            path: path.unwrap_or_default(),
            source: source.unwrap_or_default(),
        });
        Ok(NoneType)
    }

    /// Describes the machine this is running on.
    fn platform<'v>(eval: &mut Evaluator<'v, '_, '_>) -> anyhow::Result<Value<'v>> {
        let machine = context(eval)?.platform.clone();
        Ok(eval.heap().alloc_simple(PlatformValue(machine)))
    }

    /// Chooses a value by platform.
    ///
    /// The keys are `//platform:*` conditions and `//conditions:default`. No
    /// condition matching and no default present is an error rather than
    /// `None`, because a configuration that meant to cover this machine and
    /// did not should say so.
    fn select<'v>(
        #[starlark(require = pos)] cases: Value<'v>,
        eval: &mut Evaluator<'v, '_, '_>,
    ) -> anyhow::Result<Value<'v>> {
        let platform = context(eval)?.platform.clone();
        let dict = DictRef::from_value(cases)
            .ok_or_else(|| anyhow::anyhow!("select: cases must be a dict"))?;

        let mut fallback = None;
        for (key, value) in dict.iter() {
            let condition = key.unpack_str().ok_or_else(|| {
                anyhow::anyhow!(
                    "select: condition key must be a string, got {}",
                    key.get_type()
                )
            })?;
            if condition == "//conditions:default" {
                fallback = Some(value);
                continue;
            }
            if platform.matches(condition) {
                return Ok(value);
            }
        }

        fallback.ok_or_else(|| {
            anyhow::anyhow!("select: no condition matched and no //conditions:default provided")
        })
    }
}
