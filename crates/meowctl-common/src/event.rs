//! What a command produces, for a sink to render.
//!
//! The vocabulary lives here rather than in `meowctl-engine` so a sink never
//! depends on the engine that emits into it. That is what lets the sinks be
//! tested against a fixture stream with no engine and no terminal; see
//! [R-COMMON-040] and [R-TUI-080].
//!
//! An event carries facts, never decoration. A colour, a glyph, or a rendered
//! line here would make `JsonSink` emit terminal output and give the engine a
//! theme; see [R-COMMON-043].

use serde::{Deserialize, Serialize};

use crate::{ComponentId, Phase, PhaseSet, Span};

/// How important a message is, which decides where a sink puts it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Level {
    /// Detail wanted only under `--verbose`.
    Debug,
    /// Ordinary progress.
    Info,
    /// Something the user should notice but that did not fail the run.
    Warn,
    /// Something failed.
    Error,
}

/// Why a component was not run.
///
/// Every skip carries one. "120 components skipped" is the line that made
/// `v0.1.0`'s dry run useless, because a user cannot act on it; see
/// [R-ENGINE-052].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "reason")]
pub enum SkipReason {
    /// Already recorded as completed for this phase.
    AlreadyCompleted,
    /// Excluded by the component filter on the command line.
    FilteredOut,
    /// Its platform or distribution guard does not match this machine.
    PlatformMismatch {
        /// The guard that did not match.
        guard: String,
    },
}

/// How a component finished.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "outcome")]
pub enum Outcome {
    /// The hook ran and succeeded.
    Succeeded,
    /// The component exports no hook for this phase, which is a success with
    /// nothing to do rather than a skip; see [R-ENGINE-033].
    NothingToDo,
    /// The hook failed.
    Failed {
        /// What went wrong, already rendered by the crate that raised it.
        error: String,
    },
}

/// Which stream a line of subprocess output came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stream {
    /// Standard output.
    Stdout,
    /// Standard error.
    Stderr,
}

/// One step in a plan: what a phase intends to do to a component.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlannedStep {
    /// The phase the step belongs to.
    pub phase: Phase,
    /// The component the step is for.
    pub component: ComponentId,
    /// Set when the step will not run.
    pub skipped: Option<SkipReason>,
}

/// Everything a command can report.
///
/// The engine emits these and holds no renderer; see [R-ENGINE-051].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "event")]
pub enum Event {
    /// A plan was computed. Emitted before execution, and on its own under
    /// `--dry-run`; see [R-ENGINE-050].
    PlanComputed {
        /// The set being run.
        phase_set: PhaseSet,
        /// Every step, in execution order.
        steps: Vec<PlannedStep>,
    },

    /// A phase started.
    PhaseStarted {
        /// The phase.
        phase: Phase,
        /// How many components it will run, skips excluded.
        total: usize,
    },

    /// A phase finished.
    PhaseFinished {
        /// The phase.
        phase: Phase,
        /// How many components failed in it.
        failed: usize,
    },

    /// A component's hook is about to run.
    ComponentStarted {
        /// The component.
        component: ComponentId,
        /// The phase it is running in.
        phase: Phase,
    },

    /// A component was not run.
    ComponentSkipped {
        /// The component.
        component: ComponentId,
        /// The phase it would have run in.
        phase: Phase,
        /// Why it was skipped.
        reason: SkipReason,
    },

    /// A component's hook finished.
    ComponentFinished {
        /// The component.
        component: ComponentId,
        /// The phase it ran in.
        phase: Phase,
        /// How it went.
        outcome: Outcome,
    },

    /// A reversible operation was applied.
    OpApplied {
        /// The operation's kind, as the journal records it.
        kind: String,
        /// What it acted on.
        target: String,
    },

    /// A subprocess started.
    ProcessStarted {
        /// The program.
        program: String,
        /// Its arguments.
        args: Vec<String>,
    },

    /// A line of subprocess output.
    ProcessOutput {
        /// Which stream it came from.
        stream: Stream,
        /// The line, without its trailing newline.
        line: String,
    },

    /// A subprocess exited.
    ProcessFinished {
        /// Its exit code, or `None` when it was killed by a signal.
        exit_code: Option<i32>,
    },

    /// A subprocess needs the real terminal. The sink stands down until
    /// [`Event::TerminalReleased`]; see [R-EXEC-021].
    TerminalRequested,

    /// The terminal is available again. Emitted even when the process failed,
    /// because a renderer that never reclaims it leaves the user without a
    /// cursor; see [R-EXEC-031].
    TerminalReleased,

