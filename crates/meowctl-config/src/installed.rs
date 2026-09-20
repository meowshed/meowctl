//! `installed.lock`, in both of its schema versions.

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use meowctl_fs::FileSystem;

use crate::emit::{self, Value};
use crate::{ConfigError, ConfigResult};

/// The version this build writes.
const CURRENT_SCHEMA: i64 = 2;

/// One installed component, with the module it came from.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstalledComponent {
    /// The component, as the configuration names it.
    pub name: String,
    /// The fingerprint of the module it was installed from.
    ///
    /// Empty for a component that belongs to no module, and for every entry
    /// read from a version 1 file.
    #[serde(default)]
    pub version: String,
}

/// What is installed.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstalledLock {
    /// Which shape the file is in.
    #[serde(default)]
    pub schema_version: i64,
    /// The version 1 form: names and nothing else.
    ///
    /// Still read, because a user who installed an earlier build has it on
    /// disk; never written; see [R-CONFIG-031].
    #[serde(default)]
    pub components: Vec<String>,
    /// The version 2 form.
    #[serde(default)]
    pub installed: Vec<InstalledComponent>,
}

impl InstalledLock {
    /// Reads the file, in whichever shape it is in.
    ///
    /// # Errors
    ///
    /// [`ConfigError::Malformed`] when it is not valid TOML, or
    /// [`ConfigError::NewerSchema`] when a newer build wrote it.
    pub fn read(fs: &dyn FileSystem, path: &Path) -> ConfigResult<InstalledLock> {
        let bytes = match fs.read(path) {
            Ok(b) => b,
            Err(meowctl_fs::FsError::NotFound { .. }) => return Ok(InstalledLock::default()),
            Err(e) => return Err(e.into()),
        };
        let text = String::from_utf8_lossy(&bytes);
        let lock: InstalledLock = toml::from_str(&text).map_err(|e| ConfigError::Malformed {
            path: path.to_path_buf(),
            reason: e.to_string(),
        })?;

        if lock.schema_version > CURRENT_SCHEMA {
            return Err(ConfigError::NewerSchema {
                path: path.to_path_buf(),
                found: lock.schema_version,
                understood: CURRENT_SCHEMA,
            });
        }
        Ok(lock)
    }

    /// Each component's recorded module fingerprint.
    ///
    /// Reads both shapes, with a version 1 entry treated as unknown, which is
    /// what `installedLock.versionMap` does; see [R-CONFIG-031].
    #[must_use]
    pub fn fingerprints(&self) -> BTreeMap<String, String> {
        if !self.installed.is_empty() {
            return self
                .installed
                .iter()
                .map(|c| (c.name.clone(), c.version.clone()))
                .collect();
        }
        self.components
            .iter()
            .map(|name| (name.clone(), String::new()))
            .collect()
    }

    /// The installed component names, from either shape.
    #[must_use]
    pub fn names(&self) -> Vec<String> {
        if self.installed.is_empty() {
            return self.components.clone();
        }
        self.installed.iter().map(|c| c.name.clone()).collect()
    }

    /// Writes the current shape, sorted by name.
    ///
    /// Sorted so the file does not churn between runs and show a diff every
    /// time somebody applies; see [R-CONFIG-032].
    ///
    /// # Errors
    ///
    /// Whatever the filesystem fails with.
    pub fn write(
        fs: &dyn FileSystem,
        path: &Path,
        components: &[InstalledComponent],
    ) -> ConfigResult<()> {
        let mut sorted = components.to_vec();
        sorted.sort_by(|a, b| a.name.cmp(&b.name));

        let document = Value::Table(vec![
            ("schema_version".to_owned(), Value::Integer(CURRENT_SCHEMA)),
            (
                "installed".to_owned(),
                Value::ArrayOfTables(
                    sorted
                        .iter()
                        .map(|c| {
                            Value::table(vec![
                                ("name", Some(Value::String(c.name.clone()))),
                                ("version", emit::Value::optional_string(&c.version)),
                            ])
                        })
                        .collect(),
                ),
            ),
        ]);

        fs.write(path, emit::to_string(&document).as_bytes())?;
        Ok(())
    }
}
