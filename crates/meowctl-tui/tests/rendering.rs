//! Rendering a recorded stream, with no terminal and no engine.
//!
//! That is the point of the design: a sink takes events, so a test supplies
//! them; see [R-TUI-080]. Every test here builds a stream, renders it, and
//! asserts on the text.

// `clippy.toml` exempts tests from `expect_used`, but only a function carrying
// `#[test]`. A helper in a test binary is test code by construction.
#![allow(clippy::expect_used)]

use std::sync::{Arc, Mutex};

use meowctl_common::{
    ComponentId, Event, Level, Outcome, Phase, PhaseSet, PlannedStep, SkipReason, Span, Stream,
};
use meowctl_tui::{Caps, ColourDepth, JsonSink, PlainSink, Sink, Theme};

/// A writer a test can read back.
#[derive(Clone, Default)]
struct Captured(Arc<Mutex<Vec<u8>>>);

impl Captured {
    fn text(&self) -> String {
        String::from_utf8_lossy(&self.0.lock().expect("the buffer").clone()).into_owned()
    }
}

impl std::io::Write for Captured {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().expect("the buffer").extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn caps(colour: ColourDepth, unicode: bool) -> Caps {
    Caps {
        tty: false,
        motion: false,
        unicode,
        colour,
        width: Some(80),
        height: Some(24),
    }
}

/// Renders a stream through the plain sink and returns what it wrote.
fn plain(events: &[Event], colour: ColourDepth, unicode: bool) -> String {
    let captured = Captured::default();
    let mut sink = PlainSink::new(
        Box::new(captured.clone()),
        Theme::new(caps(colour, unicode)),
    );
    for event in events {
        sink.handle(event);
    }
    sink.finish();
    captured.text()
}

fn component(name: &str) -> ComponentId {
    name.parse().expect("a component id")
}

/// The stream one small apply produces.
fn an_apply() -> Vec<Event> {
    vec![
        Event::PhaseStarted {
            phase: Phase::Install,
            total: 2,
        },
        Event::ComponentStarted {
            component: component("base"),
            phase: Phase::Install,
        },
        Event::OpApplied {
            kind: "write_file".to_owned(),
            target: "/home/u/.config/base.conf".to_owned(),
        },
        Event::ComponentFinished {
            component: component("base"),
            phase: Phase::Install,
            outcome: Outcome::Succeeded,
        },
        Event::ComponentSkipped {
            component: component("linux-only"),
            phase: Phase::Install,
            reason: SkipReason::PlatformMismatch {
                guard: "linux".to_owned(),
            },
        },
        Event::PhaseFinished {
            phase: Phase::Install,
            failed: 0,
        },
    ]
}

/// [R-TUI-002] column zero is the command talking about itself, two spaces is
/// one item, six is captured detail. Nothing goes deeper.
#[test]
fn output_uses_two_levels_of_indent() {
    let rendered = plain(&an_apply(), ColourDepth::None, true);

    for line in rendered.lines().filter(|l| !l.is_empty()) {
        let indent = line.len() - line.trim_start().len();
        assert!(
            indent == 0 || indent == 2 || indent == 6,
            "unexpected indent {indent} in {line:?}"
        );
    }
}

/// [R-TUI-003] a monochrome terminal loses decoration and never information,
/// so the same run says the same things without colour.
#[test]
fn colour_carries_no_information_of_its_own() {
    let with = plain(&an_apply(), ColourDepth::TrueColour, true);
    let without = plain(&an_apply(), ColourDepth::None, true);

    let stripped: String = strip_escapes(&with);
    assert_eq!(stripped, without);
}

fn strip_escapes(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c != '\u{1b}' {
            out.push(c);
            continue;
        }
        for c in chars.by_ref() {
            if c == 'm' {
                break;
            }
        }
    }
    out
}

/// [R-TUI-043] an ASCII terminal loses shape, not meaning: the same states are
/// still told apart.
#[test]
fn the_ascii_tier_says_the_same_things() {
    let unicode = plain(&an_apply(), ColourDepth::None, true);
    let ascii = plain(&an_apply(), ColourDepth::None, false);

    assert_ne!(unicode, ascii, "the tiers should differ in glyph");
    assert!(ascii.is_ascii(), "{ascii}");
    for line in ascii.lines() {
        assert!(!line.contains('\u{fffd}'), "{line}");
    }
    assert_eq!(unicode.lines().count(), ascii.lines().count());
}

/// [R-ENGINE-052] "120 components skipped" is the line that made `v0.1.0`'s
/// dry run useless, because a user cannot act on it.
#[test]
fn every_skip_says_why() {
    let rendered = plain(
        &[Event::PlanComputed {
            phase_set: PhaseSet::Install,
            steps: vec![
                PlannedStep {
                    phase: Phase::Install,
                    component: component("a"),
                    skipped: Some(SkipReason::AlreadyCompleted),
                },
                PlannedStep {
                    phase: Phase::Install,
                    component: component("b"),
                    skipped: Some(SkipReason::FilteredOut),
                },
                PlannedStep {
                    phase: Phase::Install,
                    component: component("c"),
                    skipped: Some(SkipReason::PlatformMismatch {
                        guard: "linux".to_owned(),
                    }),
                },
            ],
        }],
        ColourDepth::None,
        true,
    );

    assert!(rendered.contains("already done"), "{rendered}");
    assert!(rendered.contains("not in the filter"), "{rendered}");
    assert!(
        rendered.contains("not for this machine (linux)"),
        "{rendered}"
    );
}

