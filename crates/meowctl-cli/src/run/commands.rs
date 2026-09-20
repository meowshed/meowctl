//! What each command does.
//!
//! Every one of them is the same shape: read what it needs, call into a
//! domain crate, and report. Nothing here decides anything a domain crate
//! could have decided.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use meowctl_common::{ComponentId, Event, Level, Phase, PhaseSet};
use meowctl_config::{HookError, InstalledLock, LockFile, Sentinel};
use meowctl_engine::{
    ComponentSource, Declaration, Discovered, Graph, Inputs, Plan, Progress, Runner, Settings,
    Sources, discover, fingerprints, interrupted_run, stale_components,
};
use meowctl_module::{Cache, ModuleLoader, Roots, Upgrade};
use meowctl_starlark::{Evaluator, Loader, NoLoader};

use super::Session;
use crate::cli::{Cli, Command, DepCommand};
use crate::{CliError, CliResult};

/// Runs the parsed command.
pub(super) fn run(cli: &Cli, session: &mut Session<'_>) -> CliResult<()> {
    match &cli.command {
        Command::Version => {
            session.say(crate::version::STRING);
            Ok(())
        }
        Command::Completions { shell } => {
            super::completions(*shell);
            Ok(())
        }
        Command::Shell { shell } => {
            print_to_stdout(crate::shell::snippet(*shell));
            Ok(())
        }
        Command::Check { dir } => check(session, dir),
        Command::Status { all, .. } => status(session, *all),
        Command::Doctor { .. } => doctor(session),
        Command::Dep {
            command: DepCommand::List,
        } => dep_list(session),
        Command::Dep {
            command: DepCommand::Sync { .. },
        } => dep_sync(session, &Upgrade::Nothing),
        Command::Apply {
            components,
            force,
            no_rollback,
            ..
        } => apply(
            session,
            PhaseSet::Install,
            components,
            *force,
            !*no_rollback,
        ),
        Command::Upgrade {
            components,
            force,
            no_rollback,
            ..
        } => apply(
            session,
            PhaseSet::Upgrade,
            components,
            *force,
            !*no_rollback,
        ),
        // `verify` forces, because a check that skipped what it checked last
        // time is not a check. It never rolls back whatever `--no-rollback`
        // says: its one phase is read-only, so there is nothing journalled to
        // undo; see [R-COMMON-012].
        Command::Verify { components, .. } => {
            apply(session, PhaseSet::Verify, components, true, false)
        }
        Command::Hook { phase } => hook(session, phase),

        Command::Init { repo_url, force } => match repo_url {
            Some(url) => super::writing::bootstrap(session, url, *force),
            None => super::writing::init(session, *force),
        },
        Command::Add {
            components,
            force,
            no_rollback,
            ..
        } => super::writing::add(session, components, *force, !*no_rollback),
        Command::Remove {
            components,
            no_rollback,
            ..
        } => super::writing::remove(session, components, !*no_rollback),
        Command::Dep {
            command:
                DepCommand::Add {
                    name,
                    version,
                    source,
                    local,
                },
        } => super::writing::dep_add(session, name, version.as_deref(), source.as_deref(), *local),
        Command::Dep {
            command: DepCommand::Remove { name, local },
        } => super::writing::dep_remove(session, name, *local),
        Command::Dep {
            command: DepCommand::Upgrade { modules, .. },
        } => {
            let upgrade = if modules.is_empty() {
                Upgrade::Everything
            } else {
                Upgrade::Named(modules.iter().cloned().collect())
            };
            dep_sync(session, &upgrade)
        }
        Command::Dep {
            command: DepCommand::Tidy { .. },
        } => super::writing::dep_tidy(session),
        Command::Update {
            yes, no_rollback, ..
        } => super::writing::update(session, *yes, !*no_rollback),

        // Not absent, and not pretending. `v0.1.0` renames an unverified
        // download over the running binary; reproducing that would ship the
        // weakness deliberately, and there is no published checksum to verify
        // against yet. See [R-CLI-070] and [R-CLI-071].
        Command::SelfUpdate => Err(CliError::General(
            "self-update cannot update this build: no release publishes a checksum to verify a \
             download against. Install the new release the way you installed this one"
                .to_owned(),
        )),
    }
}

