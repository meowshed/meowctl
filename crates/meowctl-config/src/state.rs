//! `state.toml`, the sentinel that records what a run did.

use std::path::Path;

use serde::{Deserialize, Serialize};

use meowctl_fs::FileSystem;

use crate::emit::{self, Value};
use crate::{ConfigError, ConfigResult};

/// The version this build writes.
const CURRENT_SCHEMA: i64 = 1;

/// How a rollback went.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RolledBack {
    /// No rollback was attempted.
    #[default]
    #[serde(rename = "")]
    None,
    /// Every inverse applied.
    Ok,
    /// Some applied and some did not.
    Partial,
    /// None applied.
    Failed,
}

impl RolledBack {
    /// The string the file stores; see [R-CONFIG-044].
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            RolledBack::None => "",
            RolledBack::Ok => "ok",
            RolledBack::Partial => "partial",
            RolledBack::Failed => "failed",
        }
    }
}

/// What the most recent run was.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LastRun {
    /// The phase set it ran.
    #[serde(default)]
    pub phase_set: String,
    /// When it started, in UTC.
    #[serde(default)]
    pub started_at: Option<toml::value::Datetime>,
    /// Whether it finished.
    #[serde(default)]
    pub completed: bool,
    /// How a rollback went, when there was one.
    #[serde(default)]
    pub rolled_back: RolledBack,
}

/// One component that finished a phase.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompletedComponent {
    /// The phase.
    pub phase: String,
    /// The component.
    pub component: String,
    /// When, in UTC.
    #[serde(default)]
    pub completed_at: Option<toml::value::Datetime>,
}

/// The sentinel file.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sentinel {
    /// Which shape the file is in.
    #[serde(default)]
    pub schema_version: i64,
    /// The most recent run.
    #[serde(default)]
    pub last_run: LastRun,
    /// Every component that finished a phase, appended as it did.
    #[serde(default)]
    pub completed_components: Vec<CompletedComponent>,
    /// Where the configuration was bootstrapped from.
    #[serde(default)]
    pub repo_url: String,
}

impl Sentinel {
    /// Reads the sentinel.
    ///
    /// # Errors
    ///
    /// [`ConfigError::Malformed`] when it is not valid TOML, or
    /// [`ConfigError::NewerSchema`] when a newer build wrote it.
    pub fn read(fs: &dyn FileSystem, path: &Path) -> ConfigResult<Sentinel> {
        let bytes = match fs.read(path) {
            Ok(b) => b,
            Err(meowctl_fs::FsError::NotFound { .. }) => {
                return Ok(Sentinel {
                    schema_version: CURRENT_SCHEMA,
                    ..Sentinel::default()
                });
            }
            Err(e) => return Err(e.into()),
        };
        let text = String::from_utf8_lossy(&bytes);
        let sentinel: Sentinel = toml::from_str(&text).map_err(|e| ConfigError::Malformed {
            path: path.to_path_buf(),
            reason: e.to_string(),
        })?;

        // `v0.1.0` ignores this field, so an older binary rewrites a newer
        // file and drops what it did not understand. Refusing is visible and
        // recoverable; the data loss is neither; see [R-CONFIG-041].
        if sentinel.schema_version > CURRENT_SCHEMA {
            return Err(ConfigError::NewerSchema {
                path: path.to_path_buf(),
                found: sentinel.schema_version,
                understood: CURRENT_SCHEMA,
            });
        }
        Ok(sentinel)
    }

    /// Whether a component has already finished a phase.
    #[must_use]
    pub fn is_completed(&self, phase: &str, component: &str) -> bool {
        self.completed_components
            .iter()
            .any(|c| c.phase == phase && c.component == component)
    }

    /// Forgets everything recorded for a component, so the next run redoes it.
    ///
    /// Used when the module a component came from has changed; see
    /// [R-ENGINE-043].
    pub fn forget(&mut self, component: &str) {
        self.completed_components
            .retain(|c| c.component != component);
    }

    /// Writes the sentinel in the layout `v0.1.0` emits.
    ///
    /// # Errors
    ///
    /// Whatever the filesystem fails with.
    pub fn write(&self, fs: &dyn FileSystem, path: &Path) -> ConfigResult<()> {
        fs.write(path, emit::to_string(&self.to_value()).as_bytes())?;
        Ok(())
    }

    fn to_value(&self) -> Value {
        let datetime =
            |d: &Option<toml::value::Datetime>| d.as_ref().map(|d| Value::Datetime(d.to_string()));

        Value::table(vec![
            (
                "schema_version",
                Some(Value::Integer(if self.schema_version == 0 {
                    CURRENT_SCHEMA
                } else {
                    self.schema_version
                })),
            ),
            // Omitted when empty, as `repo_url,omitempty` is.
            ("repo_url", emit::Value::optional_string(&self.repo_url)),
            (
                "last_run",
                Some(Value::table(vec![
                    (
                        "phase_set",
                        Some(Value::String(self.last_run.phase_set.clone())),
                    ),
                    ("started_at", datetime(&self.last_run.started_at)),
                    ("completed", Some(Value::Boolean(self.last_run.completed))),
                    (
                        "rolled_back",
                        Some(Value::String(self.last_run.rolled_back.as_str().to_owned())),
                    ),
                ])),
            ),
            (
                "completed_components",
                Some(Value::ArrayOfTables(
                    self.completed_components
                        .iter()
                        .map(|c| {
                            Value::table(vec![
                                ("phase", Some(Value::String(c.phase.clone()))),
                                ("component", Some(Value::String(c.component.clone()))),
                                ("completed_at", datetime(&c.completed_at)),
                            ])
                        })
                        .collect(),
                )),
            ),
        ])
    }
}
