//! The Starlark value, its attributes, and what each one does.
//!
//! Three types rather than one, because the restriction is the value the hook
//! receives rather than a check inside each method: a hook in a read-only
//! phase gets a `ctx` with no mutating attribute at all, and asking for one
//! reports attribute-not-found; see [R-CTX-030], [R-CTX-031] and [R-CTX-032].

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use allocative::Allocative;
use meowctl_common::{Event, Integrity, Level, paths};
use meowctl_exec::Command;
use meowctl_ops::Op;
use meowctl_starlark::{HookArgument, PlatformValue};
use starlark::collections::SmallMap;
use starlark::environment::{Methods, MethodsBuilder, MethodsStatic};
use starlark::starlark_module;
use starlark::values::{
    Heap, NoSerialize, ProvidesStaticType, StarlarkValue, Value, ValueLike as _, starlark_value,
};

use crate::state::Core;
use crate::{Capabilities, CtxError, CtxResult, Effects, Surface};

/// The `ctx` a hook is called with.
#[derive(Debug, Clone, ProvidesStaticType, NoSerialize, Allocative)]
pub struct Ctx(#[allocative(skip)] Arc<Core>);

/// The same `ctx` with no mutating attribute; see [R-CTX-030].
#[derive(Debug, Clone, ProvidesStaticType, NoSerialize, Allocative)]
struct ReadOnlyCtx(#[allocative(skip)] Arc<Core>);

/// The eight attributes `shell.star` gets; see [R-CTX-031].
#[derive(Debug, Clone, ProvidesStaticType, NoSerialize, Allocative)]
struct ShellCtx(#[allocative(skip)] Arc<Core>);

starlark::starlark_simple_value!(Ctx);
starlark::starlark_simple_value!(ReadOnlyCtx);
starlark::starlark_simple_value!(ShellCtx);

impl Ctx {
    /// A `ctx` over these capabilities and effects.
    #[must_use]
    pub fn new(capabilities: Capabilities, effects: Effects) -> Self {
        Ctx(Arc::new(Core {
            capabilities,
            effects,
        }))
    }

    /// What it knows.
    #[must_use]
    pub fn capabilities(&self) -> &Capabilities {
        &self.0.capabilities
    }
}

/// Allocates the surface a hook should see.
///
/// Which of the three types is allocated is the whole of the restriction; see
/// [R-CTX-032].
#[derive(Debug, Clone)]
pub struct Restricted {
    ctx: Ctx,
    surface: Surface,
}

impl Restricted {
    /// The surface a hook in this phase gets.
    #[must_use]
    pub fn new(ctx: Ctx, surface: Surface) -> Self {
        Restricted { ctx, surface }
    }
}

impl HookArgument for Restricted {
    fn allocate<'v>(&self, heap: Heap<'v>) -> Value<'v> {
        match self.surface {
            Surface::Full => heap.alloc_simple(self.ctx.clone()),
            Surface::ReadOnly => heap.alloc_simple(ReadOnlyCtx(Arc::clone(&self.ctx.0))),
            Surface::Shell => heap.alloc_simple(ShellCtx(Arc::clone(&self.ctx.0))),
        }
    }
}

impl HookArgument for Ctx {
    fn allocate<'v>(&self, heap: Heap<'v>) -> Value<'v> {
        heap.alloc_simple(self.clone())
    }
}

macro_rules! display_as_ctx {
    ($($t:ty),*) => {$(
        impl std::fmt::Display for $t {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("<ctx>")
            }
        }
    )*};
}
display_as_ctx!(Ctx, ReadOnlyCtx, ShellCtx);

/// The six data properties, per [R-CTX-001].
fn property<'v>(core: &Core, attr: &str, heap: Heap<'v>) -> Option<Value<'v>> {
    let caps = &core.capabilities;
    match attr {
        "home" => Some(heap.alloc(caps.home.display().to_string())),
        "dry_run" => Some(Value::new_bool(caps.dry_run)),
        "component_dir" => Some(heap.alloc(caps.component_dir.display().to_string())),
        "state_dir" => Some(heap.alloc(caps.state_dir.display().to_string())),
        // `None` outside `shell.star`, which is how a component tests whether
        // it is being asked to contribute to a shell; see [R-CTX-002].
        "shell" => Some(
            caps.shell
                .as_ref()
                .map_or_else(Value::new_none, |s| heap.alloc(s.as_str())),
        ),
        "platform" => Some(heap.alloc_simple(PlatformValue::new(caps.platform.clone()))),
        _ => None,
    }
}

