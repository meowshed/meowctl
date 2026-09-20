//! Where events go.
//!
//! Three sinks over one stream. `PlainSink` writes one line per event and
//! moves no cursor; `JsonSink` writes one object per line and no decoration at
//! all. The live renderer arrives with issue 26.

use std::io::Write;

use meowctl_common::{Event, Level, Outcome, SkipReason};

use crate::theme::{Role, Theme};

/// Something that renders events.
///
/// A sink renders every event it is given. One that silently dropped a variant
/// would make a command's output depend on where it ran; see [R-TUI-012].
pub trait Sink {
    /// Renders one event.
    ///
    /// A write that fails is not propagated: a closed pipe is normal when
    /// output goes into `head`, and a run must not abort for it; see
    /// [R-TUI-070].
    fn handle(&mut self, event: &Event);

    /// Flushes anything held back.
    fn finish(&mut self) {}
}

/// One line per event, no cursor movement.
///
/// What a pipe, a CI log, a dumb terminal, and `MEOWCTL_OUTPUT=plain` all get.
pub struct PlainSink {
    out: Box<dyn Write>,
    theme: Theme,
    /// Whether anything has been written under the current component, so
    /// captured output knows whether it is the first line.
    in_component: bool,
}

impl std::fmt::Debug for PlainSink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlainSink")
            .field("theme", &self.theme)
            .finish_non_exhaustive()
    }
}

impl PlainSink {
    /// A sink writing to `out`.
    #[must_use]
    pub fn new(out: Box<dyn Write>, theme: Theme) -> Self {
        PlainSink {
            out,
            theme,
            in_component: false,
        }
    }

    /// Writes a line, ignoring a failure.
    fn line(&mut self, text: &str) {
        let _ = writeln!(self.out, "{text}");
    }

    /// A line at column zero: the command talking about itself.
    fn heading(&mut self, text: &str) {
        let painted = self.theme.paint(Role::Accent, text);
        self.line(&painted);
    }

    /// A line at one indent: one item.
    fn item(&mut self, role: Role, text: &str) {
        let symbol = self.theme.paint(role, self.theme.symbol(role));
        self.line(&format!("  {symbol} {text}"));
    }
}

