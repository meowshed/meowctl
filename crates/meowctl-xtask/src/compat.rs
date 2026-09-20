//! The v0.1.0 compatibility corpus.
//!
//! A case is a configuration directory plus a list of commands. Each command
//! runs in its own sandbox: a fresh `HOME`, a copy of the configuration, and a
//! shared read-only module cache so a run needs no network. What gets recorded
//! is the exit code, the sandbox file tree with a hash per file, and the
//! contents of every file under the configuration directory, which is where
//! meowctl writes its locks and its state.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha256};

/// Where the sandbox, the home, and the configuration directory are rewritten
/// to before anything is recorded or compared.
///
/// Normalising hides a class of real difference: a binary that printed the
/// wrong path would still match if the wrong path happened to be the sandbox.
/// The alternative is a corpus that cannot be recorded at all, because a fresh
/// sandbox has a fresh path and a run has a fresh clock. The rewrite is kept
/// narrow for that reason: three paths and one timestamp shape, nothing else.
const SANDBOX_TOKEN: &str = "<SANDBOX>";
const HOME_TOKEN: &str = "<HOME>";
const CONFIG_TOKEN: &str = "<CONFIG>";
const TIMESTAMP_TOKEN: &str = "<TIMESTAMP>";
const BUILD_TOKEN: &str = "<BUILD>";

/// Fixtures are stored per platform, because the corpus compares two binaries
/// on one machine and not two machines. A `select` on the platform produces
/// different files on macOS and Linux, and a dry run lists different
/// components; comparing a macOS recording against a Linux run would report
/// those as rewrite defects.
fn platform_key() -> String {
    format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH)
}

/// Rewrites the volatile parts of `text` so two runs of the same command
/// produce the same bytes.
fn normalize(text: &str, sandbox: &Path, home: &Path, config: &Path) -> String {
    // Longest first: the config and home directories live inside the sandbox,
    // so rewriting the sandbox first would leave `<SANDBOX>/config` behind.
    let mut out = text.replace(&config.to_string_lossy().into_owned(), CONFIG_TOKEN);
    out = out.replace(&home.to_string_lossy().into_owned(), HOME_TOKEN);
    out = out.replace(&sandbox.to_string_lossy().into_owned(), SANDBOX_TOKEN);
    replace_build_metadata(&replace_timestamps(&out))
}

/// Rewrites the commit hash `meowctl version` prints. It changes on every
/// commit, and a fixture that drifts on every commit is one nobody trusts. The
/// version and the target are left alone: those are what the command exists to
/// report.
fn replace_build_metadata(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find("commit ") {
        out.push_str(&rest[..at + "commit ".len()]);
        let tail = &rest[at + "commit ".len()..];
        let end = tail
            .find(|c: char| !c.is_ascii_alphanumeric())
            .unwrap_or(tail.len());
        out.push_str(BUILD_TOKEN);
        rest = &tail[end..];
    }
    out.push_str(rest);
    out
}

/// Replaces anything shaped like an RFC 3339 instant, and the bare `YYYY-MM-DD`
/// that `meowctl version` prints as its build date. Written by hand rather
/// than with a regular expression crate, because one shape is all the corpus
/// needs and a dependency for it would be hard to justify.
fn replace_timestamps(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < bytes.len() {
        if let Some(end) = timestamp_end(bytes, i) {
            out.push_str(TIMESTAMP_TOKEN);
            i = end;
        } else {
            let ch_len = text[i..].chars().next().map_or(1, char::len_utf8);
            out.push_str(&text[i..i + ch_len]);
            i += ch_len;
        }
    }
    out
}