impl Session<'_> {
    /// Says something through the sink.
    ///
    /// The only route to standard output there is; see [R-CLI-020].
    pub(super) fn say(&mut self, text: &str) {
        self.sink.handle(&Event::Message {
            level: Level::Info,
            text: text.to_owned(),
        });
    }

    /// Says something the reader should act on.
    pub(super) fn warn(&mut self, text: &str) {
        self.sink.handle(&Event::Message {
            level: Level::Warn,
            text: text.to_owned(),
        });
    }

    /// Whether there is a configuration here at all.
    pub(super) fn require_configured(&self) -> CliResult<()> {
        self.layout
            .check(self.fs.as_ref())
            .map_err(|_| CliError::NotConfigured {
                directory: self.layout.root().display().to_string(),
            })
    }

    /// The lock, overlaid with the machine-local one.
    fn lock(&self) -> CliResult<LockFile> {
        let shared = LockFile::read(self.fs.as_ref(), &self.layout.lock())?;
        let local = LockFile::read(self.fs.as_ref(), &self.layout.local_lock())?;
        Ok(shared.overlaid_with(&local))
    }

    /// A loader over the module cache, with the lock's resolutions.
    fn loader<'a>(&'a self, lock: &LockFile) -> ModuleLoader<'a> {
        let mut loader = ModuleLoader::new(
            self.fs.as_ref(),
            self.http.as_ref(),
            Cache::new(self.cache.clone()),
            Roots {
                dotfiles: self.layout.root().to_path_buf(),
                config: self.layout.root().to_path_buf(),
            },
        );
        for (name, entry) in &lock.modules {
            if entry.replaced {
                loader = loader.replacing(name.clone(), entry.path.clone());
                continue;
            }
            let key = if entry.commit_sha.is_empty() {
                entry.version.clone()
            } else {
                entry.commit_sha.clone()
            };
            if !key.is_empty() {
                loader = loader.resolved(name.clone(), key);
            }
        }
        loader
    }
}

/// Where a component's file comes from: the configuration, or a module.
///
/// It holds the pieces it needs rather than the whole session, so a command
/// can still render while discovery is holding one.
struct ConfigSources<'a> {
    declared: Vec<Declaration>,
    loader: &'a dyn Loader,
    components: std::path::PathBuf,
    /// The directories under `components/`, so a component knows whether one
    /// sits beside its file.
    directories: Vec<std::path::PathBuf>,
}

impl Sources for ConfigSources<'_> {
    fn declared(&self) -> Result<Vec<Declaration>, meowctl_engine::EngineError> {
        Ok(self.declared.clone())
    }

    fn source(&self, id: &ComponentId) -> Result<ComponentSource, meowctl_engine::EngineError> {
        if id.is_module_qualified() {
            let file = self.loader.load(id.as_str()).map_err(|e| {
                meowctl_engine::EngineError::Configuration {
                    path: id.as_str().to_owned(),
                    reason: e.to_string(),
                }
            })?;
            return Ok(ComponentSource {
                text: file.source,
                directory: self.components.join(id.logical_name()),
            });
        }

        // A bare name is a file in the configuration's own tree, in either of
        // the two layouts in use; see [R-ENGINE-017].
        let name = id.logical_name();
        let candidates = [
            format!("components/{name}.star"),
            format!("components/{name}/init.star"),
        ];
        for candidate in &candidates {
            if let Ok(file) = self.loader.load(candidate) {
                let beside = self.components.join(name);
                return Ok(ComponentSource {
                    text: file.source,
                    // The directory beside the file when there is one, so
                    // `render_file` finds the data files; otherwise the
                    // components directory itself.
                    directory: if self.directories.contains(&beside) {
                        beside
                    } else {
                        self.components.clone()
                    },
                });
            }
        }
        // Both are named, because a reader who wrote one of them needs to
        // know which spelling was looked for.
        Err(meowctl_engine::EngineError::Configuration {
            path: name.to_owned(),
            reason: format!("no {} and no {}", candidates[0], candidates[1]),
        })
    }
}

