//! The command surface, and nothing else.
//!
//! It parses arguments, constructs the effects, calls into
//! [`meowctl_engine`], chooses a sink, and maps an error to an exit code. It
//! holds no domain logic: in `v0.1.0` this layer also owns apply planning,
//! lockfile reading and writing, module syncing and the staleness
//! computation, which is defects #1 and #2.
//!
//! This is the one crate allowed to write to standard output, and it does so
//! through a sink; see [R-CLI-020].

mod cli;
mod error;
mod run;
mod shell;
mod signals;
pub mod templates;
mod update;
mod version;

pub use cli::{Cli, Command, DepCommand, Format, Global, Shell};
pub use error::{CliError, CliResult};
pub use run::{main, run};
pub use shell::snippet as shell_snippet;
pub use version::string as version;
