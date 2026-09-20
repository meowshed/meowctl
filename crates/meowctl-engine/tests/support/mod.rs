//! A configuration written out as a table.
//!
//! Shared by several test binaries, each of which needs some of it. Rust
//! compiles the module separately into each one, so a helper another binary
//! uses looks dead here.
#![allow(dead_code)]
#![allow(clippy::expect_used)]

use std::collections::BTreeMap;

use meowctl_engine::{Declaration, EngineError, EngineResult, Graph, Sources, discover};
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

    fn source(&self, id: &meowctl_common::ComponentId) -> EngineResult<String> {
        self.files
            .get(id.as_str())
            .cloned()
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