/// Returns the index just past a timestamp starting at `i`, if one is there.
/// The shape is `YYYY-MM-DDTHH:MM:SS`, plus optional fractional seconds and a
/// `Z`, which covers both what `state.toml` writes and what the lock files do.
fn timestamp_end(b: &[u8], i: usize) -> Option<usize> {
    let digits = |at: usize, n: usize| -> bool {
        at + n <= b.len() && b[at..at + n].iter().all(u8::is_ascii_digit)
    };
    let is_date = digits(i, 4)
        && b.get(i + 4) == Some(&b'-')
        && digits(i + 5, 2)
        && b.get(i + 7) == Some(&b'-')
        && digits(i + 8, 2);
    if !is_date {
        return None;
    }
    // A bare date, as the version line's build date, ends here.
    if !(b.get(i + 10) == Some(&b'T')
        && digits(i + 11, 2)
        && b.get(i + 13) == Some(&b':')
        && digits(i + 14, 2)
        && b.get(i + 16) == Some(&b':')
        && digits(i + 17, 2))
    {
        return Some(i + 10);
    }
    let mut end = i + 19;
    if b.get(end) == Some(&b'.') {
        end += 1;
        while end < b.len() && b[end].is_ascii_digit() {
            end += 1;
        }
    }
    if b.get(end) == Some(&b'Z') {
        end += 1;
    }
    Some(end)
}

/// A configuration the corpus exercises.
struct Case {
    name: &'static str,
    /// Configuration directory, relative to the repository root.
    config: PathBuf,
    /// Whether this configuration's hooks are safe to execute.
    ///
    /// True only for a configuration written for the corpus, whose hooks are
    /// in this repository and touch nothing outside the sandbox. A
    /// configuration belonging to a person runs their components, and a
    /// component may do anything a command can do.
    hooks_are_ours: bool,
}

impl Case {
    /// Whether this command may run against this configuration.
    fn may_run(&self, inv: &Invocation) -> bool {
        self.hooks_are_ours || !inv.runs_hooks
    }
}

/// One command, with the slug its fixtures are stored under.
struct Invocation {
    slug: &'static str,
    args: &'static [&'static str],
    /// True when stdout is a machine interface and must match byte for byte.
    stdout_is_interface: bool,
    /// True when the command evaluates a component's hooks.
    ///
    /// A hook runs whatever the configuration tells it to. The standard
    /// library's `verify` hooks call `open -a <App>` to check an application
    /// is installed, so running one against somebody's real configuration
    /// launches their applications. That happened once, and this flag is what
    /// stops it; see [`Case::may_run`].
    runs_hooks: bool,
}

/// `verify` is deliberately absent. Its whole job is to execute a component's
/// verification hook, and in the standard library that means launching the
/// application being verified. There is no version of that which is polite to
/// run repeatedly, so the corpus does not run it at all; `apply --dry-run`
/// already covers evaluation, the graph, and planning.
const INVOCATIONS: &[Invocation] = &[
    // `init` scaffolds five files and every one of them is machine-written,
    // so they compare byte for byte. It runs against the empty configuration
    // only: it refuses to write over an `init.star` that is already there.
    Invocation {
        slug: "init",
        args: &["init"],
        stdout_is_interface: false,
        runs_hooks: false,
    },
    Invocation {
        slug: "version",
        args: &["version"],
        stdout_is_interface: false,
        runs_hooks: false,
    },
    Invocation {
        slug: "status",
        args: &["status"],
        stdout_is_interface: false,
        runs_hooks: false,
    },
    // `--format json` is part of the output redesign rather than of the
    // parity contract: `v0.1.0` has one hand-written JSON body shaped like a
    // list of checks, and `v0.2.0` emits the event stream from every command.
    // Comparing them would hold the shape the redesign removes; what checks
    // the new one is [R-TUI-030] against a recorded stream. The exit code and
    // the file tree are still compared; see [R-CLI-041].
    Invocation {
        slug: "doctor-json",
        args: &["doctor", "--json"],
        stdout_is_interface: false,
        runs_hooks: false,
    },
    Invocation {
        slug: "dep-list",
        args: &["dep", "list"],
        stdout_is_interface: false,
        runs_hooks: false,
    },
    Invocation {
        slug: "apply-dry-run",
        args: &["apply", "--dry-run"],
        stdout_is_interface: false,
        runs_hooks: true,
    },
    Invocation {
        slug: "shell-fish",
        args: &["shell", "fish"],
        stdout_is_interface: true,
        runs_hooks: true,
    },
    Invocation {
        slug: "shell-zsh",
        args: &["shell", "zsh"],
        stdout_is_interface: true,
        runs_hooks: true,
    },
    Invocation {
        slug: "shell-bash",
        args: &["shell", "bash"],
        stdout_is_interface: true,
        runs_hooks: true,
    },
    Invocation {
        slug: "shell-posix",
        args: &["shell", "posix"],
        stdout_is_interface: true,
        runs_hooks: true,
    },
];
/// What one command produced. Everything here is compared except `stdout`,
/// which is compared only when the invocation says it is an interface.
#[derive(Debug, PartialEq, Eq)]
struct Observation {
    exit: i32,
    stdout: String,
    /// Path relative to the sandbox, to a content hash. Directories are
    /// recorded with an empty hash so an empty directory still shows up.
    tree: BTreeMap<String, String>,
    /// Path relative to the configuration directory, to its contents. Only
    /// text files are captured; a binary file appears in `tree` and not here.
    files: BTreeMap<String, String>,
}