/// Reads the components `init.star` and `local.star` declare.
fn declarations(session: &Session<'_>, loader: &dyn Loader) -> CliResult<Vec<Declaration>> {
    let evaluator = Evaluator::new(session.platform.clone(), loader);
    let mut declared: Vec<Declaration> = Vec::new();

    for path in [session.layout.entry(), session.layout.local_entry()] {
        let Ok(bytes) = session.fs.read(&path) else {
            continue;
        };
        let text = String::from_utf8(bytes).map_err(|_| CliError::Configuration {
            message: format!("{} is not UTF-8", path.display()),
            span: None,
        })?;
        let name = path.display().to_string();
        let evaluated = evaluator
            .evaluate(&name, &text)
            .map_err(|e| CliError::Configuration {
                message: format!("{name}: {e}"),
                span: e.span().cloned(),
            })?;
        declared.extend(
            evaluated
                .declarations
                .components
                .iter()
                .map(|decl| Declaration {
                    name: decl.name.clone(),
                    after: decl.after.clone(),
                }),
        );
    }
    Ok(declared)
}

/// Reads the configuration and discovers what it declares.
///
/// The first pass, shared by every command that needs a graph. It evaluates
/// each component file once and registers the package managers, so a hook in
/// the first component can declare a package the last one handles; see
/// [R-ENGINE-020].
fn resolve(session: &Session<'_>, loader: &ModuleLoader<'_>) -> CliResult<Discovered> {
    let declared = declarations(session, loader)?;
    let components = session.layout.components();
    let directories: Vec<std::path::PathBuf> = session
        .fs
        .read_dir(&components)
        .unwrap_or_default()
        .into_iter()
        .filter(|path| {
            matches!(
                session.fs.entry(path),
                Ok(Some(meowctl_fs::Entry::Directory))
            )
        })
        .collect();
    let sources = ConfigSources {
        declared,
        loader,
        components,
        directories,
    };
    Ok(discover(&sources, loader, &session.platform)?)
}

/// The components `local.star` declares, by logical name.
///
/// Which file a component was declared in decides which of the two package
/// locks its packages land in, and nothing else needs to know; see
/// [R-CONFIG-025].
fn local_declarations(session: &Session<'_>, loader: &dyn Loader) -> CliResult<BTreeSet<String>> {
    let Ok(bytes) = session.fs.read(&session.layout.local_entry()) else {
        return Ok(BTreeSet::new());
    };
    let Ok(text) = String::from_utf8(bytes) else {
        return Ok(BTreeSet::new());
    };
    let path = session.layout.local_entry();
    let name = path.display().to_string();
    let evaluated = Evaluator::new(session.platform.clone(), loader)
        .evaluate(&name, &text)
        .map_err(|e| CliError::Configuration {
            message: format!("{name}: {e}"),
            span: e.span().cloned(),
        })?;
    Ok(evaluated
        .declarations
        .components
        .iter()
        .map(|decl| ComponentId::logical_of(&decl.name).to_owned())
        .collect())
}

