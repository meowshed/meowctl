//! Every file meowctl reads or writes in the configuration directory.
//!
//! One crate owns all of them: the schemas, the parsers, the writers, and the
//! schema versions. In `v0.1.0` they are spread across five packages and two
//! of them — `installed.lock` and `pkgs.lock` — are defined by ad-hoc readers
//! inside the CLI layer, next to the cobra commands.
//!
//! Parity is strictest here. These files are shared between two binaries
//! during the rewrite, and a serializer that reorders keys breaks
//! reproducibility without breaking a test; see [`emit`].

pub mod edit;
pub mod emit;
mod error;
mod installed;
mod layout;
mod lock;
mod modfile;
mod state;

pub use error::{ConfigError, ConfigResult};
pub use installed::{InstalledComponent, InstalledLock};
pub use layout::{LEGACY_ENTRY, Layout};
pub use lock::{GitHubEntry, LockFile, LockMeta, ModuleEntry, PackageEntry};
pub use modfile::{Dep, Modfile, Module, Replace};
pub use state::{CompletedComponent, LastRun, RolledBack, Sentinel};