fn repo_root() -> Result<PathBuf> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .context("locating the repository root from CARGO_MANIFEST_DIR")
}

fn cases(root: &Path) -> Result<Vec<Case>> {
    let mut cases = vec![
        Case {
            name: "smoke",
            config: root.join("tests/compat/configs/smoke"),
            hooks_are_ours: true,
        },
        // A directory with nothing in it, so `init` has somewhere to
        // scaffold: every other case already holds an `init.star`, which it
        // refuses to write over.
        Case {
            name: "empty",
            config: root.join("tests/compat/configs/empty"),
            hooks_are_ours: true,
        },
    ];

    if let Ok(extra) = std::env::var("MEOWCTL_COMPAT_EXTRA") {
        let path = PathBuf::from(&extra);
        if !path.is_dir() {
            bail!("MEOWCTL_COMPAT_EXTRA is not a directory: {extra}");
        }
        cases.push(Case {
            name: "extra",
            config: path,
            hooks_are_ours: false,
        });
    }

    Ok(cases)
}

/// The binary the corpus was recorded from: the v0.1.0 Go tree.
fn oracle_binary(root: &Path) -> Result<PathBuf> {
    let path = root.join("bin/meowctl");
    if !path.is_file() {
        bail!(
            "the v0.1.0 binary is missing; run `mise run go-build` first ({})",
            path.display()
        );
    }
    Ok(path)
}

/// The binary under test. Absent until the CLI crate exists, which is why
/// `check` reports that rather than failing.
///
/// `MEOWCTL_COMPAT_SUBJECT` overrides it. Pointing it at the oracle is how the
/// harness checks itself: a run against the binary the fixtures were recorded
/// from must report no differences, and if it does not, the corpus is
/// measuring its own noise rather than the rewrite.
fn subject_binary(root: &Path) -> Option<PathBuf> {
    if let Ok(path) = std::env::var("MEOWCTL_COMPAT_SUBJECT") {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Some(path);
        }
    }
    for profile in ["debug", "release"] {
        let path = root.join("target").join(profile).join("meowctl");
        if path.is_file() {
            return Some(path);
        }
    }
    None
}

pub fn list() -> Result<bool> {
    let root = repo_root()?;
    println!("platform: {}\n", platform_key());
    for case in cases(&root)? {
        println!("{} ({})", case.name, case.config.display());
        for inv in INVOCATIONS {
            let compared = if inv.stdout_is_interface {
                " [stdout compared]"
            } else {
                ""
            };
            let skipped = if case.may_run(inv) {
                ""
            } else {
                "  -- skipped: it would run your hooks"
            };
            println!("    meowctl {}{compared}{skipped}", inv.args.join(" "));
        }
    }
    Ok(true)
}

pub fn record() -> Result<bool> {
    let root = repo_root()?;
    let binary = oracle_binary(&root)?;
    let fixtures = root.join("tests/compat/fixtures").join(platform_key());

    for case in cases(&root)? {
        for inv in INVOCATIONS.iter().filter(|i| case.may_run(i)) {
            let observed = run(&binary, &case, inv)?;
            let dir = fixtures.join(case.name).join(inv.slug);
            write_observation(&dir, &observed)?;
            println!("recorded {}/{}", case.name, inv.slug);
        }
    }
    Ok(true)
}

