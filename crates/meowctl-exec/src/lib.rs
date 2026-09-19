//! Every subprocess meowctl spawns, behind one trait.
//!
//! The trait exists for the same reason [`meowctl_fs::FileSystem`] does: a dry
//! run is an implementation, and a test supplies a scripted one rather than
//! needing `brew` installed.
//!
//! It also owns the hand-off of the terminal to an interactive command. In
//! `v0.1.0` that is `SuspendOutput func() (resume func())`, a callback threaded
//! from the renderer into `ctx.Capabilities` so a Starlark builtin can reach
//! back and stand the renderer down. Here it is two events, and nothing in this
//! crate knows a renderer exists; see [R-EXEC-022].
//!
//! [`meowctl_fs::FileSystem`]: https://docs.rs/meowctl-fs

mod dry_run;
mod error;
mod real;
mod script;

use std::collections::BTreeMap;
use std::fmt::Debug;
use std::path::{Path, PathBuf};

use meowctl_common::Event;

pub use dry_run::DryRunExecutor;
pub use error::{ExecError, ExecResult};
pub use real::RealExecutor;
pub use script::{ScriptedExecutor, ScriptedRun};

/// What to run.
///
/// Built from a program and an argument list, never from a shell string, so a
/// component's arguments cannot be reinterpreted by a shell its author did not
/// know was there; see [R-EXEC-002]. A component that wants shell syntax
/// invokes a shell explicitly, which is visible in the command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    /// The program to run, as written. Resolved on `PATH` by the executor.
    pub program: String,
    /// Its arguments, already split.
    pub args: Vec<String>,
    /// Environment overrides, merged over the parent environment.
    pub env: BTreeMap<String, String>,
    /// The working directory, or the parent's when absent.
    pub cwd: Option<PathBuf>,
    /// Whether the command needs the real terminal.
    ///
    /// An interactive command takes the terminal for its duration, which the
    /// sink is told about through [`Event::TerminalRequested`]. A command that
    /// is not interactive has its output captured and rendered under its
    /// component; see [R-EXEC-023].
    pub interactive: bool,
}

impl Command {
    /// A command with no arguments, no overrides, and no terminal.
    #[must_use]
    pub fn new(program: impl Into<String>) -> Self {
        Command {
            program: program.into(),
            args: Vec::new(),
            env: BTreeMap::new(),
            cwd: None,
            interactive: false,
        }
    }

    /// Adds arguments.
    #[must_use]
    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.args.extend(args.into_iter().map(Into::into));
        self
    }

    /// Sets an environment override.
    #[must_use]
    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    /// Sets the working directory.
    #[must_use]
    pub fn cwd(mut self, dir: impl Into<PathBuf>) -> Self {
        self.cwd = Some(dir.into());
        self
    }

    /// Marks the command as needing the real terminal.
    #[must_use]
    pub fn interactive(mut self) -> Self {
        self.interactive = true;
        self
    }
}

/// What a command produced.
///
/// The three fields are what `ctx.run` returns in `v0.1.0`, under these names,
/// and component code branches on `exit_code`; see [R-EXEC-003].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Output {
    /// Everything the command wrote to standard output.
    pub stdout: String,
    /// Everything it wrote to standard error.
    pub stderr: String,
    /// Its exit code, or `None` when a signal killed it.
    pub exit_code: Option<i32>,
}

impl Output {
    /// Whether the command exited zero.
    ///
    /// A non-zero exit is a result, not an error: an interrogation hook asking
    /// `brew list` about a package it does not have gets a 1 and wants to read
    /// it; see [R-EXEC-004].
    #[must_use]
    pub fn succeeded(&self) -> bool {
        self.exit_code == Some(0)
    }
}

/// Where an event goes on its way to a sink.
///
/// A function rather than a trait because it has one method and every caller
/// has a closure to hand. The engine passes one that forwards to its stream;
/// a test passes one that collects.
pub type EventSink<'a> = &'a mut dyn FnMut(Event);

/// Running commands.
pub trait Executor: Debug {
    /// Runs a command to completion.
    ///
    /// Emits [`Event::ProcessStarted`], a [`Event::ProcessOutput`] per line,
    /// and [`Event::ProcessFinished`]. An interactive command is wrapped in
    /// [`Event::TerminalRequested`] and [`Event::TerminalReleased`], and the
    /// release is emitted even when the command fails, because a renderer that
    /// never reclaims the terminal leaves the user without a cursor; see
    /// [R-EXEC-031].
    ///
    /// # Errors
    ///
    /// [`ExecError::NotOnPath`] when the program does not exist, or
    /// [`ExecError::Spawn`] when it could not be started.
    fn run(&self, command: &Command, events: EventSink<'_>) -> ExecResult<Output>;

    /// Resolves a program name on `PATH`.
    ///
    /// Returns `Ok(None)` when nothing matches, because asking whether a tool
    /// is installed is a question rather than an assertion; see [R-EXEC-032].
    ///
    /// # Errors
    ///
    /// [`ExecError::Spawn`] when `PATH` cannot be read.
    fn which(&self, program: &str) -> ExecResult<Option<PathBuf>>;
}

impl<T: Executor + ?Sized> Executor for std::sync::Arc<T> {
    fn run(&self, command: &Command, events: EventSink<'_>) -> ExecResult<Output> {
        (**self).run(command, events)
    }
    fn which(&self, program: &str) -> ExecResult<Option<PathBuf>> {
        (**self).which(program)
    }
}

/// Splits captured output into the lines a sink renders.
///
/// A trailing newline does not produce an empty last line, because a command
/// that ends its output properly should not look as though it printed a blank.
pub(crate) fn lines(text: &str) -> impl Iterator<Item = &str> {
    text.strip_suffix('\n')
        .unwrap_or(text)
        .split('\n')
        .filter(|l| !text.is_empty() || !l.is_empty())
}

/// Looks a program up on `PATH`, for implementations that need it.
pub(crate) fn find_on_path(program: &str) -> Option<PathBuf> {
    if program.contains(std::path::MAIN_SEPARATOR) {
        let path = Path::new(program);
        return path.is_file().then(|| path.to_path_buf());
    }
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path).find_map(|dir| {
        let candidate = dir.join(program);
        candidate.is_file().then_some(candidate)
    })
}
