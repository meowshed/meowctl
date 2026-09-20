//! A configuration written out as a table.
//!
//! Shared by several test binaries, each of which needs some of it. Rust
//! compiles the module separately into each one, so a helper another binary
//! uses looks dead here.
#![allow(dead_code)]
#![allow(clippy::expect_used)]

use std::collections::BTreeMap;

use meowctl_engine::{
    ComponentSource, Declaration, EngineError, EngineResult, Graph, Sources, discover,
};
use meowctl_starlark::{LoadedFile, Loader, Platform, StarlarkError, StarlarkResult};

/// A configuration written out as a table.
#[derive(Default)]
pub struct Config {
    pub declared: Vec<Declaration>,
    files: BTreeMap<String, String>,
}

impl Config {
    pub fn new() -> Self {
        Config::default()
    }

    /// A component the configuration declares, with the source of its file.
    pub fn declaring(mut self, declaration: Declaration, source: &str) -> Self {
        self.files
            .insert(declaration.name.clone(), source.to_owned());
        self.declared.push(declaration);
        self
    }

    /// A component file the configuration does not declare, reachable only
    /// through an `after` list.
    pub fn holding(mut self, name: &str, source: &str) -> Self {
        self.files.insert(name.to_owned(), source.to_owned());
        self
    }
}

impl Sources for Config {
    fn declared(&self) -> EngineResult<Vec<Declaration>> {
        Ok(self.declared.clone())
    }

    fn source(&self, id: &meowctl_common::ComponentId) -> EngineResult<ComponentSource> {
        self.files
            .get(id.as_str())
            .map(|text| ComponentSource {
                text: text.clone(),
                directory: std::path::Path::new(HOME)
                    .join("components")
                    .join(id.logical_name()),
            })
            .ok_or_else(|| EngineError::Configuration {
                path: id.as_str().to_owned(),
                reason: "there is no such file".to_owned(),
            })
    }
}

/// A loader that refuses, because nothing in these tests loads.
#[derive(Debug)]
pub struct NoLoads;

impl Loader for NoLoads {
    fn load(&self, module: &str) -> StarlarkResult<LoadedFile> {
        Err(StarlarkError::Load {
            module: module.to_owned(),
            reason: "this test loads nothing".to_owned(),
        })
    }
}

#[must_use]
pub fn macos() -> Platform {
    Platform {
        os: "macos".to_owned(),
        ..Platform::default()
    }
}

#[must_use]
pub fn linux(distro: &str, like: &str) -> Platform {
    Platform {
        os: "linux".to_owned(),
        distro: distro.to_owned(),
        distro_like: like.to_owned(),
        ..Platform::default()
    }
}

/// The source of a component file that declares itself and nothing else.
#[must_use]
pub fn plain(name: &str) -> String {
    format!("component({name:?})\n")
}

/// Discovers and orders a configuration.
///
/// # Errors
///
/// Whatever discovery or the sort refuses.
pub fn graph_of(config: &Config, platform: &Platform) -> EngineResult<Graph> {
    let loader = NoLoads;
    let discovered = discover(config, &loader, platform)?;
    Graph::build(&discovered)
}

/// An absolute home directory for the platform the test runs on.
///
/// `/home/u` is not absolute on Windows, and [R-CTX-012] refuses a path that
/// is not absolute once `~` expands, so a test that hard-coded a Unix path
/// would fail there for a reason that has nothing to do with the engine.
#[cfg(unix)]
pub const HOME: &str = "/home/u";
#[cfg(not(unix))]
pub const HOME: &str = r"C:\Users\u";

#[cfg(unix)]
pub const STATE_ROOT: &str = "/state";
#[cfg(not(unix))]
pub const STATE_ROOT: &str = r"C:\state";

/// A world a runner can run against: an in-memory filesystem, a scripted
/// executor, and a recorder for the events.
pub struct World {
    pub fs: std::sync::Arc<meowctl_fs::MemFs>,
    pub exec: std::sync::Arc<meowctl_exec::ScriptedExecutor>,
    pub events: std::sync::Arc<std::sync::Mutex<Vec<meowctl_common::Event>>>,
    pub journal: Option<std::sync::Arc<std::sync::Mutex<meowctl_ops::Journal>>>,
}

impl World {
    /// The events, as the sink would have seen them.
    #[must_use]
    pub fn seen(&self) -> Vec<meowctl_common::Event> {
        self.events.lock().expect("the recorder").clone()
    }

    /// Which components finished, and how.
    #[must_use]
    pub fn finished(&self) -> Vec<(String, meowctl_common::Outcome)> {
        self.seen()
            .into_iter()
            .filter_map(|event| match event {
                meowctl_common::Event::ComponentFinished {
                    component, outcome, ..
                } => Some((component.as_str().to_owned(), outcome)),
                _ => None,
            })
            .collect()
    }
}

/// Builds the effects a runner needs.
#[must_use]
pub fn world(
    runs: Vec<meowctl_exec::ScriptedRun>,
    journal: Option<std::path::PathBuf>,
) -> (meowctl_ctx::Effects, World) {
    use meowctl_fs::FileSystem as _;

    let fs = std::sync::Arc::new(meowctl_fs::MemFs::new());
    fs.create_dir_all(std::path::Path::new(HOME)).expect("home");
    let exec = std::sync::Arc::new(meowctl_exec::ScriptedExecutor::new(runs));
    let events: std::sync::Arc<std::sync::Mutex<Vec<meowctl_common::Event>>> =
        std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let recorder = std::sync::Arc::clone(&events);
    let opened = journal.map(|path| {
        std::sync::Arc::new(std::sync::Mutex::new(
            meowctl_ops::Journal::open(path).expect("the journal opens"),
        ))
    });

    let effects = meowctl_ctx::Effects {
        fs: std::sync::Arc::clone(&fs) as std::sync::Arc<dyn meowctl_fs::FileSystem + Send + Sync>,
        exec: std::sync::Arc::clone(&exec)
            as std::sync::Arc<dyn meowctl_exec::Executor + Send + Sync>,
        http: std::sync::Arc::new(meowctl_net::ScriptedHttp::new()),
        interaction: std::sync::Arc::new(std::sync::Mutex::new(meowctl_tui::Always(true))),
        journal: opened.clone(),
        events: std::sync::Arc::new(std::sync::Mutex::new(move |event| {
            recorder.lock().expect("the recorder").push(event);
        })),
    };

    (
        effects,
        World {
            fs,
            exec,
            events,
            journal: opened,
        },
    )
}

/// Settings for a run against that world.
#[must_use]
pub fn settings(platform: &Platform, rollback: bool) -> meowctl_engine::Settings {
    meowctl_engine::Settings {
        home: std::path::PathBuf::from(HOME),
        state_root: std::path::PathBuf::from(STATE_ROOT),
        platform: platform.clone(),
        environment: std::collections::BTreeMap::new(),
        dry_run: false,
        rollback,
    }
}