pub fn check() -> Result<bool> {
    let root = repo_root()?;
    let fixtures = root.join("tests/compat/fixtures").join(platform_key());
    let Some(binary) = subject_binary(&root) else {
        println!(
            "no Rust binary yet; the corpus has nothing to check against.\n\
             This is expected until the CLI crate lands."
        );
        return Ok(true);
    };

    let mut failures = 0usize;
    for case in cases(&root)? {
        for inv in INVOCATIONS.iter().filter(|i| case.may_run(i)) {
            let dir = fixtures.join(case.name).join(inv.slug);
            if !dir.is_dir() {
                println!(
                    "MISSING {}/{} on {}: no fixture; run `cargo xtask compat record`",
                    case.name,
                    inv.slug,
                    platform_key()
                );
                failures += 1;
                continue;
            }
            let expected = read_observation(&dir)?;
            let observed = run(&binary, &case, inv)?;
            let diffs = diff(&expected, &observed, inv.stdout_is_interface);
            if diffs.is_empty() {
                println!("ok      {}/{}", case.name, inv.slug);
            } else {
                failures += 1;
                println!("FAIL    {}/{}", case.name, inv.slug);
                for d in diffs {
                    println!("          {d}");
                }
            }
        }
    }

    if failures > 0 {
        println!("\n{failures} difference(s) against v0.1.0");
        return Ok(false);
    }
    println!("\nno differences against v0.1.0");
    Ok(true)
}

/// Runs one command in a fresh sandbox and observes what it did.
fn run(binary: &Path, case: &Case, inv: &Invocation) -> Result<Observation> {
    let sandbox = tempdir()?;
    let home = sandbox.join("home");
    let config = sandbox.join("config");
    fs::create_dir_all(&home)?;
    copy_dir(&case.config, &config)
        .with_context(|| format!("copying the {} configuration", case.name))?;

    // Everything meowctl might write lands under the sandbox. The module cache
    // is the one thing shared with the machine, so a run needs no network; see
    // R-MODULE-043.
    //
    // It is shared by linking it into the sandbox home rather than through
    // XDG_CACHE_HOME, because v0.1.0's `cacheDir` ignores that variable and
    // always looks under $HOME. Only the meowctl subdirectory is linked, so
    // nothing else in the real cache is reachable from the sandbox.
    link_module_cache(&home)?;

    let mut cmd = Command::new(binary);
    cmd.arg("--config")
        .arg(&config)
        .args(inv.args)
        .env_clear()
        .env("HOME", &home)
        .env("PATH", std::env::var("PATH").unwrap_or_default())
        .env("MEOWCTL_OUTPUT", "plain")
        .env("NO_COLOR", "1")
        .env("TERM", "dumb")
        .current_dir(&sandbox);

    let out = cmd.output().context("running meowctl")?;

    let raw_stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    let observation = Observation {
        exit: out.status.code().unwrap_or(-1),
        stdout: normalize(&raw_stdout, &sandbox, &home, &config),
        tree: hash_tree(&sandbox, &home, &config)?,
        files: capture_text_files(&config, &sandbox, &home, &config)?,
    };

    let _ = fs::remove_dir_all(&sandbox);
    Ok(observation)
}

/// Links the machine's module cache into the sandbox home, so a module the
/// configuration depends on is already there and the run needs no network.
///
/// Absent on a machine that has never fetched a module, which is fine: a
/// configuration with no dependencies never looks, and one with dependencies
/// fails the same way under both binaries.
fn link_module_cache(home: &Path) -> Result<()> {
    let Some(real) = real_module_cache() else {
        return Ok(());
    };
    if !real.is_dir() {
        return Ok(());
    }
    let cache = home.join(".cache");
    fs::create_dir_all(&cache)?;
    copy_symlink(&real, &cache.join("meowctl"))
}

/// Where `v0.1.0` keeps its module cache: `$HOME/.cache/meowctl`, with no
/// `XDG_CACHE_HOME` in it; see `cacheDir` in `internal/cli/sync.go`.
fn real_module_cache() -> Option<PathBuf> {
    std::env::var("HOME")
        .ok()
        .map(|h| PathBuf::from(h).join(".cache").join("meowctl"))
}

