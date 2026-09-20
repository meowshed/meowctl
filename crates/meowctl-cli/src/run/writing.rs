//! The commands that change a configuration.
//!
//! Every one of them is the same three steps: edit a file, write it, and run
//! an apply over what changed. They are together because that shape is what
//! they share, and apart from the reading commands because it is not the
//! shape those have.

use meowctl_common::PhaseSet;
use meowctl_config::{Dep, Layout, Modfile, edit};
use meowctl_fs::FileSystem;

use super::Session;
use super::commands::apply;
use crate::templates;
use crate::{CliError, CliResult};

/// The files `init` creates and `.gitignore` must list.
const GITIGNORED: [&str; 3] = ["local.star", "deps.local.mod", "deps.local.lock"];

/// Scaffolds a configuration directory.
///
/// # Errors
///
/// [`CliError::Configuration`] when `init.star` is already there and the
/// caller did not ask to overwrite it: a configuration is somebody's work,
/// and `--force` is how they say they meant it.
pub(super) fn init(session: &mut Session<'_>, force: bool) -> CliResult<()> {
    let layout = session.layout.clone();
    let entry = layout.entry();

    if session.fs.exists(&entry)? && !force {
        return Err(CliError::Configuration {
            message: format!(
                "{} already exists; delete it or run `meowctl init --force`",
                entry.display()
            ),
            span: None,
        });
    }

    for directory in [
        layout.root().to_path_buf(),
        layout.components(),
        layout.root().join("hooks"),
    ] {
        session.fs.create_dir_all(&directory)?;
    }

    session.fs.write(&entry, templates::INIT_STAR.as_bytes())?;

    // The machine-local files are only written when absent: a re-run with
    // `--force` is about the entry point, not about what this machine added.
    for (path, contents) in [
        (layout.local_entry(), templates::LOCAL_STAR),
        (layout.local_modfile(), templates::LOCAL_MOD),
    ] {
        if !session.fs.exists(&path)? {
            session.fs.write(&path, contents.as_bytes())?;
        }
    }

    Modfile {
        module: Some(meowctl_config::Module {
            name: templates::MODULE_NAME.to_owned(),
            version: templates::MODULE_VERSION.to_owned(),
        }),
        ..Modfile::default()
    }
    .write(session.fs.as_ref(), &layout.modfile())?;

    ignore(session, &layout)?;

    session.say(&format!("initialized {}", layout.root().display()));
    session.say("edit init.star to declare components, then run `meowctl apply`");
    Ok(())
}

/// Makes sure `.gitignore` lists the machine-local files.
///
/// Appended rather than rewritten, because the file belongs to whoever made
/// the repository and may already say things meowctl knows nothing about.
fn ignore(session: &Session<'_>, layout: &Layout) -> CliResult<()> {
    let path = layout.root().join(".gitignore");
    let existing = session
        .fs
        .read(&path)
        .ok()
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .unwrap_or_default();

    let mut out = existing.clone();
    for entry in GITIGNORED {
        if existing.lines().any(|line| line.trim() == entry) {
            continue;
        }
        if !out.is_empty() && !out.ends_with('\n') {
            out.push('\n');
        }
        out.push_str(entry);
        out.push('\n');
    }
    if out != existing {
        session.fs.write(&path, out.as_bytes())?;
    }
    Ok(())
}

/// Adds components to `local.star` and installs them.
pub(super) fn add(
    session: &mut Session<'_>,
    components: &[String],
    force: bool,
    rollback: bool,
) -> CliResult<()> {
    edit_local(session, components, true)?;
    // Scoped to what was added, and to what those depend on; see
    // [R-ENGINE-016].
    apply(session, PhaseSet::Install, components, force, rollback)
}

/// Removes components from `local.star` and uninstalls them.
pub(super) fn remove(
    session: &mut Session<'_>,
    components: &[String],
    rollback: bool,
) -> CliResult<()> {
    // Uninstalled first, while the declaration is still there to run the
    // hooks from: a component removed from the file is a component nothing
    // can uninstall.
    apply(session, PhaseSet::Uninstall, components, false, rollback)?;
    edit_local(session, components, false)
}

/// Adds or removes declarations in `local.star`.
fn edit_local(session: &mut Session<'_>, components: &[String], adding: bool) -> CliResult<()> {
    let path = session.layout.local_entry();
    let name = path.display().to_string();
    let source = session
        .fs
        .read(&path)
        .ok()
        .and_then(|bytes| String::from_utf8(bytes).ok())
        .unwrap_or_default();

    let mut edited = source.clone();
    for component in components {
        edited = if adding {
            edit::add_component(&name, &edited, component)?
        } else {
            edit::remove_component(&name, &edited, component)?
        };
    }

    if edited == source {
        session.say("nothing to change in local.star");
        return Ok(());
    }
    session.fs.write(&path, edited.as_bytes())?;
    for component in components {
        session.say(&format!(
            "{} {component}",
            if adding { "added" } else { "removed" }
        ));
    }
    Ok(())
}

