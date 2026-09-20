//! Resolving the modules a configuration depends on.
//!
//! Version selection, fetching, integrity, and the cache. What the crate above
//! it sees is a resolved set: which version of each module, where it came
//! from, and the hashes that prove the cache still holds what was fetched.
//!
//! Nothing here reaches the filesystem or the network directly. Both arrive as
//! traits, which is what makes an offline resolution and an in-memory cache
//! ordinary tests rather than integration ones.

pub mod archive;
mod cache;
mod error;
pub mod github;
mod loader;
mod mvs;
pub mod registry;
mod sync;
mod url;
mod version;

pub use cache::{Cache, CacheRecord, Source};
pub use error::{ModuleError, ModuleResult};
pub use loader::{ModuleLoader, Roots};
pub use mvs::{MvsError, Requirement, Requirements, build_list};
pub use sync::{Synced, Syncer, Upgrade, overlay_replaces};
pub use url::{LocalRoot, ModuleUrl};
pub use version::Version;