/// Records what a run declared in `pkgs.lock` and `pkgs.local.lock`.
///
/// Merged into whatever is there rather than replacing it, and a file with
/// nothing new is left alone, which is what `appendPkgsLock` does. `v0.1.0`
/// records the constraint as both the requested and the installed version,
/// because nothing interrogates the manager for what it actually put down;
/// see [R-CONFIG-025].
fn record_packages(
    session: &mut Session<'_>,
    packages: &BTreeMap<String, Vec<meowctl_starlark::PackageDecl>>,
    local: &BTreeSet<String>,
) -> CliResult<()> {
    if packages.is_empty() {
        return Ok(());
    }

    let mut shared = LockFile::read(session.fs.as_ref(), &session.layout.packages_lock())?;
    let mut owned = LockFile::read(session.fs.as_ref(), &session.layout.local_packages_lock())?;
    let (mut shared_dirty, mut owned_dirty) = (false, false);

    for (component, declarations) in packages {
        let into = if local.contains(component) {
            owned_dirty = true;
            &mut owned
        } else {
            shared_dirty = true;
            &mut shared
        };
        for declaration in declarations {
            into.packages
                .entry(declaration.manager.clone())
                .or_default()
                .insert(
                    declaration.name.clone(),
                    meowctl_config::PackageEntry {
                        requested: declaration.version.clone(),
                        installed: declaration.version.clone(),
                        note: String::new(),
                    },
                );
        }
    }

    if shared_dirty {
        shared.write(session.fs.as_ref(), &session.layout.packages_lock())?;
    }
    if owned_dirty {
        owned.write(session.fs.as_ref(), &session.layout.local_packages_lock())?;
    }
    Ok(())
}

/// Runs a phase set.
pub(super) fn apply(
    session: &mut Session<'_>,
    phase_set: PhaseSet,
    filter: &[String],
    force: bool,
    rollback: bool,
) -> CliResult<()> {
    session.require_configured()?;

    // A journal left behind means the last run stopped partway, and that is
    // worth saying before anything else happens; see [R-ENGINE-042].
    if let Some(records) = interrupted_run(&session.layout.journal()) {
        session.say(&format!(
            "a previous run stopped partway and left {records} operation(s) to undo"
        ));
    }

    let lock = session.lock()?;
    let loader = session.loader(&lock);
    let discovered = resolve(session, &loader)?;
    let graph = Graph::build(&discovered)?;
    let graph = graph.restricted_to(filter)?;

    // A module whose fingerprint moved invalidates every component that came
    // from it, transitive ones included; see [R-ENGINE-043].
    let installed = InstalledLock::read(session.fs.as_ref(), &session.layout.installed())?;
    let ids: Vec<ComponentId> = graph.components().iter().map(|c| c.id.clone()).collect();
    let stale = stale_components(&ids, &installed.fingerprints(), &fingerprints(&ids, &lock));

    let sentinel = Sentinel::read(session.fs.as_ref(), &session.layout.state())?;
    let mut progress = Progress::new(sentinel, session.layout.state());
    progress.forget(session.fs.as_ref(), &stale)?;

    let plan = Plan::compute(
        &graph,
        phase_set,
        &Inputs {
            sentinel: Some(progress.sentinel()),
            filter,
            excluded: &discovered.excluded,
            force,
        },
    );

    // A dry run renders the plan and executes nothing, which is what makes
    // the two agree about what work there is; see [R-CLI-022].
    if session.dry_run {
        session.sink.handle(&Event::PlanComputed {
            phase_set: plan.phase_set,
            steps: plan.steps.clone(),
        });
        return Ok(());
    }

    let effects = meowctl_ctx::Effects {
        fs: Arc::clone(&session.fs),
        exec: Arc::clone(&session.exec),
        http: Arc::clone(&session.http),
        interaction: Arc::clone(&session.interaction),
        journal: Some(Arc::new(std::sync::Mutex::new(
            meowctl_ops::Journal::open(session.layout.journal()).map_err(|e| {
                CliError::General(format!("the rollback journal could not be opened: {e}"))
            })?,
        ))),
        events: Arc::clone(&session.events),
    };

    let evaluator = Evaluator::new(session.platform.clone(), &loader);
    let mut runner = Runner::new(
        &graph,
        &discovered.registry,
        evaluator,
        effects,
        Settings {
            home: session.home.clone(),
            state_root: session.layout.root().join("state"),
            platform: session.platform.clone(),
            environment: session.environment.clone(),
            dry_run: false,
            rollback,
            shell: None,
        },
    )
    .recording(progress);

    let report = runner.run(&plan);

    // Recorded even when the run failed, because a package installed before
    // the failure is installed; `appendPkgsLock` runs on the same footing.
    let local = local_declarations(session, &loader)?;
    record_packages(session, &report.packages, &local)?;

    match report.failure {
        None => Ok(()),
        Some(failure) => Err(CliError::General(failure.to_string())),
    }
}

