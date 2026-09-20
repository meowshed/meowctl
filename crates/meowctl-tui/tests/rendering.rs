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
use meowctl_tui::{Caps, ColourDepth, JsonSink, PlainSink, ShellSink, Sink, Theme};

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
///
/// Also [R-TUI-010] and [R-TUI-032]: the stream is one stream, and each of
/// the sinks that renders a run consumes all of it. `--format json` selects
/// this one, and a command whose events it did not carry would be a command
/// nothing could script.
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

/// [R-TUI-012] a shell evaluates whatever the sink writes, so it writes the
/// line and not a rendering of it.
#[test]
fn the_shell_sink_writes_the_line_and_nothing_around_it() {
    let out = Captured::default();
    let mut sink = ShellSink::new(Box::new(out.clone()));

    sink.handle(&Event::ShellLine {
        line: "export EDITOR=nvim".to_owned(),
    });
    sink.finish();

    assert_eq!(out.text(), "export EDITOR=nvim\n");
}

/// [R-TUI-012] every other event is dropped, because a shell would evaluate
/// a heading as a command.
#[test]
fn the_shell_sink_writes_nothing_for_any_other_event() {
    let out = Captured::default();
    let mut sink = ShellSink::new(Box::new(out.clone()));

    for event in [
        Event::Message {
            level: Level::Info,
            text: "installing".to_owned(),
        },
        Event::Message {
            level: Level::Warn,
            text: "a warning".to_owned(),
        },
        Event::PhaseStarted {
            phase: Phase::Shell,
            total: 3,
        },
        Event::ComponentStarted {
            component: component("git"),
            phase: Phase::Shell,
        },
        Event::PathPrepended {
            directory: "/opt/homebrew/bin".to_owned(),
        },
    ] {
        sink.handle(&event);
    }
    sink.finish();

    assert_eq!(out.text(), "");
}

/// [R-TUI-013] `--format json` on `hook` is the event stream, not shell code:
/// a program reading events is not a shell evaluating them, so the two sinks
/// render the same event differently and both are right.
#[test]
fn the_json_sink_still_carries_a_shell_line() {
    let out = Captured::default();
    let mut sink = JsonSink::new(Box::new(out.clone()));

    sink.handle(&Event::ShellLine {
        line: "export EDITOR=nvim".to_owned(),
    });
    sink.finish();

    let text = out.text();
    assert!(text.contains("shell_line"), "{text}");
    assert!(text.contains("export EDITOR=nvim"), "{text}");
}

/// [R-TUI-071] every sink that renders a run handles every variant, which the
/// compiler already guarantees: `Event` is not `#[non_exhaustive]` and the
/// matches are exhaustive, so a new variant is an error in each one.
///
/// What is left to check is that the guarantee is real rather than assumed --
/// that no sink reached for a catch-all and made a future variant silent.
#[test]
fn no_sink_that_renders_a_run_has_a_catch_all() {
    let sources = [
        include_str!("../src/sink.rs"),
        include_str!("../src/live.rs"),
    ];
    for source in sources {
        for (number, line) in source.lines().enumerate() {
            let code = line.split("//").next().unwrap_or(line);
            assert!(
                !code.trim_start().starts_with("_ =>"),
                "line {} reaches for a catch-all over Event: {line}",
                number + 1
            );
        }
    }
}

/// [R-TUI-040] detection resolves four things independently, so a pipe on a
/// colour terminal is not confused with a dumb terminal on a tty.
#[test]
fn the_four_capabilities_are_resolved_independently() {
    let piped_but_colourful = caps(ColourDepth::TrueColour, true);
    assert!(!piped_but_colourful.tty);
    assert_eq!(piped_but_colourful.colour, ColourDepth::TrueColour);

    let ascii = caps(ColourDepth::Ansi16, false);
    assert!(!ascii.unicode);
    assert_eq!(ascii.colour, ColourDepth::Ansi16);

    // A theme built from each renders differently, which is what makes the
    // four separate rather than one "is it fancy" flag.
    assert_ne!(
        Theme::new(piped_but_colourful).symbols().spinner,
        Theme::new(ascii).symbols().spinner
    );
}