/// The properties the full and read-only surfaces carry.
const PROPERTIES: [&str; 6] = [
    "home",
    "dry_run",
    "component_dir",
    "state_dir",
    "shell",
    "platform",
];

/// The three `shell.star` may read; see [R-CTX-031].
const SHELL_PROPERTIES: [&str; 3] = ["platform", "shell", "state_dir"];

#[starlark_value(type = "ctx")]
impl<'v> StarlarkValue<'v> for Ctx {
    fn get_methods() -> Option<&'static Methods> {
        static RES: MethodsStatic = MethodsStatic::new("ctx", |builder| {
            reading_methods(builder);
            general_methods(builder);
            mutating_methods(builder);
        });
        Some(RES.methods())
    }

    fn get_attr(&self, attr: &str, heap: Heap<'v>) -> Option<Value<'v>> {
        property(&self.0, attr, heap)
    }

    fn has_attr(&self, attr: &str, _heap: Heap<'v>) -> bool {
        PROPERTIES.contains(&attr)
    }

    fn dir_attr(&self) -> Vec<String> {
        PROPERTIES.iter().map(|a| (*a).to_owned()).collect()
    }
}

#[starlark_value(type = "ctx")]
impl<'v> StarlarkValue<'v> for ReadOnlyCtx {
    fn get_methods() -> Option<&'static Methods> {
        static RES: MethodsStatic = MethodsStatic::new("ctx", |builder| {
            reading_methods(builder);
            general_methods(builder);
        });
        Some(RES.methods())
    }

    fn get_attr(&self, attr: &str, heap: Heap<'v>) -> Option<Value<'v>> {
        property(&self.0, attr, heap)
    }

    fn has_attr(&self, attr: &str, _heap: Heap<'v>) -> bool {
        PROPERTIES.contains(&attr)
    }

    fn dir_attr(&self) -> Vec<String> {
        PROPERTIES.iter().map(|a| (*a).to_owned()).collect()
    }
}

#[starlark_value(type = "ctx")]
impl<'v> StarlarkValue<'v> for ShellCtx {
    fn get_methods() -> Option<&'static Methods> {
        static RES: MethodsStatic = MethodsStatic::new("ctx", reading_methods);
        Some(RES.methods())
    }

    fn get_attr(&self, attr: &str, heap: Heap<'v>) -> Option<Value<'v>> {
        if !SHELL_PROPERTIES.contains(&attr) {
            return None;
        }
        property(&self.0, attr, heap)
    }

    fn has_attr(&self, attr: &str, _heap: Heap<'v>) -> bool {
        SHELL_PROPERTIES.contains(&attr)
    }

    fn dir_attr(&self) -> Vec<String> {
        SHELL_PROPERTIES.iter().map(|a| (*a).to_owned()).collect()
    }
}

/// Recovers the `ctx` behind whichever of the three surfaces was allocated.
fn core<'v>(this: Value<'v>) -> anyhow::Result<&'v Core> {
    if let Some(ctx) = this.downcast_ref::<Ctx>() {
        return Ok(&ctx.0);
    }
    if let Some(ctx) = this.downcast_ref::<ReadOnlyCtx>() {
        return Ok(&ctx.0);
    }
    if let Some(ctx) = this.downcast_ref::<ShellCtx>() {
        return Ok(&ctx.0);
    }
    Err(anyhow::anyhow!("this method is only available on ctx"))
}

/// Resolves a path argument.
///
/// Expands a leading `~` and refuses anything that is not absolute
/// afterwards, for every method that takes one; see [R-CTX-012].
fn path(method: &'static str, core: &Core, raw: &str) -> CtxResult<PathBuf> {
    let env = HomeOnly(&core.capabilities.home);
    paths::resolve(raw, &env).map_err(|source| CtxError::Path { method, source })
}

/// An environment that knows only `$HOME`.
///
/// `ctx` resolves against the home directory the run was given rather than
/// against the process environment, so a test can hand it one.
struct HomeOnly<'a>(&'a Path);