/// The shell `ctx.shell` reports, from `$SHELL`.
///
/// The login shell rather than the one that spawned this process, which is
/// what `v0.1.0` reads and what a component branching on `ctx.shell` means.
fn current_shell(environment: &std::collections::BTreeMap<String, String>) -> Option<String> {
    let path = environment.get("SHELL")?;
    std::path::Path::new(path)
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
}

/// Runs the runtime hooks for a phase and emits what they contributed.
///
/// The command a shell runs on every spawn. Nothing it does reaches the exit
/// code: a shell that cannot start is worse than a shell that starts without
/// its integration, so a failure is recorded in `.hook-error` and the command
/// still succeeds; see [R-CLI-062].
fn hook(session: &mut Session<'_>, phase: &str) -> CliResult<()> {
    // Not every phase name, only the two a shell spawn runs. The rest belong
    // to a phase set and are reached through `apply`; see [R-CLI-060].
    let phase: Phase = match phase.parse() {
        Ok(phase @ (Phase::Shell | Phase::Login)) => phase,
        _ => {
            return Err(CliError::Usage(format!(
                "hook takes {} or {}, not {phase}",
                Phase::Shell,
                Phase::Login
            )));
        }
    };

    match run_hook(session, phase) {
        Ok(()) => {
            HookError::clear(session.fs.as_ref(), &session.layout.hook_error())?;
            Ok(())
        }
        Err(reason) => {
            let flag = HookError {
                at: rfc3339(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map_or(0, |d| d.as_secs()),
                ),
                reason,
            };
            // The write failing is the one thing left to report, and even
            // that goes to the flag's absence rather than to the shell.
            let _ = flag.write(session.fs.as_ref(), &session.layout.hook_error());
            Ok(())
        }
    }
}

/// Runs one runtime hook phase, reporting why it stopped rather than failing.
///
/// The error is a string because it is written to a file and read by a human,
/// not mapped to an exit code: every path out of `hook` exits zero.
fn run_hook(session: &mut Session<'_>, phase: Phase) -> Result<(), String> {
    let describe = |e: &dyn std::fmt::Display| e.to_string();

    let lock = session.lock().map_err(|e| describe(&e))?;
    let loader = session.loader(&lock);
    let discovered = resolve(session, &loader).map_err(|e| describe(&e))?;
    let graph = Graph::build(&discovered).map_err(|e| describe(&e))?;

    // No journal: a `shell` hook that writes is misusing the phase, and a
    // rollback on a shell spawn would undo the previous one; see
    // [R-CLI-065].
    let effects = meowctl_ctx::Effects {
        fs: Arc::clone(&session.fs),
        exec: Arc::clone(&session.exec),
        http: Arc::clone(&session.http),
        interaction: Arc::clone(&session.interaction),
        journal: None,
        events: Arc::clone(&session.events),
    };

    let evaluator = Evaluator::new(session.platform.clone(), &loader);
    let mut runner = Runner::new(
        &graph,
        &discovered.registry,
        evaluator,
        effects,
        Settings {
            home: session.home.clone(),
            state_root: session.layout.root().join("state"),
            platform: session.platform.clone(),
            environment: session.environment.clone(),
            dry_run: false,
            rollback: false,
            shell: current_shell(&session.environment),
        },
    );

    match runner.run_hook(phase).failure {
        None => Ok(()),
        Some(failure) => Err(failure.to_string()),
    }
}