impl Sink for PlainSink {
    fn handle(&mut self, event: &Event) {
        match event {
            Event::PlanComputed { phase_set, steps } => {
                let running = steps.iter().filter(|s| s.skipped.is_none()).count();
                self.heading(&format!(
                    "{phase_set}: {running} of {} component(s)",
                    steps.len()
                ));
                for step in steps {
                    match &step.skipped {
                        // Every skip says why: a count alone is not something
                        // a user can act on; see [R-ENGINE-052].
                        Some(reason) => {
                            let separator = self.theme.symbols().separator;
                            self.item(
                                Role::Muted,
                                &format!("{} {separator} {}", step.component, describe(reason)),
                            );
                        }
                        None => self.item(Role::Success, &step.component.to_string()),
                    }
                }
            }

            Event::PhaseStarted { phase, total } => {
                self.heading(&format!("{phase} ({total})"));
            }

            Event::PhaseFinished { phase, failed } => {
                if *failed > 0 {
                    self.item(Role::Failure, &format!("{phase}: {failed} failed"));
                }
            }

            Event::ComponentStarted { component, .. } => {
                self.in_component = true;
                self.item(Role::Info, &component.to_string());
            }

            Event::ComponentSkipped {
                component, reason, ..
            } => {
                let separator = self.theme.symbols().separator;
                self.item(
                    Role::Muted,
                    &format!("{component} {separator} {}", describe(reason)),
                );
            }

            Event::ComponentFinished {
                component, outcome, ..
            } => {
                self.in_component = false;
                match outcome {
                    Outcome::Succeeded => self.item(Role::Success, &component.to_string()),
                    // Nothing to do is a success with nothing to say, so it
                    // says nothing; see [R-ENGINE-033].
                    Outcome::NothingToDo => {}
                    Outcome::Failed { error } => {
                        self.item(Role::Failure, &format!("{component}: {error}"));
                    }
                }
            }

            Event::OpApplied { kind, target } => {
                let text = self
                    .theme
                    .paint(Role::Muted, &format!("      {kind} {target}"));
                self.line(&text);
            }

            Event::ProcessStarted { program, args } => {
                let text = self.theme.paint(
                    Role::Muted,
                    &format!("      $ {program} {}", args.join(" ")),
                );
                self.line(&text);
            }

            // Captured output sits under its item and dimmed, which is the
            // second level of indent and the last one; see [R-TUI-002].
            Event::ProcessOutput { line, .. } => {
                let text = self.theme.paint(Role::Muted, &format!("      {line}"));
                self.line(&text);
            }

            Event::ProcessFinished { exit_code } => {
                if !matches!(exit_code, Some(0)) {
                    let code = exit_code.map_or_else(|| "a signal".to_owned(), |c| c.to_string());
                    self.item(Role::Warning, &format!("exited {code}"));
                }
            }

            // Nothing to stand down from: this sink owns no region.
            Event::TerminalRequested | Event::TerminalReleased => {}

            Event::Diagnostic {
                level,
                message,
                span,
            } => {
                let role = role_of(*level);
                match span {
                    Some(span) => {
                        self.item(role, &format!("{span}: {message}"));
                        if let Some(source) = &span.source_line {
                            let text = self.theme.paint(Role::Muted, &format!("      {source}"));
                            self.line(&text);
                        }
                    }
                    None => self.item(role, message),
                }
            }

            // Shell code goes to the shell, not to a transcript. `meowctl
            // shell` writes it to stdout itself; a sink rendering a run puts
            // it where a reader can see what a component contributed; see
            // [R-COMMON-044].
            Event::ShellLine { line } => {
                let text = self.theme.paint(Role::Muted, &format!("      {line}"));
                self.line(&text);
            }

            Event::PathPrepended { directory } => {
                let text = self
                    .theme
                    .paint(Role::Muted, &format!("      PATH += {directory}"));
                self.line(&text);
            }

            Event::Message { level, text } => match level {
                // Verbose detail is held back rather than shown, because a
                // plain log that says everything says nothing.
                Level::Debug => {}
                level => self.item(role_of(*level), text),
            },
        }
    }

    fn finish(&mut self) {
        let _ = self.out.flush();
    }
}

/// One JSON object per event, one per line.
///
/// The interface another program reads. `doctor --json` stops being a
/// hand-written special case and every command gains a machine-readable form;
/// see [R-TUI-030] and [R-TUI-032].
pub struct JsonSink {
    out: Box<dyn Write>,
}

impl std::fmt::Debug for JsonSink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JsonSink").finish_non_exhaustive()
    }
}

impl JsonSink {
    /// A sink writing to `out`.
    #[must_use]
    pub fn new(out: Box<dyn Write>) -> Self {
        JsonSink { out }
    }
}

impl Sink for JsonSink {
    fn handle(&mut self, event: &Event) {
        // No colour, no glyph, no padding: a consumer that had to strip
        // terminal decoration would be parsing a rendering; see [R-TUI-031].
        let Ok(line) = serde_json::to_string(event) else {
            return;
        };
        let _ = writeln!(self.out, "{line}");
    }

    fn finish(&mut self) {
        let _ = self.out.flush();
    }
}

/// Why a component was skipped, in words a user can act on.
pub(crate) fn describe(reason: &SkipReason) -> String {
    match reason {
        SkipReason::AlreadyCompleted => "already done".to_owned(),
        SkipReason::FilteredOut => "not in the filter".to_owned(),
        SkipReason::PlatformMismatch { guard } => format!("not for this machine ({guard})"),
    }
}

/// Which role a message level renders as.
pub(crate) const fn role_of(level: Level) -> Role {
    match level {
        Level::Debug | Level::Info => Role::Info,
        Level::Warn => Role::Warning,
        Level::Error => Role::Failure,
    }
}
