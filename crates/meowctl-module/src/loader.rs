//! Serving a `load()` from wherever the module lives.
//!
//! Four schemes dispatch to three places: a local directory, the registry
//! cache, and a GitHub repository. `internal/starlark/loader/composite.go` is
//! the same dispatch; what is different is that a registry file is checked
//! against what was extracted before it is handed back, so nothing evaluates
//! a file that changed in the cache; see [R-STAR-022].

use std::collections::BTreeMap;
use std::path::PathBuf;

use meowctl_common::paths::is_contained;
use meowctl_fs::FileSystem;
use meowctl_net::Http;
use meowctl_starlark::{LoadedFile, Loader, StarlarkError, StarlarkResult};

use crate::{
    Cache, LocalRoot, ModuleError, ModuleResult, ModuleUrl, Source, cache, github, registry,
};

/// The two directories a local `load()` resolves against.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Roots {
    /// Where `self//` starts: the dotfiles root.
    pub dotfiles: PathBuf,
    /// Where `user://` and a bare relative path start: the config directory.
    pub config: PathBuf,
}

/// Serves `load()` for every scheme.
///
/// Holds the effects as traits rather than performing them, so a test of the
/// dispatch needs neither a network nor a disk, and an `OfflineHttp` proves a
/// resolution reached neither.
pub struct ModuleLoader<'a> {
    fs: &'a dyn FileSystem,
    http: &'a dyn Http,
    cache: Cache,
    roots: Roots,
    index_url: String,
    endpoints: github::GitHubEndpoints,
    /// Module name to the local directory serving it; see [R-MODULE-020].
    replacements: BTreeMap<String, PathBuf>,
    /// Module name to the key it is cached under: a version, or a commit.
    ///
    /// Filled from the lock before evaluation starts. A module absent from it
    /// is a module nothing resolved, which is a failure here rather than a
    /// silent fetch of whatever the registry published today; see
    /// [R-MODULE-050].
    resolved: BTreeMap<String, String>,
}

impl std::fmt::Debug for ModuleLoader<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModuleLoader")
            .field("roots", &self.roots)
            .field("index_url", &self.index_url)
            .field("replacements", &self.replacements)
            .field("resolved", &self.resolved)
            .finish_non_exhaustive()
    }
}

impl<'a> ModuleLoader<'a> {
    /// A loader over a cache and a set of roots.
    #[must_use]
    pub fn new(fs: &'a dyn FileSystem, http: &'a dyn Http, cache: Cache, roots: Roots) -> Self {
        ModuleLoader {
            fs,
            http,
            cache,
            roots,
            index_url: registry::DEFAULT_INDEX_URL.to_owned(),
            endpoints: github::GitHubEndpoints::default(),
            replacements: BTreeMap::new(),
            resolved: BTreeMap::new(),
        }
    }

    /// Points the registry at another index.
    #[must_use]
    pub fn with_index_url(mut self, url: impl Into<String>) -> Self {
        self.index_url = url.into();
        self
    }

    /// Points GitHub at other endpoints, which is what a test does.
    #[must_use]
    pub fn with_endpoints(mut self, endpoints: github::GitHubEndpoints) -> Self {
        self.endpoints = endpoints;
        self
    }

    /// Serves a module from a local directory instead of the registry.
    #[must_use]
    pub fn replacing(mut self, module: impl Into<String>, path: impl Into<PathBuf>) -> Self {
        self.replacements.insert(module.into(), path.into());
        self
    }

    /// Records that a module resolved to a version or a commit.
    #[must_use]
    pub fn resolved(mut self, module: impl Into<String>, key: impl Into<String>) -> Self {
        self.resolved.insert(module.into(), key.into());
        self
    }

    /// Resolves a URL to the bytes of the file it names.
    ///
    /// # Errors
    ///
    /// [`ModuleError`], which names the module and says which step failed.
    pub fn fetch(&self, raw: &str) -> ModuleResult<Vec<u8>> {
        match ModuleUrl::parse(raw)? {
            ModuleUrl::Local { root, path } => self.local(root, &path),
            ModuleUrl::Registry { module, path } => self.registry(&module, &path),
            ModuleUrl::GitHub {
                owner,
                repo,
                reference,
                path,
            } => self.github(&owner, &repo, &reference, &path),
        }
    }

