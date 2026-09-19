//! Where meowctl keeps things, and how a path from a configuration is resolved.
//!
//! Resolution is here rather than in `meowctl-fs` because a dry run and a real
//! run must resolve a path identically. A filesystem that resolved its own
//! paths could plan against different files than it writes; see [R-FS-002].

use std::path::{Path, PathBuf};

use crate::Error;

/// How the environment is read.
///
/// A trait so tests can supply an environment without setting process-wide
/// variables, which two tests running in parallel cannot do safely.
pub trait Env {
    /// The value of a variable, or `None` when it is unset or empty.
    fn var(&self, key: &str) -> Option<String>;
}

/// The real process environment.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemEnv;

impl Env for SystemEnv {
    fn var(&self, key: &str) -> Option<String> {
        std::env::var(key).ok().filter(|v| !v.is_empty())
    }
}

/// The configuration directory.
///
/// `$MEOWCTL_CONFIG`, then `$XDG_CONFIG_HOME/meowctl`, then
/// `~/.config/meowctl`. `v0.1.0` resolves the last two in
/// `internal/cli/config.go` and does not consult `$MEOWCTL_CONFIG`; that
/// variable is added by [R-COMMON-020] so the compatibility corpus can point
/// two binaries at one configuration without a flag on every call.
///
/// # Errors
///
/// Fails when no override is set and the home directory is unknown.
pub fn config_dir(env: &impl Env) -> Result<PathBuf, Error> {
    if let Some(explicit) = env.var("MEOWCTL_CONFIG") {
        return Ok(PathBuf::from(explicit));
    }
    if let Some(xdg) = env.var("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(xdg).join("meowctl"));
    }
    Ok(home_dir(env)?.join(".config").join("meowctl"))
}

/// The module cache directory.
///
/// `$XDG_CACHE_HOME/meowctl/modules`, then `~/.cache/meowctl/modules`.
/// `v0.1.0` uses the second unconditionally, ignoring `$XDG_CACHE_HOME`, which
/// means a machine that relocates its cache still has meowctl writing to the
/// default. Honouring the variable is a deliberate change; see [R-COMMON-021].
///
/// # Errors
///
/// Fails when no override is set and the home directory is unknown.
pub fn cache_dir(env: &impl Env) -> Result<PathBuf, Error> {
    if let Some(xdg) = env.var("XDG_CACHE_HOME") {
        return Ok(PathBuf::from(xdg).join("meowctl").join("modules"));
    }
    Ok(home_dir(env)?
        .join(".cache")
        .join("meowctl")
        .join("modules"))
}

/// The home directory, from `$HOME`.
///
/// # Errors
///
/// Fails when `$HOME` is unset or empty.
pub fn home_dir(env: &impl Env) -> Result<PathBuf, Error> {
    env.var("HOME")
        .map(PathBuf::from)
        .ok_or(Error::NoHomeDirectory("HOME"))
}

/// Resolves a path written in a configuration to an absolute one.
///
/// Expands a leading `~`, and refuses anything that is not absolute
/// afterwards. `v0.1.0` expands `~` the same way in `expandPath` but accepts a
/// relative path and resolves it against whatever the process working
/// directory happens to be, which is not something a component author can
/// predict. Refusing is a deliberate change; see [R-COMMON-022].
///
/// `~user` is left alone by `v0.1.0` and is therefore refused here, because
/// passing it through produces a literal directory named `~user`.
///
/// # Errors
///
/// Fails when the path is empty, is not absolute after expansion, or needs a
/// home directory that is not set.
pub fn resolve(path: &str, env: &impl Env) -> Result<PathBuf, Error> {
    if path.is_empty() {
        return Err(Error::UnusablePath {
            path: path.to_owned(),
            reason: "the path is empty",
        });
    }

    let expanded = if path == "~" {
        home_dir(env)?
    } else if let Some(rest) = path.strip_prefix("~/") {
        home_dir(env)?.join(rest)
    } else if path.starts_with('~') {
        return Err(Error::UnusablePath {
            path: path.to_owned(),
            reason: "~user is not supported; write the path out",
        });
    } else {
        PathBuf::from(path)
    };

    if !expanded.is_absolute() {
        return Err(Error::UnusablePath {
            path: path.to_owned(),
            reason: "the path is relative, and a hook has no defined working directory",
        });
    }
    Ok(expanded)
}