/// A plan says how much of it will run, because that is the number a reader
/// wants from a dry run.
#[test]
fn a_plan_says_how_much_of_it_runs() {
    let rendered = plain(
        &[Event::PlanComputed {
            phase_set: PhaseSet::Install,
            steps: vec![
                PlannedStep {
                    phase: Phase::Install,
                    component: component("a"),
                    skipped: None,
                },
                PlannedStep {
                    phase: Phase::Install,
                    component: component("b"),
                    skipped: Some(SkipReason::AlreadyCompleted),
                },
            ],
        }],
        ColourDepth::None,
        true,
    );
    assert!(rendered.contains("1 of 2"), "{rendered}");
}

/// [R-ENGINE-033] a component with nothing to do in a phase has succeeded at
/// it, and saying so on every line would bury what did happen.
#[test]
fn nothing_to_do_says_nothing() {
    let rendered = plain(
        &[Event::ComponentFinished {
            component: component("quiet"),
            phase: Phase::Verify,
            outcome: Outcome::NothingToDo,
        }],
        ColourDepth::None,
        true,
    );
    assert_eq!(rendered, "");
}

/// A failure names the component and what went wrong.
#[test]
fn a_failure_names_the_component_and_the_error() {
    let rendered = plain(
        &[Event::ComponentFinished {
            component: component("broken"),
            phase: Phase::Install,
            outcome: Outcome::Failed {
                error: "brew exited 1".to_owned(),
            },
        }],
        ColourDepth::None,
        true,
    );
    assert!(rendered.contains("broken"), "{rendered}");
    assert!(rendered.contains("brew exited 1"), "{rendered}");
}

/// [R-STAR-040] the span is what `v0.1.0` cannot show, so the sink has to.
#[test]
fn a_diagnostic_shows_where_it_happened() {
    let rendered = plain(
        &[Event::Diagnostic {
            level: Level::Error,
            message: "missing argument for name".to_owned(),
            span: Some(Span {
                file: "init.star".to_owned(),
                line: 12,
                column: 1,
                source_line: Some("component()".to_owned()),
            }),
        }],
        ColourDepth::None,
        true,
    );
    assert!(rendered.contains("init.star:12:1"), "{rendered}");
    assert!(rendered.contains("component()"), "{rendered}");
}

/// [R-TUI-030] one object per event, one per line, in order.
#[test]
fn the_json_sink_writes_one_object_per_line() {
    let captured = Captured::default();
    let mut sink = JsonSink::new(Box::new(captured.clone()));
    for event in an_apply() {
        sink.handle(&event);
    }
    sink.finish();

    let text = captured.text();
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), an_apply().len());

    for line in &lines {
        let parsed: serde_json::Value = serde_json::from_str(line).expect("valid JSON");
        assert!(parsed.get("event").is_some(), "{line}");
    }
    assert!(lines[0].contains("phase_started"), "{}", lines[0]);
}

/// [R-TUI-031] a consumer that had to strip terminal decoration would be
/// parsing a rendering.
#[test]
fn the_json_sink_writes_no_decoration() {
    let captured = Captured::default();
    let mut sink = JsonSink::new(Box::new(captured.clone()));
    for event in an_apply() {
        sink.handle(&event);
    }

    let text = captured.text();
    assert!(!text.contains('\u{1b}'), "{text}");
    for glyph in ["✓", "✗", "▸", "  "] {
        assert!(!text.contains(glyph), "{text} contains {glyph}");
    }
}

/// [R-TUI-012] a sink that dropped a variant would make a command's output
/// depend on where it ran.
#[test]
fn every_event_reaches_the_json_sink() {
    let events = vec![
        Event::TerminalRequested,
        Event::TerminalReleased,
        Event::ProcessOutput {
            stream: Stream::Stderr,
            line: "warning".to_owned(),
        },
        Event::Message {
            level: Level::Debug,
            text: "detail".to_owned(),
        },
    ];

    let captured = Captured::default();
    let mut sink = JsonSink::new(Box::new(captured.clone()));
    for event in &events {
        sink.handle(event);
    }
    assert_eq!(captured.text().lines().count(), events.len());
}

/// [R-TUI-070] a closed pipe is normal when output goes into `head`, and a run
/// must not abort for it.
#[test]
fn a_write_that_fails_does_not_panic() {
    struct Closed;
    impl std::io::Write for Closed {
        fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::from(std::io::ErrorKind::BrokenPipe))
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Err(std::io::Error::from(std::io::ErrorKind::BrokenPipe))
        }
    }

    let mut sink = PlainSink::new(Box::new(Closed), Theme::new(caps(ColourDepth::None, true)));
    for event in an_apply() {
        sink.handle(&event);
    }
    sink.finish();
}

/// The plain sink owns no region, so there is nothing to stand down from.
#[test]
fn the_plain_sink_ignores_the_terminal_hand_off() {
    let rendered = plain(
        &[Event::TerminalRequested, Event::TerminalReleased],
        ColourDepth::None,
        true,
    );
    assert_eq!(rendered, "");
}
