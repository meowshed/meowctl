//! The implementation that runs only what a read-only phase asked for.
//!
//! A dry run cannot refuse every subprocess: an `install_check` hook that
//! cannot ask `brew list` what is installed reports a plan built on nothing.
//! It cannot allow every subprocess either. The phase decides, and it decides
//! when this is constructed rather than on every call, so nothing below
//! `meowctl-cli` branches on a dry run; see [R-EXEC-011] and [R-CTX-014].

use std::path::PathBuf;

use meowctl_common::{Event, Level, Phase};

use crate::{Command, EventSink, ExecResult, Executor, Output};

/// Runs a command when the phase is read-only, and reports it otherwise.
#[derive(Debug)]
pub struct DryRunExecutor {
    /// Performs the commands this one does allow.
    underlying: Box<dyn Executor + Send + Sync>,
    /// The phase this executor was built for.
    phase: Phase,
}

impl DryRunExecutor {
    /// Wraps an executor for one phase.
    ///
    /// When the phase is read-only every command runs, because reading is what
    /// those phases do. Otherwise nothing runs.
    #[must_use]
    pub fn new(underlying: Box<dyn Executor + Send + Sync>, phase: Phase) -> Self {
        DryRunExecutor { underlying, phase }
    }

    /// Whether commands issued in this phase actually run.
    #[must_use]
    pub const fn runs_commands(&self) -> bool {
        self.phase.is_read_only()
    }
}

impl Executor for DryRunExecutor {
    fn run(&self, command: &Command, events: EventSink<'_>) -> ExecResult<Output> {
        if self.runs_commands() {
            return self.underlying.run(command, events);
        }

        // Reported rather than silently skipped: a plan that does not say what
        // it would have run is a plan nobody can check.
        let mut line = command.program.clone();
        for arg in &command.args {
            line.push(' ');
            line.push_str(arg);
        }
        events(Event::Message {
            level: Level::Info,
            text: format!("would run: {line}"),
        });

        // Zero, because a hook that branches on failure would otherwise take
        // the failure path for every command in a dry run and plan a repair
        // that is not needed.
        Ok(Output {
            stdout: String::new(),
            stderr: String::new(),
            exit_code: Some(0),
        })
    }

    fn which(&self, program: &str) -> ExecResult<Option<PathBuf>> {
        // Asking reads nothing and changes nothing, and a hook that cannot ask
        // whether a tool exists takes the wrong branch.
        self.underlying.which(program)
    }
}
