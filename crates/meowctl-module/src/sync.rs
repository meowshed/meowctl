//! Resolving a manifest into a lock.
//!
//! `deps.mod` says what a configuration wants. `deps.lock` says what it got:
//! a version for each module, where that version came from, and the hash that
//! proves it. Syncing is the step between them, and it is the only step that
//! chooses a version.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use meowctl_config::{Dep, LockFile, LockMeta, ModuleEntry, Replace};
use meowctl_fs::FileSystem;
use meowctl_net::Http;
use meowctl_starlark::{Evaluator, NoLoader, Platform};

use crate::{
    Cache, ModuleError, ModuleResult, MvsError, Requirement, Requirements, Source, build_list,
    github, registry,
};

/// The manifest file a `MODULE.meow` is, inside a module.
const MANIFEST: &str = "MODULE.meow";

/// Which modules to resolve again rather than take from the lock.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Upgrade {
    /// Keep every locked version; resolve only what is missing.
    ///
    /// The ordinary `meowctl dep sync`, and what [R-MODULE-050] describes.
    #[default]
    Nothing,
    /// Clear these modules' locked versions and resolve them again, leaving
    /// the rest of the lock where it is; see [R-MODULE-053].
    Named(BTreeSet<String>),
    /// Ignore the lock entirely.
    Everything,
}

impl Upgrade {
    /// Whether this module's locked version is still binding.
    fn keeps(&self, module: &str) -> bool {
        match self {
            Upgrade::Nothing => true,
            Upgrade::Named(names) => !names.contains(module),
            Upgrade::Everything => false,
        }
    }
}

/// What a sync produced.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Synced {
    /// The lock to write.
    pub lock: LockFile,
    /// Which version each registry module resolved to, for a caller that
    /// reports what changed.
    pub resolved: BTreeMap<String, String>,
    /// Which modules are served from a local directory, and from where.
    pub replaced: BTreeMap<String, PathBuf>,
}

/// Resolves manifests into locks.
pub struct Syncer<'a> {
    fs: &'a dyn FileSystem,
    http: &'a dyn Http,
    cache: Cache,
    index_url: String,
    endpoints: github::GitHubEndpoints,
    generated_by: String,
    now: String,
}

impl std::fmt::Debug for Syncer<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Syncer")
            .field("index_url", &self.index_url)
            .field("generated_by", &self.generated_by)
            .finish_non_exhaustive()
    }
}