fn diff(expected: &Observation, observed: &Observation, compare_stdout: bool) -> Vec<String> {
    let mut out = Vec::new();

    if expected.exit != observed.exit {
        out.push(format!(
            "exit code: expected {}, got {}",
            expected.exit, observed.exit
        ));
    }

    if compare_stdout && expected.stdout != observed.stdout {
        out.push(format!(
            "stdout differs ({} vs {} bytes); first difference at byte {}",
            expected.stdout.len(),
            observed.stdout.len(),
            first_difference(&expected.stdout, &observed.stdout)
        ));
    }

    for (path, hash) in &expected.tree {
        match observed.tree.get(path) {
            None => out.push(format!("missing from the tree: {path}")),
            Some(other) if other != hash => out.push(format!("content differs: {path}")),
            Some(_) => {}
        }
    }
    for path in observed.tree.keys() {
        if !expected.tree.contains_key(path) {
            out.push(format!("unexpected in the tree: {path}"));
        }
    }

    for (path, want) in &expected.files {
        if let Some(got) = observed.files.get(path).filter(|got| *got != want) {
            out.push(format!(
                "{path} differs; first difference at byte {}",
                first_difference(want, got)
            ));
        }
    }

    out
}

fn first_difference(a: &str, b: &str) -> usize {
    a.bytes().zip(b.bytes()).take_while(|(x, y)| x == y).count()
}

fn write_observation(dir: &Path, o: &Observation) -> Result<()> {
    if dir.exists() {
        fs::remove_dir_all(dir)?;
    }
    fs::create_dir_all(dir)?;
    fs::write(dir.join("exit"), format!("{}\n", o.exit))?;
    fs::write(dir.join("stdout"), &o.stdout)?;

    let tree: String = o.tree.iter().map(|(p, h)| format!("{h}  {p}\n")).collect();
    fs::write(dir.join("tree"), tree)?;

    let files_dir = dir.join("files");
    fs::create_dir_all(&files_dir)?;
    for (path, contents) in &o.files {
        let target = files_dir.join(path);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(target, contents)?;
    }
    Ok(())
}

fn read_observation(dir: &Path) -> Result<Observation> {
    let exit = fs::read_to_string(dir.join("exit"))?
        .trim()
        .parse::<i32>()?;
    let stdout = fs::read_to_string(dir.join("stdout")).unwrap_or_default();

    let mut tree = BTreeMap::new();
    for line in fs::read_to_string(dir.join("tree"))
        .unwrap_or_default()
        .lines()
    {
        if let Some((hash, path)) = line.split_once("  ") {
            tree.insert(path.to_owned(), hash.to_owned());
        }
    }

    let files_dir = dir.join("files");
    let mut files = BTreeMap::new();
    if files_dir.is_dir() {
        for entry in walkdir::WalkDir::new(&files_dir)
            .into_iter()
            .filter_map(Result::ok)
        {
            if entry.file_type().is_file() {
                let rel = entry
                    .path()
                    .strip_prefix(&files_dir)?
                    .to_string_lossy()
                    .into_owned();
                files.insert(rel, fs::read_to_string(entry.path())?);
            }
        }
    }

    Ok(Observation {
        exit,
        stdout,
        tree,
        files,
    })
}

/// Directory names whose contents are not meowctl's effects.
///
/// A hook runs real commands, and those commands write their own state:
/// Homebrew's API cache, `gh`'s device id. Recording them makes the corpus
/// compare a third-party tool's bookkeeping instead of the rewrite, and none
/// of it is reproducible between two runs seconds apart.
///
/// The list also covers the module cache the harness links into the sandbox
/// home, which exists only on a machine that has fetched a module.
///
/// The exclusion is by directory name rather than by path, and the list is
/// short on purpose. It hides a real effect if meowctl ever writes into one of
/// these, which is worth knowing; the configuration directory, where meowctl
/// actually writes, is never excluded.
const VOLATILE_DIRS: &[&str] = &[".cache", "Caches", "state"];