    /// A problem in a source file, with the position to point at.
    Diagnostic {
        /// How serious it is.
        level: Level,
        /// What is wrong.
        message: String,
        /// Where, when the crate that raised it knows.
        span: Option<Span>,
    },

    /// Text with no structure of its own.
    ///
    /// The only variant carrying free text, and it carries a level so the sink
    /// rather than the call site decides where it lands. Anything a sink needs
    /// to render differently belongs in its own variant; see [R-COMMON-042].
    Message {
        /// How serious it is.
        level: Level,
        /// The text.
        text: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn component(name: &str) -> ComponentId {
        name.parse().unwrap()
    }

    /// [R-COMMON-041] `JsonSink` emits one object per event, so every variant
    /// has to survive a round trip through JSON.
    #[test]
    fn every_event_round_trips_through_json() {
        let events = vec![
            Event::PlanComputed {
                phase_set: PhaseSet::Install,
                steps: vec![
                    PlannedStep {
                        phase: Phase::Install,
                        component: component("base"),
                        skipped: None,
                    },
                    PlannedStep {
                        phase: Phase::Install,
                        component: component("@stdlib//components/zsh"),
                        skipped: Some(SkipReason::AlreadyCompleted),
                    },
                ],
            },
            Event::PhaseStarted {
                phase: Phase::Install,
                total: 2,
            },
            Event::PhaseFinished {
                phase: Phase::Install,
                failed: 0,
            },
            Event::ComponentStarted {
                component: component("base"),
                phase: Phase::Install,
            },
            Event::ComponentSkipped {
                component: component("base"),
                phase: Phase::Install,
                reason: SkipReason::PlatformMismatch {
                    guard: "linux".to_owned(),
                },
            },
            Event::ComponentFinished {
                component: component("base"),
                phase: Phase::Install,
                outcome: Outcome::Failed {
                    error: "boom".to_owned(),
                },
            },
            Event::OpApplied {
                kind: "write_file".to_owned(),
                target: "/home/u/.zshrc".to_owned(),
            },
            Event::ProcessStarted {
                program: "brew".to_owned(),
                args: vec!["list".to_owned()],
            },
            Event::ProcessOutput {
                stream: Stream::Stderr,
                line: "warning".to_owned(),
            },
            Event::ProcessFinished { exit_code: Some(1) },
            Event::TerminalRequested,
            Event::TerminalReleased,
            Event::Diagnostic {
                level: Level::Error,
                message: "missing argument".to_owned(),
                span: Some(Span {
                    file: "init.star".to_owned(),
                    line: 2,
                    column: 1,
                    source_line: Some("component()".to_owned()),
                }),
            },
            Event::Message {
                level: Level::Info,
                text: "hello".to_owned(),
            },
        ];

        for event in events {
            let json = serde_json::to_string(&event).unwrap();
            let back: Event = serde_json::from_str(&json).unwrap();
            assert_eq!(back, event, "{json}");
        }
    }

    /// [R-TUI-030] the JSON form is an interface another program reads, so
    /// every object says which event it is.
    #[test]
    fn the_json_form_is_tagged_by_event_name() {
        let json = serde_json::to_string(&Event::TerminalRequested).unwrap();
        assert_eq!(json, r#"{"event":"terminal_requested"}"#);

        let json = serde_json::to_string(&Event::PhaseStarted {
            phase: Phase::InstallCheck,
            total: 3,
        })
        .unwrap();
        assert!(json.contains(r#""event":"phase_started""#), "{json}");
        assert!(json.contains(r#""phase":"install_check""#), "{json}");
    }

    /// [R-COMMON-042] free text is confined to one variant, which is what
    /// keeps the rest of the stream structured.
    #[test]
    fn only_message_carries_unstructured_text() {
        let json = serde_json::to_string(&Event::Message {
            level: Level::Warn,
            text: "something".to_owned(),
        })
        .unwrap();
        assert!(json.contains(r#""level":"warn""#), "{json}");
    }

    /// [R-COMMON-043] an event that carried a colour or a glyph would make the
    /// JSON sink emit terminal decoration.
    #[test]
    fn no_event_carries_decoration() {
        let json = serde_json::to_string(&Event::ComponentFinished {
            component: component("base"),
            phase: Phase::Install,
            outcome: Outcome::Succeeded,
        })
        .unwrap();
        for decoration in ["\u{1b}", "color", "glyph", "symbol", "width"] {
            assert!(!json.contains(decoration), "{json} carries {decoration}");
        }
    }
}
