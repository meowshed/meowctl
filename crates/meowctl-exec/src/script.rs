//! The implementation a test supplies.
//!
//! `v0.1.0` has `RunFunc`, an injection point on `ctx` alone, which is the
//! only seam its tests have. This is the equivalent with the rest of the
//! workspace behind it, and it fails a test that runs a command the test did
//! not expect rather than returning an empty result; see [R-EXEC-012].

use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Mutex;

use meowctl_common::{Event, Stream};

use crate::{Command, EventSink, ExecError, ExecResult, Executor, Output, lines};

/// One scripted answer.
#[derive(Debug, Clone)]
pub struct ScriptedRun {
    /// The command line this answers, as `program arg arg`.
    pub command: String,
    /// What it produces.
    pub output: Output,
}

impl ScriptedRun {
    /// A command that succeeds with this on standard output.
    #[must_use]
    pub fn ok(command: impl Into<String>, stdout: impl Into<String>) -> Self {
        ScriptedRun {
            command: command.into(),
            output: Output {
                stdout: stdout.into(),
                stderr: String::new(),
                exit_code: Some(0),
            },
        }
    }

    /// A run that died without an exit code, as a process killed by a signal
    /// does.
    #[must_use]
    pub fn killed(command: impl Into<String>) -> Self {
        ScriptedRun {
            command: command.into(),
            output: Output {
                stdout: String::new(),
                stderr: String::new(),
                exit_code: None,
            },
        }
    }

    /// A command that exits non-zero.
    #[must_use]
    pub fn fails(command: impl Into<String>, code: i32, stderr: impl Into<String>) -> Self {
        ScriptedRun {
            command: command.into(),
            output: Output {
                stdout: String::new(),
                stderr: stderr.into(),
                exit_code: Some(code),
            },
        }
    }
}

/// Answers a fixed list of commands, in order, and refuses anything else.
#[derive(Debug)]
pub struct ScriptedExecutor {
    expected: Mutex<VecDeque<ScriptedRun>>,
    on_path: Vec<String>,
    ran: Mutex<Vec<String>>,
}

impl ScriptedExecutor {
    /// An executor that answers these commands, in this order.
    #[must_use]
    pub fn new(runs: impl IntoIterator<Item = ScriptedRun>) -> Self {
        ScriptedExecutor {
            expected: Mutex::new(runs.into_iter().collect()),
            on_path: Vec::new(),
            ran: Mutex::new(Vec::new()),
        }
    }

    /// Declares which programs `which` finds.
    #[must_use]
    pub fn with_path(mut self, programs: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.on_path = programs.into_iter().map(Into::into).collect();
        self
    }

    /// Every command that was run, in order.
    ///
    /// # Panics
    ///
    /// If the lock is poisoned.
    #[must_use]
    pub fn ran(&self) -> Vec<String> {
        self.ran
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    /// Whether every scripted command was used.
    ///
    /// A test that scripts three commands and runs two has a gap it probably
    /// did not intend.
    ///
    /// # Panics
    ///
    /// If the lock is poisoned.
    #[must_use]
    pub fn is_exhausted(&self) -> bool {
        self.expected
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_empty()
    }
}

fn command_line(command: &Command) -> String {
    let mut line = command.program.clone();
    for arg in &command.args {
        line.push(' ');
        line.push_str(arg);
    }
    line
}

impl Executor for ScriptedExecutor {
    fn run(&self, command: &Command, events: EventSink<'_>) -> ExecResult<Output> {
        let line = command_line(command);
        self.ran
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(line.clone());

        let next = self
            .expected
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .pop_front();

        let Some(scripted) = next else {
            return Err(ExecError::Unscripted { command: line });
        };
        if scripted.command != line {
            return Err(ExecError::Unscripted {
                command: format!("{line} (expected `{}`)", scripted.command),
            });
        }

        events(Event::ProcessStarted {
            program: command.program.clone(),
            args: command.args.clone(),
        });
        for text in lines(&scripted.output.stdout) {
            events(Event::ProcessOutput {
                stream: Stream::Stdout,
                line: text.to_owned(),
            });
        }
        for text in lines(&scripted.output.stderr) {
            events(Event::ProcessOutput {
                stream: Stream::Stderr,
                line: text.to_owned(),
            });
        }
        events(Event::ProcessFinished {
            exit_code: scripted.output.exit_code,
        });
        Ok(scripted.output)
    }

    fn which(&self, program: &str) -> ExecResult<Option<PathBuf>> {
        Ok(self
            .on_path
            .iter()
            .any(|p| p == program)
            .then(|| PathBuf::from("/usr/bin").join(program)))
    }
}