/// Lists what the manifests declare.
///
/// A directory with no configuration gets an empty list rather than a
/// refusal: "nothing is declared" is an answer to the question; see
/// [R-CLI-050].
fn dep_list(session: &mut Session<'_>) -> CliResult<()> {
    let lock = session.lock()?;
    for (name, entry) in &lock.modules {
        let what = if entry.replaced {
            format!("replaced by {}", entry.path)
        } else if entry.commit_sha.is_empty() {
            entry.version.clone()
        } else {
            entry.commit_sha.clone()
        };
        session.say(&format!("{name} {what}"));
    }
    Ok(())
}

/// The components a file declares, by name.
pub(super) fn parse_components(
    session: &Session<'_>,
    name: &str,
    source: &str,
) -> CliResult<Vec<String>> {
    let loader = NoLoader;
    let evaluated = Evaluator::new(session.platform.clone(), &loader)
        .evaluate(name, source)
        .map_err(|e| CliError::Configuration {
            message: format!("{name}: {e}"),
            span: e.span().cloned(),
        })?;
    Ok(evaluated
        .declarations
        .components
        .iter()
        .map(|decl| decl.name.clone())
        .collect())
}

/// Reads a manifest by evaluating it.
///
/// One evaluator reads every manifest meowctl encounters, so `deps.mod` and a
/// module's `MODULE.meow` cannot drift apart; see [R-STAR-006].
pub(super) fn parse_modfile(
    session: &Session<'_>,
    name: &str,
    source: &str,
) -> CliResult<meowctl_config::Modfile> {
    let loader = NoLoader;
    let evaluated = Evaluator::new(session.platform.clone(), &loader)
        .evaluate(name, source)
        .map_err(|e| CliError::Configuration {
            message: format!("{name}: {e}"),
            span: e.span().cloned(),
        })?;

    Ok(meowctl_config::Modfile {
        module: evaluated
            .declarations
            .module
            .as_ref()
            .map(|decl| meowctl_config::Module {
                name: decl.name.clone(),
                version: decl.version.clone(),
            }),
        deps: evaluated
            .declarations
            .deps
            .iter()
            .map(|dep| meowctl_config::Dep {
                name: dep.name.clone(),
                version: dep.version.clone(),
                source: dep.source.clone(),
            })
            .collect(),
        replaces: evaluated
            .declarations
            .replaces
            .iter()
            .map(|replace| meowctl_config::Replace {
                name: replace.name.clone(),
                path: replace.path.clone(),
                source: replace.source.clone(),
            })
            .collect(),
    })
}

/// Resolves the manifests into their locks.
///
/// Each manifest gets its own lock, and a module in both resolves in each:
/// a machine-local override must not land in the committed lock; see
/// [R-MODULE-052].
pub(super) fn dep_sync(session: &mut Session<'_>, upgrade: &Upgrade) -> CliResult<()> {
    session.require_configured()?;

    let shared_replaces = manifest_of(session, &session.layout.modfile())?.replaces;
    let local_replaces = manifest_of(session, &session.layout.local_modfile())?.replaces;
    // A machine-local override wins wherever both name a module, which is
    // what the local file is for; see [R-MODULE-022].
    let replaces = meowctl_module::overlay_replaces(&shared_replaces, &local_replaces);

    let syncer = meowctl_module::Syncer::new(
        session.fs.as_ref(),
        session.http.as_ref(),
        Cache::new(session.cache.clone()),
        crate::version::STRING,
        now(),
    );

    let mut resolved: Vec<(std::path::PathBuf, meowctl_module::Synced)> = Vec::new();
    for (manifest, lock) in [
        (session.layout.modfile(), session.layout.lock()),
        (session.layout.local_modfile(), session.layout.local_lock()),
    ] {
        if !session.fs.exists(&manifest).unwrap_or(false) {
            continue;
        }
        let declared = manifest_of(session, &manifest)?;
        if declared.deps.is_empty() {
            continue;
        }
        let previous = LockFile::read(session.fs.as_ref(), &lock)?;
        let synced = syncer.sync(&declared.deps, &replaces, &previous, upgrade)?;
        resolved.push((lock, synced));
    }

    for (lock, synced) in resolved {
        for (name, version) in &synced.resolved {
            session.say(&format!("{name} {version}"));
        }
        if session.dry_run {
            continue;
        }
        synced.lock.write(session.fs.as_ref(), &lock)?;
    }
    Ok(())
}