impl paths::Env for HomeOnly<'_> {
    fn var(&self, key: &str) -> Option<String> {
        (key == "HOME").then(|| self.0.display().to_string())
    }
}

/// Reads a `vars` argument as the mapping [R-CTX-027] requires.
fn variables(
    method: &'static str,
    vars: &SmallMap<String, Value<'_>>,
) -> CtxResult<BTreeMap<String, String>> {
    vars.iter()
        .map(|(key, value)| {
            value
                .unpack_str()
                .map(|text| (key.clone(), text.to_owned()))
                .ok_or_else(|| CtxError::Argument {
                    method,
                    argument: format!("vars[{key:?}]"),
                    expected: "a string",
                })
        })
        .collect()
}

/// Builds the value `ctx.run` returns.
///
/// Three fields under these names, because component code branches on
/// `exit_code`; see [R-CTX-020].
fn run_result<'v>(heap: Heap<'v>, output: &meowctl_exec::Output) -> Value<'v> {
    let mut fields = SmallMap::new();
    for (name, value) in [
        ("stdout", heap.alloc(output.stdout.as_str())),
        ("stderr", heap.alloc(output.stderr.as_str())),
        (
            "exit_code",
            heap.alloc(i64::from(output.exit_code.unwrap_or(-1))),
        ),
    ] {
        if let Ok(key) = heap.alloc_str(name).to_value().get_hashed() {
            fields.insert_hashed(key, value);
        }
    }
    heap.alloc(starlark::values::dict::Dict::new(fields))
}

/// The methods every surface carries, and the whole of `shell.star`'s.
#[starlark_module]
fn reading_methods(builder: &mut MethodsBuilder) {
    /// Reads a file.
    ///
    /// Fails when it is not there, where `file_exists` answers false. The two
    /// are how a component tests and then reads; see [R-CTX-042].
    fn read_file<'v>(this: Value<'v>, path_arg: String) -> anyhow::Result<String> {
        let core = core(this)?;
        let target = path("read_file", core, &path_arg)?;
        let bytes = core
            .effects
            .fs
            .read(&target)
            .map_err(|e| CtxError::Effect {
                method: "read_file",
                reason: e.to_string(),
            })?;
        String::from_utf8(bytes).map_err(|_| {
            CtxError::Effect {
                method: "read_file",
                reason: format!("{} is not UTF-8", target.display()),
            }
            .into()
        })
    }

    /// Whether anything is at a path.
    fn file_exists<'v>(this: Value<'v>, path_arg: String) -> anyhow::Result<bool> {
        let core = core(this)?;
        let target = path("file_exists", core, &path_arg)?;
        Ok(core.effects.fs.exists(&target).unwrap_or(false))
    }

    /// The names in a directory, sorted.
    fn list_dir<'v>(this: Value<'v>, path_arg: String) -> anyhow::Result<Vec<String>> {
        let core = core(this)?;
        let target = path("list_dir", core, &path_arg)?;
        let entries = core
            .effects
            .fs
            .read_dir(&target)
            .map_err(|e| CtxError::Effect {
                method: "list_dir",
                reason: e.to_string(),
            })?;
        Ok(entries
            .iter()
            .filter_map(|e| e.file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .collect())
    }

    /// Runs a command.
    ///
    /// A non-zero exit is a result rather than an error: an interrogation hook
    /// asking about a package it does not have gets a 1 and wants to read it;
    /// see [R-CTX-043].
    fn run<'v>(
        this: Value<'v>,
        cmd: String,
        args: Option<starlark::values::list::UnpackList<String>>,
        env: Option<SmallMap<String, String>>,
        cwd: Option<String>,
        interactive: Option<bool>,
        heap: Heap<'v>,
    ) -> anyhow::Result<Value<'v>> {
        let core = core(this)?;
        let mut command = Command::new(cmd).args(args.map(|a| a.items).unwrap_or_default());
        for (key, value) in env.unwrap_or_default() {
            command = command.env(key, value);
        }
        if let Some(dir) = cwd {
            command = command.cwd(path("run", core, &dir)?);
        }
        if interactive.unwrap_or(false) {
            command = command.interactive();
        }
        let mut sink = |event| core.effects.emit(event);
        let output = core
            .effects
            .exec
            .run(&command, &mut sink)
            .map_err(|e| CtxError::Effect {
                method: "run",
                reason: e.to_string(),
            })?;
        Ok(run_result(heap, &output))
    }

    /// Writes a line for the calling shell to evaluate.
    ///
    /// Only during `shell` and `login`; anywhere else stdout belongs to the
    /// command and a stray line corrupts a piped run; see [R-CTX-024].
    fn emit<'v>(this: Value<'v>, line: String) -> anyhow::Result<starlark::values::none::NoneType> {
        let core = core(this)?;
        if core.capabilities.phase.is_runtime_hook() {
            core.effects.emit(Event::ShellLine { line });
        }
        Ok(starlark::values::none::NoneType)
    }
}

