//! The command tree.
//!
//! Exactly what `newRootCmd` registers, with the flag names and shorthands
//! `internal/cli/` uses: a component written against one binary's command
//! line has to work against the other's; see [R-CLI-001].

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

/// Dotfiles and dev environment manager powered by Starlark.
#[derive(Debug, Parser)]
#[command(name = "meowctl", version = crate::version::STRING, disable_help_subcommand = true)]
pub struct Cli {
    /// What to do.
    #[command(subcommand)]
    pub command: Command,

    /// The flags every command carries.
    #[command(flatten)]
    pub global: Global,
}

/// The flags every command carries; see [R-CLI-004] and [R-CLI-006].
#[derive(Debug, Args, Clone, Default)]
pub struct Global {
    /// Config directory (default: ~/.config/meowctl)
    #[arg(long, global = true, value_name = "DIR")]
    pub config: Option<PathBuf>,

    /// Enable verbose output
    #[arg(long, short = 'v', global = true)]
    pub verbose: bool,

    /// How to render what happens
    #[arg(long, global = true, value_name = "FORMAT")]
    pub format: Option<Format>,
}

/// What a command's output looks like.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Format {
    /// Whatever the terminal takes.
    Auto,
    /// One line per event, no cursor movement.
    Plain,
    /// One JSON object per event; see [R-TUI-032].
    Json,
}

/// Which shell to emit integration code for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Shell {
    /// Bash.
    Bash,
    /// Zsh.
    Zsh,
    /// Fish, whose syntax differs enough to need its own snippet.
    Fish,
    /// Anything POSIX, for a shell not named above.
    Posix,
}

impl std::fmt::Display for Shell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Shell::Bash => "bash",
            Shell::Zsh => "zsh",
            Shell::Fish => "fish",
            Shell::Posix => "posix",
        })
    }
}