/// Which manifest a `dep` subcommand acts on.
fn manifest(layout: &Layout, local: bool) -> std::path::PathBuf {
    if local {
        layout.local_modfile()
    } else {
        layout.modfile()
    }
}

/// Adds a dependency to a manifest.
pub(super) fn dep_add(
    session: &mut Session<'_>,
    name: &str,
    version: Option<&str>,
    source: Option<&str>,
    local: bool,
) -> CliResult<()> {
    // Exactly one of the two, never both and never neither, which is what a
    // `dep()` may carry; see [R-CONFIG-012].
    let dep = match (version, source) {
        (Some(version), None) => Dep {
            name: name.to_owned(),
            version: version.to_owned(),
            source: String::new(),
        },
        (None, Some(source)) => Dep {
            name: name.to_owned(),
            version: String::new(),
            source: source.to_owned(),
        },
        (Some(_), Some(_)) => {
            return Err(CliError::Usage(
                "a dependency has a version or a source, not both".to_owned(),
            ));
        }
        (None, None) => {
            return Err(CliError::Usage(
                "a dependency needs a --version or a --source".to_owned(),
            ));
        }
    };

    let path = manifest(&session.layout, local);
    let mut modfile = read_modfile(session, &path)?;
    if modfile.deps.iter().any(|held| held.name == name) {
        return Err(CliError::Configuration {
            message: format!("{name} is already declared in {}", path.display()),
            span: None,
        });
    }
    modfile.deps.push(dep);
    modfile.write(session.fs.as_ref(), &path)?;
    session.say(&format!("added {name} to {}", path.display()));
    Ok(())
}

/// Removes a dependency from a manifest.
pub(super) fn dep_remove(session: &mut Session<'_>, name: &str, local: bool) -> CliResult<()> {
    let path = manifest(&session.layout, local);
    let mut modfile = read_modfile(session, &path)?;
    let before = modfile.deps.len();
    modfile.deps.retain(|dep| dep.name != name);
    if modfile.deps.len() == before {
        return Err(CliError::Configuration {
            message: format!("{name} is not declared in {}", path.display()),
            span: None,
        });
    }
    modfile.write(session.fs.as_ref(), &path)?;
    session.say(&format!("removed {name} from {}", path.display()));
    Ok(())
}

/// Reads a manifest, treating an absent one as empty.
///
/// Absent is ordinary: `deps.local.mod` exists only on a machine that has
/// overridden something.
fn read_modfile(session: &Session<'_>, path: &std::path::Path) -> CliResult<Modfile> {
    let Ok(bytes) = session.fs.read(path) else {
        return Ok(Modfile::default());
    };
    let text = String::from_utf8(bytes).map_err(|_| CliError::Configuration {
        message: format!("{} is not UTF-8", path.display()),
        span: None,
    })?;
    super::commands::parse_modfile(session, &path.display().to_string(), &text)
}

/// Bootstraps a configuration from a public dotfiles repository.
///
/// Over HTTPS with no `git` subprocess, because a machine being set up may
/// not have `git` yet -- which is exactly when this runs; see [R-CLI-003].
pub(super) fn bootstrap(session: &mut Session<'_>, repo_url: &str, force: bool) -> CliResult<()> {
    let entry = session.layout.entry();
    if session.fs.exists(&entry)? && !force {
        return Err(CliError::Configuration {
            message: format!(
                "{} already exists; delete it or run `meowctl init --force`",
                entry.display()
            ),
            span: None,
        });
    }

    let url = tarball_url(repo_url)?;
    session.say(&format!("fetching {url}"));
    let archive = session
        .http
        .get(&url)
        .map_err(|e| CliError::Module(format!("{url}: {e}")))?;

    // The same extraction a module gets, with the same containment check: a
    // downloaded tarball is remote input whoever published it; see
    // [R-MODULE-033] and [R-MODULE-034].
    let files = meowctl_module::archive::strip_single_root(meowctl_module::archive::read(
        repo_url, &archive,
    )?);
    meowctl_module::archive::write_into(session.fs.as_ref(), session.layout.root(), &files)?;

    if !session.fs.exists(&entry)? {
        return Err(CliError::Configuration {
            message: format!("{repo_url} has no init.star at its root"),
            span: None,
        });
    }
    session.say(&format!(
        "bootstrapped {} from {repo_url}",
        session.layout.root().display()
    ));

    // The lock that came with the repository wins: it is what its author
    // tested against, and re-resolving would move every version.
    if !session.fs.exists(&session.layout.lock())? {
        super::commands::dep_sync(session, &meowctl_module::Upgrade::Nothing)?;
    }
    apply(session, PhaseSet::Install, &[], false, true)
}