/// The methods that read or ask but do not mutate.
#[starlark_module]
fn general_methods(builder: &mut MethodsBuilder) {
    /// Says something, at the level a sink renders as a component's own words.
    fn log<'v>(this: Value<'v>, msg: String) -> anyhow::Result<starlark::values::none::NoneType> {
        core(this)?.effects.emit(Event::Message {
            level: Level::Info,
            text: msg,
        });
        Ok(starlark::values::none::NoneType)
    }

    /// An environment variable, or the empty string when it is unset.
    fn env<'v>(this: Value<'v>, key: String) -> anyhow::Result<String> {
        Ok(core(this)?
            .capabilities
            .environment
            .get(&key)
            .cloned()
            .unwrap_or_default())
    }

    /// Where a program is, or `None`; see [R-CTX-021].
    fn which<'v>(this: Value<'v>, name: String, heap: Heap<'v>) -> anyhow::Result<Value<'v>> {
        let found = core(this)?
            .effects
            .exec
            .which(&name)
            .map_err(|e| CtxError::Effect {
                method: "which",
                reason: e.to_string(),
            })?;
        Ok(found.map_or_else(Value::new_none, |p| heap.alloc(p.display().to_string())))
    }

    /// Asks the user something and returns what they typed.
    fn prompt<'v>(this: Value<'v>, question: String) -> anyhow::Result<String> {
        let core = core(this)?;
        let failed = |reason: String| CtxError::Effect {
            method: "prompt",
            reason,
        };
        let mut interaction = core
            .effects
            .interaction
            .lock()
            .map_err(|_| failed("the prompt is poisoned".to_owned()))?;
        interaction
            .ask(&question)
            .map_err(|e| failed(e.to_string()).into())
    }

    /// Substitutes `{{name}}` into a string; see [R-CTX-027].
    fn render<'v>(
        this: Value<'v>,
        template_str: String,
        vars: SmallMap<String, Value<'v>>,
    ) -> anyhow::Result<String> {
        let _ = core(this)?;
        Ok(crate::template::render(
            &template_str,
            &variables("render", &vars)?,
        ))
    }

    /// Renders a file from the component's own directory and returns it.
    ///
    /// It writes nothing. A component that wants the result on disk passes it
    /// to `write_file`; see [R-CTX-027].
    fn render_file<'v>(
        this: Value<'v>,
        src: String,
        vars: SmallMap<String, Value<'v>>,
    ) -> anyhow::Result<String> {
        let core = core(this)?;
        let target = core.capabilities.component_dir.join(&src);
        let bytes = core
            .effects
            .fs
            .read(&target)
            .map_err(|e| CtxError::Effect {
                method: "render_file",
                reason: e.to_string(),
            })?;
        let text = String::from_utf8(bytes).map_err(|_| CtxError::Effect {
            method: "render_file",
            reason: format!("{} is not UTF-8", target.display()),
        })?;
        Ok(crate::template::render(
            &text,
            &variables("render_file", &vars)?,
        ))
    }
}

