//! The implementation that spawns the process.

use std::path::PathBuf;
use std::process::Stdio;

use meowctl_common::{Event, Stream};

use crate::{Command, EventSink, ExecError, ExecResult, Executor, Output, find_on_path, lines};

/// Runs commands for real.
#[derive(Debug, Clone, Copy, Default)]
pub struct RealExecutor;

impl RealExecutor {
    /// A new one. It holds no state.
    #[must_use]
    pub const fn new() -> Self {
        RealExecutor
    }
}

impl Executor for RealExecutor {
    fn run(&self, command: &Command, events: EventSink<'_>) -> ExecResult<Output> {
        if find_on_path(&command.program).is_none() {
            return Err(ExecError::NotOnPath {
                program: command.program.clone(),
            });
        }

        let mut process = std::process::Command::new(&command.program);
        process.args(&command.args);
        // The parent environment with the overrides on top, which is what
        // `mergeRunEnv` does: a hook that lost PATH would lose every tool.
        for (key, value) in &command.env {
            process.env(key, value);
        }
        if let Some(dir) = &command.cwd {
            process.current_dir(dir);
        }

        events(Event::ProcessStarted {
            program: command.program.clone(),
            args: command.args.clone(),
        });

        let output = if command.interactive {
            // The sink stands down and the command inherits the terminal.
            // Released in every case below, including the error one.
            events(Event::TerminalRequested);
            let status = process
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status();
            events(Event::TerminalReleased);

            let status = status.map_err(|e| ExecError::Spawn {
                program: command.program.clone(),
                source: e,
            })?;
            Output {
                stdout: String::new(),
                stderr: String::new(),
                exit_code: status.code(),
            }
        } else {
            let captured = process
                .stdin(Stdio::null())
                .output()
                .map_err(|e| ExecError::Spawn {
                    program: command.program.clone(),
                    source: e,
                })?;

            let stdout = String::from_utf8_lossy(&captured.stdout).into_owned();
            let stderr = String::from_utf8_lossy(&captured.stderr).into_owned();
            for line in lines(&stdout) {
                events(Event::ProcessOutput {
                    stream: Stream::Stdout,
                    line: line.to_owned(),
                });
            }
            for line in lines(&stderr) {
                events(Event::ProcessOutput {
                    stream: Stream::Stderr,
                    line: line.to_owned(),
                });
            }
            Output {
                stdout,
                stderr,
                exit_code: captured.status.code(),
            }
        };

        events(Event::ProcessFinished {
            exit_code: output.exit_code,
        });
        Ok(output)
    }

    fn which(&self, program: &str) -> ExecResult<Option<PathBuf>> {
        Ok(find_on_path(program))
    }
}