/// Every command.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Scaffold a new config directory or bootstrap from an existing dotfiles repo
    Init {
        /// A public dotfiles repository to bootstrap from
        repo_url: Option<String>,
        /// Overwrite existing config
        #[arg(long, short = 'f')]
        force: bool,
    },

    /// Reconcile installed components with the declared config
    Apply {
        /// Only these components, and what they depend on
        components: Vec<String>,
        /// Print what would be done without executing
        #[arg(long, short = 'n')]
        dry_run: bool,
        /// Force re-install even if already completed
        #[arg(long, short = 'f')]
        force: bool,
        /// Skip automatic rollback on failure
        #[arg(long)]
        no_rollback: bool,
        /// Ignore lock file when resolving modules
        #[arg(long)]
        ignore_lock: bool,
    },

    /// Add component(s) to local.star and install them
    Add {
        /// The components to add
        #[arg(required = true)]
        components: Vec<String>,
        /// Show what would change without writing files
        #[arg(long, short = 'n')]
        dry_run: bool,
        /// Force re-install even if already completed
        #[arg(long, short = 'f')]
        force: bool,
        /// Skip automatic rollback on failure
        #[arg(long)]
        no_rollback: bool,
        /// Ignore lock file when resolving modules
        #[arg(long)]
        ignore_lock: bool,
    },

    /// Remove component(s) from local.star and uninstall them
    Remove {
        /// The components to remove
        #[arg(required = true)]
        components: Vec<String>,
        /// Show what would change without writing files
        #[arg(long, short = 'n')]
        dry_run: bool,
        /// Skip automatic rollback on failure
        #[arg(long)]
        no_rollback: bool,
    },

    /// Run the upgrade phase set for all (or specified) components
    Upgrade {
        /// Only these components
        components: Vec<String>,
        /// Print what would be done without executing
        #[arg(long, short = 'n')]
        dry_run: bool,
        /// Force re-install even if already completed
        #[arg(long, short = 'f')]
        force: bool,
        /// Skip automatic rollback on failure
        #[arg(long)]
        no_rollback: bool,
        /// Ignore lock file when resolving modules
        #[arg(long)]
        ignore_lock: bool,
    },

    /// Verify the current environment against the dotfiles config
    Verify {
        /// Only these components
        components: Vec<String>,
        /// Print what would be done without executing
        #[arg(long, short = 'n')]
        dry_run: bool,
        /// Skip automatic rollback on failure
        #[arg(long)]
        no_rollback: bool,
    },

    /// Pull the latest dotfiles from the remote repo and apply changes
    Update {
        /// Show what would change without writing files
        #[arg(long, short = 'n')]
        dry_run: bool,
        /// Skip automatic rollback on failure
        #[arg(long)]
        no_rollback: bool,
        /// Auto-apply changes without prompting
        #[arg(long, short = 'y')]
        yes: bool,
    },

    /// Show the last-run metadata and completed components
    Status {
        /// List every completed component, not just the most recent
        #[arg(long)]
        all: bool,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Check the meowctl configuration and environment for problems
    Doctor {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Validate component .star files in a directory tree
    Check {
        /// The directory to walk
        dir: PathBuf,
    },

    /// Run runtime hooks for the given phase and emit shell code to stdout
    Hook {
        /// The phase to run
        phase: String,
    },

    /// Emit shell integration code for the given shell
    Shell {
        /// The shell to emit for
        shell: Shell,
    },

    /// Manage module dependencies
    Dep {
        /// Which of the group.
        #[command(subcommand)]
        command: DepCommand,
    },

    /// Update meowctl to the latest release
    SelfUpdate,

    /// Print version information
    Version,

    /// Generate a completion script
    Completions {
        /// The shell to generate for
        shell: clap_complete::Shell,
    },
}

/// The `dep` group; see [R-CLI-002].
#[derive(Debug, Subcommand)]
pub enum DepCommand {
    /// List declared dependencies
    List,

    /// Add a dependency to deps.mod (or deps.local.mod with --local)
    Add {
        /// The module
        name: String,
        /// Registry version to pin
        #[arg(long)]
        version: Option<String>,
        /// GitHub source ref (e.g. github:owner/repo@v1.0.0)
        #[arg(long)]
        source: Option<String>,
        /// Add to deps.local.mod instead of deps.mod
        #[arg(long)]
        local: bool,
    },

    /// Remove a dependency from deps.mod or deps.local.mod
    Remove {
        /// The module
        name: String,
        /// Remove from deps.local.mod instead of deps.mod
        #[arg(long)]
        local: bool,
    },

    /// Upgrade registry deps to latest versions; re-resolve non-SHA GitHub refs
    Upgrade {
        /// Only these modules
        modules: Vec<String>,
        /// Show what would change without writing files
        #[arg(long, short = 'n')]
        dry_run: bool,
    },

    /// Sync deps.mod and deps.local.mod to their lock files
    Sync {
        /// Show what would change without writing files
        #[arg(long, short = 'n')]
        dry_run: bool,
    },

    /// Remove deps not referenced in star files; warn on unknown refs
    Tidy {
        /// Show what would change without writing files
        #[arg(long, short = 'n')]
        dry_run: bool,
    },
}

impl Command {
    /// Whether this command writes nothing, from its own `--dry-run`.
    ///
    /// Read here rather than threaded down as a boolean: it chooses which
    /// `FileSystem` and `Executor` get constructed, and nothing below this
    /// crate sees the flag; see [R-CLI-011].
    #[must_use]
    pub const fn dry_run(&self) -> bool {
        match self {
            Command::Apply { dry_run, .. }
            | Command::Add { dry_run, .. }
            | Command::Remove { dry_run, .. }
            | Command::Upgrade { dry_run, .. }
            | Command::Verify { dry_run, .. }
            | Command::Update { dry_run, .. } => *dry_run,
            Command::Dep { command } => command.dry_run(),
            // A command that changes nothing has nothing to preview.
            _ => false,
        }
    }

    /// Whether this command's stdout is read by another program.
    ///
    /// `shell` emits code the shell evaluates, and `hook` emits what a
    /// component contributed to it; see [R-CLI-021].
    #[must_use]
    pub const fn stdout_is_an_interface(&self) -> bool {
        matches!(self, Command::Shell { .. } | Command::Hook { .. })
    }

    /// Whether the command asked for JSON through its own older flag.
    ///
    /// `doctor` and `status` had `--json` before `--format json` existed, and
    /// it stays accepted; see [R-CLI-006].
    #[must_use]
    pub const fn wants_json(&self) -> bool {
        match self {
            Command::Status { json, .. } | Command::Doctor { json } => *json,
            _ => false,
        }
    }
}

impl DepCommand {
    /// Whether this subcommand writes nothing.
    #[must_use]
    pub const fn dry_run(&self) -> bool {
        match self {
            DepCommand::Upgrade { dry_run, .. }
            | DepCommand::Sync { dry_run }
            | DepCommand::Tidy { dry_run } => *dry_run,
            DepCommand::List | DepCommand::Add { .. } | DepCommand::Remove { .. } => false,
        }
    }
}