/// Every path under `root`, with a content hash per file. Symlinks are
/// recorded by their target rather than followed, because a symlink meowctl
/// created is the effect being checked.
fn hash_tree(root: &Path, home: &Path, config: &Path) -> Result<BTreeMap<String, String>> {
    let mut tree = BTreeMap::new();
    for entry in walkdir::WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
    {
        let path = entry.path();
        if path == root {
            continue;
        }
        let rel = path.strip_prefix(root)?.to_string_lossy().into_owned();
        let meta = entry.path().symlink_metadata()?;

        // Skipped outright, not recorded as existing. Whether one of these is
        // there at all depends on the machine: the harness links the module
        // cache into the sandbox home only when the machine has one, so
        // recording its existence made a fixture that passed where a cache was
        // present and failed where it was not.
        if path
            .ancestors()
            .filter_map(Path::file_name)
            .any(|n| VOLATILE_DIRS.contains(&n.to_string_lossy().as_ref()))
        {
            continue;
        }

        let hash = if meta.is_symlink() {
            format!("symlink:{}", fs::read_link(path)?.display())
        } else if meta.is_dir() {
            String::new()
        } else {
            let bytes = fs::read(path)?;
            // Text is hashed after normalising, so a state file whose only
            // difference is its clock hashes the same across runs.
            let mut hasher = Sha256::new();
            match std::str::from_utf8(&bytes) {
                Ok(text) => hasher.update(normalize(text, root, home, config).as_bytes()),
                Err(_) => hasher.update(&bytes),
            }
            format!("{:x}", hasher.finalize())
        };
        tree.insert(rel, hash);
    }
    Ok(tree)
}

/// Contents of every text file under the configuration directory. These are
/// the files a reviewer wants to read in a diff, which is why they are stored
/// whole rather than only hashed.
fn capture_text_files(
    dir: &Path,
    sandbox: &Path,
    home: &Path,
    config: &Path,
) -> Result<BTreeMap<String, String>> {
    let mut files = BTreeMap::new();
    for entry in walkdir::WalkDir::new(dir)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let rel = entry
            .path()
            .strip_prefix(dir)?
            .to_string_lossy()
            .into_owned();
        if rel.starts_with(".git/") {
            continue;
        }
        if let Ok(text) = fs::read_to_string(entry.path()) {
            files.insert(rel, normalize(&text, sandbox, home, config));
        }
    }
    Ok(files)
}