/// Whether `candidate` stays inside `root` once both are lexically normalised.
///
/// Used where a path comes from outside: a tar entry, a configuration, a lock
/// file. Purely lexical, so it makes no filesystem call and cannot be defeated
/// by a race; a symlink inside `root` that points out of it is the caller's
/// problem to check, if it has one.
#[must_use]
pub fn is_contained(root: &Path, candidate: &Path) -> bool {
    let normal = |p: &Path| -> Option<PathBuf> {
        let mut out = PathBuf::new();
        for part in p.components() {
            match part {
                std::path::Component::ParentDir => {
                    if !out.pop() {
                        return None;
                    }
                }
                std::path::Component::CurDir => {}
                other => out.push(other),
            }
        }
        Some(out)
    };
    match (normal(root), normal(candidate)) {
        (Some(root), Some(candidate)) => candidate.starts_with(&root),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    /// An absolute home directory for the platform the test runs on.
    ///
    /// `/home/u` is not absolute on Windows, which has no drive letter in it,
    /// so a test that hard-codes a Unix path checks nothing there and fails
    /// for the wrong reason.
    #[cfg(unix)]
    const HOME: &str = "/home/u";
    #[cfg(not(unix))]
    const HOME: &str = r"C:\Users\u";

    /// Joins onto [`HOME`] with the platform's separator.
    fn under_home(rest: &str) -> PathBuf {
        PathBuf::from(HOME).join(rest)
    }

    struct FakeEnv(HashMap<&'static str, &'static str>);

    impl FakeEnv {
        fn new(pairs: &[(&'static str, &'static str)]) -> Self {
            FakeEnv(pairs.iter().copied().collect())
        }
    }

    impl Env for FakeEnv {
        fn var(&self, key: &str) -> Option<String> {
            self.0
                .get(key)
                .map(|v| (*v).to_owned())
                .filter(|v| !v.is_empty())
        }
    }

    /// [R-COMMON-020] the order decides which configuration a command reads,
    /// and getting it wrong sends a run at the wrong machine's dotfiles.
    #[test]
    fn the_config_directory_prefers_the_explicit_override() {
        let env = FakeEnv::new(&[
            ("MEOWCTL_CONFIG", "/explicit"),
            ("XDG_CONFIG_HOME", "/xdg"),
            ("HOME", HOME),
        ]);
        assert_eq!(config_dir(&env).unwrap(), PathBuf::from("/explicit"));

        let env = FakeEnv::new(&[("XDG_CONFIG_HOME", "/xdg"), ("HOME", HOME)]);
        assert_eq!(
            config_dir(&env).unwrap(),
            PathBuf::from("/xdg").join("meowctl")
        );

        let env = FakeEnv::new(&[("HOME", HOME)]);
        assert_eq!(config_dir(&env).unwrap(), under_home(".config/meowctl"));
    }

    /// An empty variable is unset. A shell that exports `XDG_CONFIG_HOME=`
    /// would otherwise send the configuration directory to `/meowctl`.
    #[test]
    fn an_empty_variable_counts_as_unset() {
        let env = FakeEnv::new(&[("XDG_CONFIG_HOME", ""), ("HOME", HOME)]);
        assert_eq!(config_dir(&env).unwrap(), under_home(".config/meowctl"));
    }

    #[test]
    fn a_missing_home_is_an_error_rather_than_a_guess() {
        let env = FakeEnv::new(&[]);
        assert!(matches!(
            config_dir(&env),
            Err(Error::NoHomeDirectory("HOME"))
        ));
    }

    /// [R-COMMON-021] `v0.1.0` ignores `XDG_CACHE_HOME`; honouring it is the
    /// deliberate change, and the fallback still matches.
    #[test]
    fn the_cache_directory_honours_xdg_and_falls_back_as_v0_1_0_does() {
        let env = FakeEnv::new(&[("XDG_CACHE_HOME", "/c"), ("HOME", HOME)]);
        assert_eq!(
            cache_dir(&env).unwrap(),
            PathBuf::from("/c").join("meowctl").join("modules")
        );

        let env = FakeEnv::new(&[("HOME", HOME)]);
        assert_eq!(
            cache_dir(&env).unwrap(),
            under_home(".cache/meowctl/modules")
        );
    }

    /// [R-COMMON-022] `~` expansion matches `expandPath`, including `~` alone.
    #[test]
    fn a_leading_tilde_expands() {
        let env = FakeEnv::new(&[("HOME", HOME)]);
        assert_eq!(resolve("~", &env).unwrap(), PathBuf::from(HOME));
        assert_eq!(
            resolve("~/.config/nvim", &env).unwrap(),
            under_home(".config/nvim")
        );
    }

    /// [R-COMMON-022] a relative path resolves against a working directory a
    /// hook author cannot predict, so it is refused rather than guessed at.
    #[test]
    fn a_relative_path_is_refused_with_the_reason() {
        let env = FakeEnv::new(&[("HOME", HOME)]);
        let err = resolve("config/nvim", &env).unwrap_err();
        assert!(err.to_string().contains("relative"), "{err}");
    }

    /// `v0.1.0` passes `~user` through, which creates a directory literally
    /// named `~user`. Refusing says so instead.
    #[test]
    fn tilde_user_is_refused_rather_than_taken_literally() {
        let env = FakeEnv::new(&[("HOME", HOME)]);
        let err = resolve("~other/file", &env).unwrap_err();
        assert!(err.to_string().contains("~user"), "{err}");
    }

    #[test]
    fn an_empty_path_is_refused() {
        let env = FakeEnv::new(&[("HOME", HOME)]);
        assert!(resolve("", &env).is_err());
    }

    /// [R-MODULE-033] a tarball is remote input, and an entry that climbs out
    /// of the module root is how it writes somewhere it was not invited.
    #[test]
    fn containment_is_checked_lexically() {
        let root = Path::new("/cache/mod");
        assert!(is_contained(
            root,
            Path::new("/cache/mod/components/x.star")
        ));
        assert!(is_contained(root, Path::new("/cache/mod/a/../b")));
        assert!(!is_contained(root, Path::new("/cache/other")));
        assert!(!is_contained(
            root,
            Path::new("/cache/mod/../../etc/passwd")
        ));
    }
}