/// The methods that change something.
#[starlark_module]
fn mutating_methods(builder: &mut MethodsBuilder) {
    /// Replaces a file's contents.
    fn write_file<'v>(
        this: Value<'v>,
        dst: String,
        content: String,
    ) -> anyhow::Result<starlark::values::none::NoneType> {
        let core = core(this)?;
        let target = path("write_file", core, &dst)?;
        core.apply(
            "write_file",
            &Op::WriteFile {
                path: target,
                contents: content.into_bytes(),
            },
        )?;
        Ok(starlark::values::none::NoneType)
    }

    /// Appends a marked block, replacing the caller's own block when it is
    /// already there; see [R-CTX-028].
    fn append_file<'v>(
        this: Value<'v>,
        dst: String,
        content: String,
        marker: Option<String>,
    ) -> anyhow::Result<starlark::values::none::NoneType> {
        let core = core(this)?;
        let target = path("append_file", core, &dst)?;
        let marker = marker.unwrap_or_else(|| format!("meowctl:{}", core.capabilities.component));
        core.apply(
            "append_file",
            &Op::AppendFile {
                path: target,
                contents: content,
                marker,
            },
        )?;
        Ok(starlark::values::none::NoneType)
    }

    /// Removes a file.
    fn delete_file<'v>(
        this: Value<'v>,
        dst: String,
    ) -> anyhow::Result<starlark::values::none::NoneType> {
        let core = core(this)?;
        let target = path("delete_file", core, &dst)?;
        core.apply("delete_file", &Op::Remove { path: target })?;
        Ok(starlark::values::none::NoneType)
    }

    /// Copies a file.
    fn copy_file<'v>(
        this: Value<'v>,
        src: String,
        dst: String,
    ) -> anyhow::Result<starlark::values::none::NoneType> {
        let core = core(this)?;
        core.apply(
            "copy_file",
            &Op::CopyFile {
                from: path("copy_file", core, &src)?,
                to: path("copy_file", core, &dst)?,
            },
        )?;
        Ok(starlark::values::none::NoneType)
    }

    /// Creates a symlink.
    fn symlink<'v>(
        this: Value<'v>,
        src: String,
        dst: String,
    ) -> anyhow::Result<starlark::values::none::NoneType> {
        let core = core(this)?;
        core.apply(
            "symlink",
            &Op::Symlink {
                target: path("symlink", core, &src)?,
                link: path("symlink", core, &dst)?,
            },
        )?;
        Ok(starlark::values::none::NoneType)
    }

    /// Removes a symlink, and only a symlink; see [R-CTX-044].
    fn remove_symlink<'v>(
        this: Value<'v>,
        dst: String,
    ) -> anyhow::Result<starlark::values::none::NoneType> {
        let core = core(this)?;
        let target = path("remove_symlink", core, &dst)?;
        match core.effects.fs.entry(&target) {
            Ok(Some(entry)) if entry.is_symlink() => {}
            Ok(Some(_)) => {
                return Err(CtxError::Effect {
                    method: "remove_symlink",
                    reason: format!("{} is not a symlink", target.display()),
                }
                .into());
            }
            _ => {}
        }
        core.apply("remove_symlink", &Op::Remove { path: target })?;
        Ok(starlark::values::none::NoneType)
    }

    /// Links a file into place, moving anything already there aside.
    fn link_file<'v>(
        this: Value<'v>,
        src: String,
        dst: String,
        backup: Option<String>,
    ) -> anyhow::Result<starlark::values::none::NoneType> {
        let core = core(this)?;
        let link = path("link_file", core, &dst)?;
        let backup = match backup {
            Some(raw) => path("link_file", core, &raw)?,
            None => backup_beside(&link),
        };
        core.apply(
            "link_file",
            &Op::LinkFile {
                target: path("link_file", core, &src)?,
                link,
                backup,
            },
        )?;
        Ok(starlark::values::none::NoneType)
    }

    /// Creates a directory and its parents.
    fn mkdir<'v>(
        this: Value<'v>,
        path_arg: String,
    ) -> anyhow::Result<starlark::values::none::NoneType> {
        let core = core(this)?;
        let target = path("mkdir", core, &path_arg)?;
        core.apply("mkdir", &Op::Mkdir { path: target })?;
        Ok(starlark::values::none::NoneType)
    }

    /// Clones a repository by running `git`; see [R-CTX-022].
    fn git_clone<'v>(
        this: Value<'v>,
        url: String,
        dst: String,
        r#ref: Option<String>,
    ) -> anyhow::Result<starlark::values::none::NoneType> {
        let core = core(this)?;
        let target = path("git_clone", core, &dst)?;
        let mut args = vec!["clone".to_owned()];
        if let Some(reference) = r#ref {
            args.push("--branch".to_owned());
            args.push(reference);
        }
        args.push(url);
        args.push(target.display().to_string());

        let command = Command::new("git").args(args);
        let mut sink = |event| core.effects.emit(event);
        let output = core
            .effects
            .exec
            .run(&command, &mut sink)
            .map_err(|e| CtxError::Effect {
                method: "git_clone",
                reason: e.to_string(),
            })?;
        if !output.succeeded() {
            return Err(CtxError::Effect {
                method: "git_clone",
                reason: format!("git clone exited {:?}: {}", output.exit_code, output.stderr),
            }
            .into());
        }
        Ok(starlark::values::none::NoneType)
    }

    /// Fetches a file over HTTPS and journals the write; see [R-CTX-023].
    fn download<'v>(
        this: Value<'v>,
        url: String,
        dst: String,
        checksum: Option<String>,
    ) -> anyhow::Result<starlark::values::none::NoneType> {
        let core = core(this)?;
        let target = path("download", core, &dst)?;
        let failed = |reason: String| CtxError::Effect {
            method: "download",
            reason,
        };
        let body = core
            .effects
            .http
            .get(&url)
            .map_err(|e| failed(e.to_string()))?;

        // Before anything is written, for the reason [R-MODULE-030] gives:
        // a check after the write has already put the bytes on disk.
        if let Some(expected) = checksum {
            let expected: Integrity = expected
                .parse()
                .map_err(|_| failed(format!("{expected} is not a sha384 SRI hash")))?;
            if !expected.matches(&body) {
                return Err(failed(format!(
                    "{url} hashes to {}, and {expected} was expected",
                    Integrity::compute(&body)
                ))
                .into());
            }
        }

        core.apply(
            "download",
            &Op::Download {
                path: target,
                contents: body,
            },
        )?;
        Ok(starlark::values::none::NoneType)
    }

    /// Sets a macOS default.
    fn defaults_write<'v>(
        this: Value<'v>,
        domain: String,
        key: String,
        r#type: String,
        value: Value<'v>,
    ) -> anyhow::Result<starlark::values::none::NoneType> {
        let core = core(this)?;
        core.apply(
            "defaults_write",
            &Op::DefaultsWrite {
                domain,
                key,
                value_type: r#type,
                value: shell_argument(value),
            },
        )?;
        Ok(starlark::values::none::NoneType)
    }

    /// Sets a key in a property list.
    fn plist_set<'v>(
        this: Value<'v>,
        file: String,
        key: String,
        r#type: String,
        value: Value<'v>,
    ) -> anyhow::Result<starlark::values::none::NoneType> {
        let core = core(this)?;
        // `v0.1.0` passes the type to `PlistBuddy` and the journal records
        // only the value, so the argument is accepted and not carried.
        let _ = r#type;
        let target = path("plist_set", core, &file)?;
        core.apply(
            "plist_set",
            &Op::PlistSet {
                path: target,
                key,
                value: shell_argument(value),
            },
        )?;
        Ok(starlark::values::none::NoneType)
    }

    /// Puts a directory first on the `PATH` every later `run` sees.
    ///
    /// It emits nothing. The name suggests a shell statement and it is not
    /// one; see [R-CTX-025].
    fn add_path<'v>(
        this: Value<'v>,
        dir: String,
    ) -> anyhow::Result<starlark::values::none::NoneType> {
        let core = core(this)?;
        if !dir.is_empty() {
            core.effects.emit(Event::PathPrepended { directory: dir });
        }
        Ok(starlark::values::none::NoneType)
    }
}

/// Where `link_file` moves what was already there.
///
/// `<name>.meowctl-backup` beside the original, which is what
/// `internal/ctx/methods.go` builds when no backup is given.
fn backup_beside(link: &Path) -> PathBuf {
    let mut name = link.file_name().unwrap_or_default().to_os_string();
    name.push(".meowctl-backup");
    link.with_file_name(name)
}

/// Renders a value as a command-line argument.
///
/// A boolean becomes `true` or `false`, because `defaults write` and
/// `PlistBuddy` both reject Starlark's `True`; `starlarkValueToShellArg` is
/// where that is already known.
fn shell_argument(value: Value<'_>) -> String {
    value.unpack_bool().map_or_else(
        || {
            value
                .unpack_str()
                .map_or_else(|| value.to_string(), str::to_owned)
        },
        |b| if b { "true" } else { "false" }.to_owned(),
    )
}