/// [R-TUI-050] the palette is one table, so pointing at a user file later is
/// a reader rather than a restructuring.
#[test]
fn the_palette_is_data() {
    let theme = Theme::new(caps(ColourDepth::TrueColour, true));
    assert_eq!(theme.palette, meowctl_tui::theme::CATPPUCCIN);

    // Every role resolves through the table rather than through a literal at
    // the call site; see [R-TUI-051].
    let painted = theme.paint(meowctl_tui::Role::Muted, "x");
    assert!(painted.contains('x'), "{painted}");
}

/// [R-TUI-050] and [R-TUI-053]: a theme file names a role and its four
/// numbers, and the palette that comes back carries them.
#[test]
fn a_theme_file_replaces_the_role_it_names() {
    let palette = meowctl_tui::Palette::parse("[accent]\nr = 1\ng = 2\nb = 3\nansi16 = 4\n")
        .expect("a role and its numbers");

    assert_eq!(palette.accent.r, 1);
    assert_eq!(palette.accent.g, 2);
    assert_eq!(palette.accent.b, 3);
    assert_eq!(palette.accent.ansi16, 4);
}

/// [R-TUI-054] a role the file does not name keeps its default, so a user who
/// wants one colour changed writes one table.
#[test]
fn a_role_the_file_leaves_out_keeps_its_default() {
    let palette = meowctl_tui::Palette::parse("[accent]\nr = 1\ng = 2\nb = 3\nansi16 = 4\n")
        .expect("one role");

    assert_eq!(palette.success, meowctl_tui::theme::CATPPUCCIN.success);
    assert_eq!(palette.failure, meowctl_tui::theme::CATPPUCCIN.failure);
    assert_eq!(palette.muted, meowctl_tui::theme::CATPPUCCIN.muted);
    assert_ne!(palette.accent, meowctl_tui::theme::CATPPUCCIN.accent);
}

/// [R-TUI-054] a role it names is replaced whole. Three numbers and a missing
/// one is a mistake, not a request to mix in a default nobody chose.
#[test]
fn a_role_with_a_missing_number_is_refused() {
    let err = meowctl_tui::Palette::parse("[accent]\nr = 1\ng = 2\nb = 3\n")
        .expect_err("a partial colour");
    assert!(err.to_string().contains("ansi16"), "{err}");
}

/// [R-TUI-052] and [R-TUI-053]: a misspelled role is a mistake the user hears
/// about, not a table that silently does nothing.
#[test]
fn a_role_that_is_not_a_role_is_refused() {
    let err = meowctl_tui::Palette::parse("[acccent]\nr = 1\ng = 2\nb = 3\nansi16 = 4\n")
        .expect_err("a misspelled role");
    assert!(err.to_string().contains("acccent"), "{err}");
}

/// [R-TUI-052] a file that is not TOML at all falls back rather than failing
/// the command, and says what the parser saw.
#[test]
fn a_file_that_is_not_toml_is_refused_with_a_reason() {
    let err = meowctl_tui::Palette::parse("this is not toml {{{").expect_err("not toml");
    assert!(!err.to_string().is_empty(), "the reason is empty");
}

/// [R-TUI-050] an empty file is a valid one that changes nothing, because a
/// user who commented every role out has not made a mistake.
#[test]
fn an_empty_theme_file_is_the_default_palette() {
    let palette = meowctl_tui::Palette::parse("# nothing here\n").expect("an empty file");
    assert_eq!(palette, meowctl_tui::theme::CATPPUCCIN);
}

/// [R-TUI-051] and [R-TUI-050]: a palette from a file reaches what is
/// rendered, which is the whole point of the file.
#[test]
fn a_palette_from_a_file_reaches_the_rendered_line() {
    let palette = meowctl_tui::Palette::parse("[info]\nr = 255\ng = 0\nb = 0\nansi16 = 31\n")
        .expect("one role");

    let out = Captured::default();
    let theme = Theme::with_palette(caps(ColourDepth::TrueColour, true), palette);
    let mut sink = PlainSink::new(Box::new(out.clone()), theme);
    sink.handle(&Event::Message {
        level: Level::Info,
        text: "hello".to_owned(),
    });
    sink.finish();

    assert!(out.text().contains("255;0;0"), "{}", out.text());
}