impl<'a> Syncer<'a> {
    /// A syncer writing locks that say `generated_by` wrote them at `now`.
    ///
    /// Both arrive as values rather than being read here, because a component
    /// that asks the clock cannot be tested and a crate below the binary does
    /// not know the binary's version; see [R-CONFIG-022].
    #[must_use]
    pub fn new(
        fs: &'a dyn FileSystem,
        http: &'a dyn Http,
        cache: Cache,
        generated_by: impl Into<String>,
        now: impl Into<String>,
    ) -> Self {
        Syncer {
            fs,
            http,
            cache,
            index_url: registry::DEFAULT_INDEX_URL.to_owned(),
            endpoints: github::GitHubEndpoints::default(),
            generated_by: generated_by.into(),
            now: now.into(),
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

    /// Resolves one manifest into one lock.
    ///
    /// `previous` is the lock as it stands. A module in it keeps its version
    /// unless `upgrade` says otherwise, because resolution that ran again on
    /// every sync would move with whatever the registry published today; see
    /// [R-MODULE-050].
    ///
    /// # Errors
    ///
    /// [`ModuleError`], naming the module and the step that failed.
    pub fn sync(
        &self,
        deps: &[Dep],
        replaces: &[Replace],
        previous: &LockFile,
        upgrade: &Upgrade,
    ) -> ModuleResult<Synced> {
        let replaced_by: BTreeMap<&str, &Replace> =
            replaces.iter().map(|r| (r.name.as_str(), r)).collect();

        let mut out = Synced {
            lock: LockFile {
                meta: LockMeta {
                    generated_by: self.generated_by.clone(),
                    updated_at: self.now.clone(),
                },
                // Packages and GitHub file pins belong to other commands and
                // are carried through untouched: a `dep sync` that dropped
                // them would undo an apply.
                packages: previous.packages.clone(),
                github: previous.github.clone(),
                modules: BTreeMap::new(),
            },
            ..Synced::default()
        };

        // The index is fetched only when something needs it, so a manifest
        // whose every dependency is replaced or on GitHub syncs offline.
        let mut index: Option<registry::Index> = None;

        for dep in deps {
            if let Some(replace) = replaced_by.get(dep.name.as_str()) {
                if !replace.path.is_empty() {
                    self.lock_replacement(&mut out, &dep.name, &replace.path)?;
                    continue;
                }
                if !replace.source.is_empty() {
                    // A fork is fetched like any other remote module and
                    // verified like one: the directive says where to look, not
                    // that the code is trusted; see [R-MODULE-021].
                    self.lock_github(&mut out, &dep.name, &replace.source)?;
                    continue;
                }
            }
            if !dep.source.is_empty() {
                self.lock_github(&mut out, &dep.name, &dep.source)?;
                continue;
            }

            let index = match &index {
                Some(index) => index,
                None => index.insert(registry::Index::fetch(self.http, &self.index_url)?),
            };
            self.lock_registry(&mut out, dep, index, previous, upgrade)?;
        }

        Ok(out)
    }

    /// Records a module served from a local directory.
    fn lock_replacement(&self, out: &mut Synced, module: &str, path: &str) -> ModuleResult<()> {
        let path = PathBuf::from(path);
        // A typo here must not quietly fetch upstream, which is the opposite
        // of what the directive asked for; see [R-MODULE-064].
        if !self.fs.exists(&path)? {
            return Err(ModuleError::NoSuchReplacement {
                module: module.to_owned(),
                path,
            });
        }
        out.lock.modules.insert(
            module.to_owned(),
            ModuleEntry {
                replaced: true,
                path: path.display().to_string(),
                ..ModuleEntry::default()
            },
        );
        out.replaced.insert(module.to_owned(), path);
        Ok(())
    }

    /// Records a module fetched from a GitHub repository at a ref.
    fn lock_github(&self, out: &mut Synced, module: &str, source: &str) -> ModuleResult<()> {
        let parsed =
            source
                .parse::<meowctl_common::ModuleRef>()
                .map_err(|_| ModuleError::UnusableUrl {
                    url: source.to_owned(),
                    reason: "a source is `github:owner/repo@tag-or-branch`".to_owned(),
                })?;
        let meowctl_common::ModuleRef::GitHub {
            owner,
            repo,
            reference,
        } = parsed
        else {
            return Err(ModuleError::UnusableUrl {
                url: source.to_owned(),
                reason: "a source names a GitHub repository".to_owned(),
            });
        };

        let commit = github::resolve_commit(self.http, &self.endpoints, &owner, &repo, &reference)?;
        let source_url = Source::from_commit(module, &self.endpoints, &owner, &repo, &commit);
        let dir = self.cache.ensure(self.fs, self.http, &source_url)?;
        let integrity = self
            .cache
            .tarball_hash(self.fs, module, &commit)
            .map_or_else(String::new, |i| i.to_string());

        out.lock.modules.insert(
            module.to_owned(),
            ModuleEntry {
                source: source_url.url().to_owned(),
                integrity,
                commit_sha: commit.clone(),
                ..ModuleEntry::default()
            },
        );
        out.resolved.insert(module.to_owned(), commit);

        // An aggregate module brings its dependencies with it, and they are
        // read from its own manifest; see [R-MODULE-012].
        for transitive in self.manifest_deps(module, &dir)? {
            if out.lock.modules.contains_key(&transitive.name) || transitive.source.is_empty() {
                continue;
            }
            self.lock_github(out, &transitive.name, &transitive.source)?;
        }
        Ok(())
    }

    /// Records a registry module and everything it selects.
    fn lock_registry(
        &self,
        out: &mut Synced,
        dep: &Dep,
        index: &registry::Index,
        previous: &LockFile,
        upgrade: &Upgrade,
    ) -> ModuleResult<()> {
        let locked = previous
            .modules
            .get(&dep.name)
            .filter(|entry| !entry.replaced && !entry.version.is_empty())
            .filter(|_| upgrade.keeps(&dep.name))
            .map(|entry| entry.version.clone());

        let wanted = match locked {
            Some(version) => version,
            None => self.requested_version(&dep.name, &dep.version, index)?,
        };

        let reqs = ManifestRequirements {
            syncer: self,
            index,
        };
        let root = Requirement::new(dep.name.clone(), wanted);
        for selected in build_list(&root, &reqs)? {
            if out.lock.modules.contains_key(&selected.name) {
                continue;
            }
            let entry = index.entry(&selected.name)?;
            let source = Source::from_index(&selected.name, &selected.version, entry);
            self.cache.ensure(self.fs, self.http, &source)?;
            let integrity = self
                .cache
                .tarball_hash(self.fs, &selected.name, &selected.version)
                .map_or_else(String::new, |i| i.to_string());
            out.lock.modules.insert(
                selected.name.clone(),
                ModuleEntry {
                    version: selected.version.clone(),
                    source: source.url().to_owned(),
                    integrity,
                    ..ModuleEntry::default()
                },
            );
            out.resolved.insert(selected.name, selected.version);
        }
        Ok(())
    }

    /// The version a manifest asked for, resolving `latest` and the empty
    /// string to what the index published most recently.
    fn requested_version(
        &self,
        module: &str,
        declared: &str,
        index: &registry::Index,
    ) -> ModuleResult<String> {
        let entry = index.entry(module)?;
        if declared.is_empty() || declared == "latest" {
            return entry.latest(module).map(str::to_owned);
        }
        if !entry.versions.iter().any(|v| v == declared) {
            return Err(ModuleError::NoSuchVersion {
                module: module.to_owned(),
                version: declared.to_owned(),
                available: entry.versions.clone(),
            });
        }
        Ok(declared.to_owned())
    }

    /// The dependencies a module's own `MODULE.meow` declares.
    ///
    /// One evaluator reads it, the same one that reads `init.star` and
    /// `deps.mod`. `v0.1.0` has three readers for these three files and the
    /// third ignores `module()`'s arguments entirely, which is how a manifest
    /// ended up able to carry a field the others reject.
    fn manifest_deps(
        &self,
        module: &str,
        dir: &std::path::Path,
    ) -> ModuleResult<Vec<meowctl_starlark::DepDecl>> {
        let path = dir.join(MANIFEST);
        // A module with no manifest has no dependencies, which `v0.1.0` also
        // treats as ordinary rather than as a failure.
        let Ok(bytes) = self.fs.read(&path) else {
            return Ok(Vec::new());
        };
        let source = String::from_utf8(bytes).map_err(|_| ModuleError::Archive {
            module: module.to_owned(),
            reason: format!("{MANIFEST} is not UTF-8"),
        })?;
        let loader = NoLoader;
        let evaluated = Evaluator::new(Platform::current(), &loader)
            .evaluate(MANIFEST, &source)
            .map_err(|e| ModuleError::Archive {
                module: module.to_owned(),
                reason: format!("{MANIFEST} does not evaluate: {e}"),
            })?;
        Ok(evaluated.declarations.deps)
    }
}

/// Reads a module's requirements out of its own manifest.
struct ManifestRequirements<'a> {
    syncer: &'a Syncer<'a>,
    index: &'a registry::Index,
}

impl Requirements for ManifestRequirements<'_> {
    fn required(&self, module: &Requirement) -> Result<Vec<Requirement>, MvsError> {
        let failed = |reason: String| MvsError::Requirements {
            module: module.name.clone(),
            version: module.version.clone(),
            reason,
        };
        let entry = self
            .index
            .entry(&module.name)
            .map_err(|e| failed(e.to_string()))?;
        let source = Source::from_index(&module.name, &module.version, entry);
        let dir = self
            .syncer
            .cache
            .ensure(self.syncer.fs, self.syncer.http, &source)
            .map_err(|e| failed(e.to_string()))?;
        let deps = self
            .syncer
            .manifest_deps(&module.name, &dir)
            .map_err(|e| failed(e.to_string()))?;
        Ok(deps
            .into_iter()
            // A `dep(source = ...)` inside a registry module names a GitHub
            // repository, which MVS has no version to compare. `v0.1.0`'s
            // `parseModuleMeow` drops it the same way, and the shared manifest
            // is where such a dependency has to be declared.
            .filter(|dep| dep.source.is_empty())
            .map(|dep| Requirement::new(dep.name, dep.version))
            .collect())
    }
}

/// Merges two manifests' `replace()` directives, local winning.
///
/// A machine-local override exists precisely to differ from what the
/// configuration commits, so it wins wherever both name a module; see
/// [R-MODULE-022].
#[must_use]
pub fn overlay_replaces(shared: &[Replace], local: &[Replace]) -> Vec<Replace> {
    let mut merged: Vec<Replace> = shared.to_vec();
    for replace in local {
        match merged.iter_mut().find(|held| held.name == replace.name) {
            Some(held) => *held = replace.clone(),
            None => merged.push(replace.clone()),
        }
    }
    merged
}