    /// Reads a file under one of the local roots.
    fn local(&self, root: LocalRoot, path: &str) -> ModuleResult<Vec<u8>> {
        let base = match root {
            LocalRoot::Dotfiles => &self.roots.dotfiles,
            LocalRoot::Config => &self.roots.config,
        };
        let target = base.join(path.replace('/', std::path::MAIN_SEPARATOR_STR));
        // The same check the tar entries get, for the same reason: a `load()`
        // argument can come from a module somebody else wrote.
        if !is_contained(base, &target) {
            return Err(ModuleError::UnusableUrl {
                url: path.to_owned(),
                reason: format!("it resolves outside {}", base.display()),
            });
        }
        self.fs.read(&target).map_err(ModuleError::from)
    }

    /// Reads a file out of a registry module, verified.
    fn registry(&self, module: &str, path: &str) -> ModuleResult<Vec<u8>> {
        if let Some(local) = self.replacements.get(module) {
            // A replacement is a checkout somebody is editing. Hashing it
            // would mean re-locking on every save, so it is served as it is;
            // see [R-MODULE-020].
            if !self.fs.exists(local)? {
                return Err(ModuleError::NoSuchReplacement {
                    module: module.to_owned(),
                    path: local.clone(),
                });
            }
            let target = local.join(path.replace('/', std::path::MAIN_SEPARATOR_STR));
            if !is_contained(local, &target) {
                return Err(ModuleError::UnusableUrl {
                    url: path.to_owned(),
                    reason: format!("it resolves outside {}", local.display()),
                });
            }
            return self.fs.read(&target).map_err(ModuleError::from);
        }

        let version = self.version_of(module)?;
        if !self.cache.is_verified(self.fs, module, &version) {
            let index = registry::Index::fetch(self.http, &self.index_url)?;
            let entry = index.entry(module)?;
            if !entry.versions.iter().any(|v| v == &version) {
                return Err(ModuleError::NoSuchVersion {
                    module: module.to_owned(),
                    version: version.clone(),
                    available: entry.versions.clone(),
                });
            }
            let source = cache::Source::from_index(module, &version, entry);
            self.cache.ensure(self.fs, self.http, &source)?;
        }
        self.cache.read_verified(self.fs, module, &version, path)
    }

    /// Reads one file out of a GitHub repository at a ref.
    fn github(
        &self,
        owner: &str,
        repo: &str,
        reference: &str,
        path: &str,
    ) -> ModuleResult<Vec<u8>> {
        let module = format!("{owner}/{repo}");
        let commit = match self.resolved.get(&module) {
            Some(commit) => commit.clone(),
            None => github::resolve_commit(self.http, &self.endpoints, owner, repo, reference)?,
        };
        let source = Source::from_commit(&module, &self.endpoints, owner, repo, &commit);
        self.cache.ensure(self.fs, self.http, &source)?;
        self.cache.read_verified(self.fs, &module, &commit, path)
    }

    /// The version a module resolved to.
    fn version_of(&self, module: &str) -> ModuleResult<String> {
        self.resolved
            .get(module)
            .cloned()
            .ok_or_else(|| ModuleError::NoSuchModule {
                module: module.to_owned(),
            })
    }
}

impl Loader for ModuleLoader<'_> {
    fn load(&self, module: &str) -> StarlarkResult<LoadedFile> {
        let bytes = self.fetch(module).map_err(|e| StarlarkError::Load {
            module: module.to_owned(),
            reason: e.to_string(),
        })?;
        let source = String::from_utf8(bytes).map_err(|_| StarlarkError::Load {
            module: module.to_owned(),
            reason: "the file is not UTF-8, and Starlark source has to be".to_owned(),
        })?;
        Ok(LoadedFile {
            name: module.to_owned(),
            source,
        })
    }
}
