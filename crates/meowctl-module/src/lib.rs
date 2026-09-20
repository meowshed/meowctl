//! Resolving the modules a configuration depends on.
//!
//! Version selection lives here, and so will fetching, integrity, and the
//! cache. What the crate above it sees is a resolved set: which version of
//! each module, where it came from, and the hashes that prove it is what the
//! lock file says.

mod mvs;
mod version;

pub use mvs::{MvsError, Requirement, Requirements, build_list};
pub use version::Version;