/// Reads a manifest, treating an absent one as empty.
fn manifest_of(
    session: &Session<'_>,
    path: &std::path::Path,
) -> CliResult<meowctl_config::Modfile> {
    let Ok(bytes) = session.fs.read(path) else {
        return Ok(meowctl_config::Modfile::default());
    };
    let text = String::from_utf8(bytes).map_err(|_| CliError::Configuration {
        message: format!("{} is not UTF-8", path.display()),
        span: None,
    })?;
    parse_modfile(session, &path.display().to_string(), &text)
}

/// The time a lock records as its own.
///
/// Read once here rather than inside the resolver, because a component that
/// asks the clock cannot be tested; see [R-CONFIG-022].
fn now() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or_else(|_| "unknown".to_owned(), |d| rfc3339(d.as_secs()))
}

/// Seconds since the epoch, as an RFC 3339 timestamp in UTC.
///
/// Written out rather than pulled in: one format, one call site, and a date
/// library would be a dependency for twenty lines of arithmetic.
fn rfc3339(seconds: u64) -> String {
    const DAYS_IN_MONTH: [u64; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let (mut days, rest) = (seconds / 86_400, seconds % 86_400);
    let (hour, minute, second) = (rest / 3600, (rest % 3600) / 60, rest % 60);

    let mut year = 1970;
    loop {
        let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
        let length = if leap { 366 } else { 365 };
        if days < length {
            break;
        }
        days -= length;
        year += 1;
    }
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let mut month = 1;
    for (index, length) in DAYS_IN_MONTH.iter().enumerate() {
        let length = if index == 1 && leap { 29 } else { *length };
        if days < length {
            break;
        }
        days -= length;
        month += 1;
    }
    format!(
        "{year:04}-{month:02}-{:02}T{hour:02}:{minute:02}:{second:02}Z",
        days + 1
    )
}

/// Reports what the last run did.
///
/// A directory with no configuration gets "no runs recorded" rather than a
/// refusal, for the reason [R-CLI-050] gives.
fn status(session: &mut Session<'_>, all: bool) -> CliResult<()> {
    // A shell whose integration is missing is what the user came to ask
    // about, so it is said before the run metadata rather than after it; see
    // [R-CLI-064].
    if HookError::present(session.fs.as_ref(), &session.layout.hook_error()) {
        session.warn("the last runtime hook failed -- run 'meowctl doctor' for the reason");
    }

    let sentinel = Sentinel::read(session.fs.as_ref(), &session.layout.state())?;

    if sentinel.last_run.phase_set.is_empty() && sentinel.completed_components.is_empty() {
        session.say("no runs recorded");
        return Ok(());
    }

    let started = sentinel
        .last_run
        .started_at
        .map_or_else(|| "never".to_owned(), |when| when.to_string());
    session.say(&format!(
        "last run: {} started {started}, {}",
        sentinel.last_run.phase_set,
        if sentinel.last_run.completed {
            "completed"
        } else {
            "did not complete"
        }
    ));

    // The ten most recent, newest first, with `--all` for the rest: the
    // ledger is append-only and on a real configuration it is 579 lines, which
    // buries the metadata the command exists to report.
    let shown: Vec<&meowctl_config::CompletedComponent> = if all {
        sentinel.completed_components.iter().rev().collect()
    } else {
        sentinel
            .completed_components
            .iter()
            .rev()
            .take(10)
            .collect()
    };
    for record in shown {
        session.say(&format!("{} {}", record.phase, record.component));
    }
    if !all && sentinel.completed_components.len() > 10 {
        let rest = sentinel.completed_components.len() - 10;
        session.say(&format!("{rest} more; --all shows them"));
    }
    Ok(())
}

/// Reports what is wrong with the configuration and the environment.
fn doctor(session: &mut Session<'_>) -> CliResult<()> {
    let root = session.layout.root().to_path_buf();
    let configured = session.layout.check(session.fs.as_ref()).is_ok();
    session.say(&format!("config directory: {}", root.display()));
    session.say(&format!(
        "entry point: {}",
        if configured { "found" } else { "missing" }
    ));

    let modfile_there = session
        .fs
        .exists(&session.layout.modfile())
        .unwrap_or(false);
    session.say(&format!(
        "deps.mod: {}",
        if modfile_there { "found" } else { "missing" }
    ));
    let lock_there = session.fs.exists(&session.layout.lock()).unwrap_or(false);
    session.say(&format!(
        "deps.lock: {}",
        if lock_there { "found" } else { "missing" }
    ));
    session.say(&format!("module cache: {}", session.cache.display()));
    session.say(&format!("platform: {}", session.platform.os));

    // `doctor` is where the reason lives, because `status` is piped and a
    // traceback would bury what it reports; see [R-CLI-064].
    if let Some(flag) = HookError::read(session.fs.as_ref(), &session.layout.hook_error()) {
        session.warn(&format!(
            "runtime hook failed at {}: {}",
            flag.at, flag.reason
        ));
    }
    Ok(())
}

/// Walks a directory and evaluates every component file in it.
fn check(session: &mut Session<'_>, dir: &std::path::Path) -> CliResult<()> {
    let loader = NoLoader;
    let evaluator = Evaluator::new(session.platform.clone(), &loader);
    let mut checked = 0usize;
    let mut problems: Vec<String> = Vec::new();

    // A directory that is not there is a mistake in the argument, not a tree
    // with no components in it.
    // A general failure rather than a configuration one: the directory is an
    // argument to this command, not part of anybody's configuration, and
    // `v0.1.0` exits 1 for it.
    if !session.fs.exists(dir).unwrap_or(false) {
        return Err(CliError::General(format!("{} is not there", dir.display())));
    }

    // Recursively, because components are laid out as `<dir>/<name>/init.star`
    // and a flat walk validates nothing; the `fix(check)` commit is why.
    let mut pending = vec![dir.to_path_buf()];
    while let Some(current) = pending.pop() {
        let Ok(entries) = session.fs.read_dir(&current) else {
            continue;
        };
        for entry in entries {
            let name = entry
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            // A dotted directory cannot contribute a component, and walking
            // `.git` finds files nobody meant to check.
            if name.starts_with('.') {
                continue;
            }
            if matches!(
                session.fs.entry(&entry),
                Ok(Some(meowctl_fs::Entry::Directory))
            ) {
                pending.push(entry);
                continue;
            }
            if !name.ends_with(".star") {
                continue;
            }
            checked += 1;
            let Ok(bytes) = session.fs.read(&entry) else {
                continue;
            };
            let Ok(text) = String::from_utf8(bytes) else {
                problems.push(format!("{}: not UTF-8", entry.display()));
                continue;
            };
            if let Err(e) = evaluator.evaluate(&entry.display().to_string(), &text) {
                problems.push(format!("{}: {e}", entry.display()));
            }
        }
    }

    session.say(&format!(
        "checked {checked} file(s), {} problem(s)",
        problems.len()
    ));
    for problem in &problems {
        session.sink.handle(&Event::Message {
            level: Level::Error,
            text: problem.clone(),
        });
    }
    if problems.is_empty() {
        Ok(())
    } else {
        Err(CliError::Configuration {
            message: format!("{} component file(s) did not evaluate", problems.len()),
            span: None,
        })
    }
}

/// Writes to standard output directly.
///
/// The one command whose standard output another program reads, so the text
/// is written verbatim rather than rendered; see [R-CLI-021].
fn print_to_stdout(text: &str) {
    use std::io::Write as _;
    let mut out = std::io::stdout();
    let _ = out.write_all(text.as_bytes());
    let _ = out.flush();
}