/// Where a repository's default branch is served as a tarball.
fn tarball_url(repo_url: &str) -> CliResult<String> {
    let trimmed = repo_url.trim_end_matches('/').trim_end_matches(".git");
    if !trimmed.starts_with("https://") {
        return Err(CliError::Usage(format!(
            "{repo_url} is not an https URL; meowctl fetches over HTTPS and never shells out to git"
        )));
    }
    Ok(format!("{trimmed}/archive/refs/heads/main.tar.gz"))
}

/// Removes dependencies nothing references.
pub(super) fn dep_tidy(session: &mut Session<'_>) -> CliResult<()> {
    session.require_configured()?;

    let referenced = referenced_modules(session)?;
    for local in [false, true] {
        let path = manifest(&session.layout, local);
        if !session.fs.exists(&path)? {
            continue;
        }
        let mut modfile = read_modfile(session, &path)?;
        let before = modfile.deps.len();
        modfile.deps.retain(|dep| referenced.contains(&dep.name));
        let removed = before - modfile.deps.len();
        if removed == 0 {
            continue;
        }
        if session.dry_run {
            session.say(&format!(
                "{} would lose {removed} unreferenced dependency(ies)",
                path.display()
            ));
            continue;
        }
        modfile.write(session.fs.as_ref(), &path)?;
        session.say(&format!(
            "removed {removed} unreferenced dependency(ies) from {}",
            path.display()
        ));
    }
    Ok(())
}

/// Which modules the configuration's own files name.
///
/// Read from the `component()` declarations rather than by scanning for text,
/// because a module named in a comment is not a module anything uses.
fn referenced_modules(session: &Session<'_>) -> CliResult<std::collections::BTreeSet<String>> {
    let mut referenced = std::collections::BTreeSet::new();
    for path in [session.layout.entry(), session.layout.local_entry()] {
        let Ok(bytes) = session.fs.read(&path) else {
            continue;
        };
        let Ok(text) = String::from_utf8(bytes) else {
            continue;
        };
        let name = path.display().to_string();
        let parsed = super::commands::parse_components(session, &name, &text)?;
        for component in parsed {
            let id: meowctl_common::ComponentId = match component.parse() {
                Ok(id) => id,
                Err(_) => continue,
            };
            if id.is_module_qualified() {
                referenced.insert(id.module_key().to_owned());
            }
        }
    }
    Ok(referenced)
}

/// Re-downloads the dotfiles repository and applies what changed.
pub(super) fn update(session: &mut Session<'_>, yes: bool, rollback: bool) -> CliResult<()> {
    session.require_configured()?;

    let sentinel = meowctl_config::Sentinel::read(session.fs.as_ref(), &session.layout.state())?;
    if sentinel.repo_url.is_empty() {
        return Err(CliError::Configuration {
            message: "this configuration was not bootstrapped from a repository, so there is nothing to pull".to_owned(),
            span: None,
        });
    }

    let url = tarball_url(&sentinel.repo_url)?;
    session.say(&format!("fetching {url}"));
    let archive = session
        .http
        .get(&url)
        .map_err(|e| CliError::Module(format!("{url}: {e}")))?;

    let files = meowctl_module::archive::strip_single_root(meowctl_module::archive::read(
        &sentinel.repo_url,
        &archive,
    )?);

    // What the remote holds that this machine does not, or holds differently.
    // The machine-local files are never touched: they are what this machine
    // added, and the remote knows nothing about them.
    let root = session.layout.root().to_path_buf();
    let mut changed: Vec<&meowctl_module::archive::ArchiveFile> = Vec::new();
    for file in &files {
        if GITIGNORED.iter().any(|held| file.path == *held) {
            continue;
        }
        let target = root.join(file.path.replace('/', std::path::MAIN_SEPARATOR_STR));
        let same = session
            .fs
            .read(&target)
            .is_ok_and(|held| held == file.contents);
        if !same {
            changed.push(file);
        }
    }

    if changed.is_empty() {
        session.say("already up to date");
        return Ok(());
    }
    for file in &changed {
        session.say(&format!("would write {}", file.path));
    }
    if session.dry_run {
        return Ok(());
    }

    if !yes {
        let mut interaction = session
            .interaction
            .lock()
            .map_err(|_| CliError::General("the prompt is unusable".to_owned()))?;
        let confirmed = interaction
            .confirm(&format!("Write {} file(s)?", changed.len()))
            .map_err(|e| CliError::General(e.to_string()))?;
        drop(interaction);
        if !confirmed {
            session.say("nothing was written");
            return Ok(());
        }
    }

    let owned: Vec<meowctl_module::archive::ArchiveFile> = changed.into_iter().cloned().collect();
    meowctl_module::archive::write_into(session.fs.as_ref(), &root, &owned)?;
    super::commands::dep_sync(session, &meowctl_module::Upgrade::Nothing)?;
    apply(session, PhaseSet::Install, &[], false, rollback)
}
