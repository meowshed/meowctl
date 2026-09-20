//! The grammar of a `load()` argument.
//!
//! Four schemes, from `internal/starlark/loader/composite.go`, plus a bare
//! relative path. Parsing them is separate from serving them because a
//! diagnostic wants to say "no module named `stdlib`" before anything reaches
//! the network, and because the rules for where `init.star` is implied are
//! worth stating once; see [R-STAR-020] and [R-STAR-021].

use std::fmt;

use crate::ModuleError;

/// Which directory a local load resolves against.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalRoot {
    /// The dotfiles root, written `self//`.
    Dotfiles,
    /// The meowctl configuration directory, written `user://`.
    ///
    /// Also where a bare relative path resolves, which is the one form
    /// `v0.1.0`'s composite loader rejects and the engine handles itself.
    Config,
}

/// A parsed `load()` argument.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModuleUrl {
    /// A file under one of the two local roots.
    Local {
        /// Which root it resolves against.
        root: LocalRoot,
        /// The path under that root, with `/` separators.
        path: String,
    },
    /// A file inside a registry module.
    Registry {
        /// The module's name in the index.
        module: String,
        /// The path inside the module, with `/` separators.
        path: String,
    },
    /// A file in a GitHub repository at a ref.
    GitHub {
        /// Repository owner.
        owner: String,
        /// Repository name.
        repo: String,
        /// The tag or branch, resolved to a commit when fetched.
        reference: String,
        /// The path inside the repository, with `/` separators.
        path: String,
    },
}

impl ModuleUrl {
    /// Parses a `load()` argument.
    ///
    /// # Errors
    ///
    /// [`ModuleError::UnusableUrl`] when no scheme matches and the remainder
    /// is not a usable relative path, or when a scheme is present and its
    /// parts are missing.
    pub fn parse(raw: &str) -> Result<ModuleUrl, ModuleError> {
        if let Some(rest) = raw.strip_prefix("self//") {
            return Ok(ModuleUrl::Local {
                root: LocalRoot::Dotfiles,
                path: local_path(raw, rest)?,
            });
        }
        if let Some(rest) = raw.strip_prefix("user://") {
            return Ok(ModuleUrl::Local {
                root: LocalRoot::Config,
                path: local_path(raw, rest)?,
            });
        }
        if let Some(rest) = raw.strip_prefix("github://") {
            return parse_github(raw, rest);
        }
        if let Some(rest) = raw.strip_prefix('@') {
            return parse_registry(raw, rest);
        }
        Ok(ModuleUrl::Local {
            root: LocalRoot::Config,
            path: local_path(raw, raw)?,
        })
    }

    /// The path inside whatever holds the file.
    #[must_use]
    pub fn path(&self) -> &str {
        match self {
            ModuleUrl::Local { path, .. }
            | ModuleUrl::Registry { path, .. }
            | ModuleUrl::GitHub { path, .. } => path,
        }
    }
}

impl fmt::Display for ModuleUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModuleUrl::Local {
                root: LocalRoot::Dotfiles,
                path,
            } => write!(f, "self//{path}"),
            ModuleUrl::Local {
                root: LocalRoot::Config,
                path,
            } => write!(f, "user://{path}"),
            ModuleUrl::Registry { module, path } => write!(f, "@{module}//{path}"),
            ModuleUrl::GitHub {
                owner,
                repo,
                reference,
                path,
            } => write!(f, "github://{owner}/{repo}@{reference}//{path}"),
        }
    }
}

/// Applies the `init.star` convention to a local path.
fn local_path(raw: &str, rest: &str) -> Result<String, ModuleError> {
    if rest.is_empty() {
        return Err(ModuleError::UnusableUrl {
            url: raw.to_owned(),
            reason: "there is no path after the scheme".to_owned(),
        });
    }
    if rest.starts_with('/') {
        return Err(ModuleError::UnusableUrl {
            url: raw.to_owned(),
            reason: "a load path is relative to its root, so it cannot start with `/`".to_owned(),
        });
    }
    Ok(imply_init(rest))
}

/// Parses `@name` and `@name//path`.
fn parse_registry(raw: &str, rest: &str) -> Result<ModuleUrl, ModuleError> {
    let invalid = |reason: &str| ModuleError::UnusableUrl {
        url: raw.to_owned(),
        reason: reason.to_owned(),
    };
    let Some((module, path)) = rest.split_once("//") else {
        // `@name` alone is the module's root `init.star`; see [R-STAR-021].
        if rest.is_empty() {
            return Err(invalid("there is no module name after `@`"));
        }
        return Ok(ModuleUrl::Registry {
            module: rest.to_owned(),
            path: "init.star".to_owned(),
        });
    };
    if module.is_empty() {
        return Err(invalid("there is no module name before `//`"));
    }
    if path.is_empty() {
        return Err(invalid("there is no path after `//`"));
    }
    Ok(ModuleUrl::Registry {
        module: module.to_owned(),
        path: imply_init(path),
    })
}

/// Parses `github://owner/repo@ref//path`.
fn parse_github(raw: &str, rest: &str) -> Result<ModuleUrl, ModuleError> {
    let invalid = |reason: &str| ModuleError::UnusableUrl {
        url: raw.to_owned(),
        reason: reason.to_owned(),
    };
    let (locator, path) = rest
        .split_once("//")
        .ok_or_else(|| invalid("there is no `//path` after the repository"))?;
    if path.is_empty() {
        return Err(invalid("there is no path after `//`"));
    }
    let (repo_path, reference) = locator
        .rsplit_once('@')
        .ok_or_else(|| invalid("there is no `@ref`; write `github://owner/repo@tag//path`"))?;
    let (owner, repo) = repo_path
        .split_once('/')
        .ok_or_else(|| invalid("the repository is not `owner/repo`"))?;
    if owner.is_empty() || repo.is_empty() || repo.contains('/') || reference.is_empty() {
        return Err(invalid("the repository is not `owner/repo@ref`"));
    }
    Ok(ModuleUrl::GitHub {
        owner: owner.to_owned(),
        repo: repo.to_owned(),
        reference: reference.to_owned(),
        // `v0.1.0` takes a `github://` path exactly as written, with no
        // `init.star` convention, and every URL in the wild is written out.
        path: path.to_owned(),
    })
}

/// Appends `/init.star` where the final segment carries no `.`.
///
/// So `@stdlib//components/apt` is `components/apt/init.star`, which is how
/// every stdlib component is laid out, and `@stdlib//components/apt.star` is
/// that file. They are not the same file; see [R-STAR-021].
fn imply_init(path: &str) -> String {
    let last = path.rsplit('/').next().unwrap_or(path);
    if last.contains('.') {
        path.to_owned()
    } else {
        format!("{}/init.star", path.trim_end_matches('/'))
    }
}