fn copy_dir(from: &Path, to: &Path) -> Result<()> {
    fs::create_dir_all(to)?;
    for entry in walkdir::WalkDir::new(from)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
    {
        let rel = entry.path().strip_prefix(from)?;
        if rel.as_os_str().is_empty() {
            continue;
        }
        if rel.starts_with(".git") {
            continue;
        }
        let target = to.join(rel);
        let meta = entry.path().symlink_metadata()?;
        if meta.is_dir() {
            fs::create_dir_all(&target)?;
        } else if meta.is_symlink() {
            copy_symlink(&fs::read_link(entry.path())?, &target)?;
        } else {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

/// Reproduces a symlink in the sandbox rather than following it, because a
/// configuration that symlinks a component directory is a case the corpus has
/// to cover.
///
/// Windows is refused rather than silently skipped. Creating a symlink there
/// needs a privilege the runner does not have by default, and a sandbox whose
/// symlinks quietly became regular files would make the corpus compare
/// something other than the configuration it was given.
#[cfg(unix)]
fn copy_symlink(link: &Path, target: &Path) -> Result<()> {
    std::os::unix::fs::symlink(link, target)
        .with_context(|| format!("linking {} -> {}", target.display(), link.display()))
}

#[cfg(not(unix))]
fn copy_symlink(link: &Path, target: &Path) -> Result<()> {
    bail!(
        "the corpus copies symlinks and this platform does not support creating them \
         unprivileged: {} -> {}",
        target.display(),
        link.display()
    )
}

fn tempdir() -> Result<PathBuf> {
    let base = std::env::temp_dir().join(format!(
        "meowctl-compat-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    fs::create_dir_all(&base)?;
    Ok(base)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn obs(exit: i32, stdout: &str, tree: &[(&str, &str)]) -> Observation {
        Observation {
            exit,
            stdout: stdout.to_owned(),
            tree: tree
                .iter()
                .map(|(p, h)| ((*p).to_owned(), (*h).to_owned()))
                .collect(),
            files: BTreeMap::new(),
        }
    }

    /// A timestamp is what makes two recordings of one command differ, so the
    /// rewrite has to cover the shapes both `state.toml` and the lock files use.
    #[test]
    fn timestamps_are_rewritten() {
        assert_eq!(
            replace_timestamps("started_at = 2026-09-19T17:10:08.103378Z"),
            "started_at = <TIMESTAMP>"
        );
        assert_eq!(
            replace_timestamps("updated-at = \"2026-09-19T17:10:08Z\""),
            "updated-at = \"<TIMESTAMP>\""
        );
    }

    /// A version number is not a date, and rewriting one would hide a real
    /// difference between the two binaries. A bare date is rewritten, because
    /// `meowctl version` prints its build date and that changes on every build.
    #[test]
    fn version_numbers_survive_but_build_dates_do_not() {
        assert_eq!(replace_timestamps("version = 0.3.27"), "version = 0.3.27");
        assert_eq!(replace_timestamps("built 2026-09-19"), "built <TIMESTAMP>");
    }

    /// The commit changes on every commit, so a fixture that carried one would
    /// drift on every commit and stop being trusted.
    #[test]
    fn the_commit_is_rewritten_and_the_target_is_not() {
        assert_eq!(
            replace_build_metadata("meowctl v0.1.0 (darwin/arm64, commit 3c79a43, built x)"),
            "meowctl v0.1.0 (darwin/arm64, commit <BUILD>, built x)"
        );
    }

    /// The config directory lives inside the sandbox, so rewriting the sandbox
    /// first would leave `<SANDBOX>/config` in the output instead of `<CONFIG>`.
    #[test]
    fn the_longest_path_is_rewritten_first() {
        let sandbox = Path::new("/tmp/sb");
        let home = Path::new("/tmp/sb/home");
        let config = Path::new("/tmp/sb/config");
        assert_eq!(
            normalize("/tmp/sb/config/init.star", sandbox, home, config),
            "<CONFIG>/init.star"
        );
        assert_eq!(
            normalize("/tmp/sb/home/.zshrc", sandbox, home, config),
            "<HOME>/.zshrc"
        );
    }

    #[test]
    fn an_identical_run_reports_nothing() {
        let a = obs(0, "hello", &[("config/deps.lock", "abc")]);
        let b = obs(0, "hello", &[("config/deps.lock", "abc")]);
        assert!(diff(&a, &b, true).is_empty());
    }

    /// The failure the corpus exists to catch: a lock file whose content moved.
    #[test]
    fn changed_content_names_the_file() {
        let a = obs(0, "", &[("config/deps.lock", "abc")]);
        let b = obs(0, "", &[("config/deps.lock", "xyz")]);
        let d = diff(&a, &b, false);
        assert_eq!(d.len(), 1, "{d:?}");
        assert!(d[0].contains("config/deps.lock"), "{d:?}");
    }

    #[test]
    fn a_missing_or_extra_path_is_reported() {
        let a = obs(0, "", &[("config/deps.lock", "abc")]);
        let b = obs(0, "", &[("config/other", "abc")]);
        let d = diff(&a, &b, false);
        assert_eq!(d.len(), 2, "{d:?}");
        assert!(
            d.iter().any(|m| m.starts_with("missing from the tree")),
            "{d:?}"
        );
        assert!(
            d.iter().any(|m| m.starts_with("unexpected in the tree")),
            "{d:?}"
        );
    }

    #[test]
    fn an_exit_code_difference_is_reported() {
        let d = diff(&obs(0, "", &[]), &obs(3, "", &[]), false);
        assert_eq!(d.len(), 1);
        assert!(d[0].contains("expected 0, got 3"), "{d:?}");
    }

    /// Rendered output is the deliberate carve-out, so a sink that words a
    /// line differently must not fail the corpus.
    #[test]
    fn rendered_output_is_only_compared_when_it_is_an_interface() {
        let a = obs(0, "Install\n  + base\n", &[]);
        let b = obs(0, "installing\n  base\n", &[]);
        assert!(diff(&a, &b, false).is_empty());
        assert_eq!(diff(&a, &b, true).len(), 1);
    }
}
