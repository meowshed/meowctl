//! The in-place renderer, rendered into a buffer.
//!
//! Every test here drives the sink with a recorded stream and reads back the
//! bytes it wrote, escapes and all. That is what makes the cursor arithmetic
//! checkable: a renderer that draws three lines and walks up two leaves the
//! next frame overwriting a permanent line, and only counting the escapes
//! catches it.

// `clippy.toml` exempts a function carrying `#[test]`; a helper in a test
// binary is test code by construction.
#![allow(clippy::expect_used)]

use std::sync::{Arc, Mutex};

use meowctl_common::{ComponentId, Event, Outcome, Phase, Stream};
use meowctl_tui::{Caps, ColourDepth, LiveSink, Sink, Theme};

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

/// Capabilities a test can state, with no terminal behind them.
///
/// `tty` is false so the width and height come from these fields rather than
/// from whatever terminal the test happens to run under.
fn caps(colour: ColourDepth, unicode: bool, width: u16, height: u16) -> Caps {
    Caps {
        tty: false,
        motion: true,
        unicode,
        colour,
        width: Some(width),
        height: Some(height),
    }
}

fn component(name: &str) -> ComponentId {
    name.parse().expect("a component id")
}

/// Renders a stream and returns the bytes, with the escapes made readable.
fn render(events: &[Event], caps: Caps) -> String {
    let captured = Captured::default();
    {
        let mut sink = LiveSink::new(Box::new(captured.clone()), Theme::new(caps));
        for event in events {
            sink.handle(event);
        }
        sink.finish();
    }
    readable(&captured.text())
}

/// The drawn frames, each without its surrounding control sequences.
fn frames(rendered: &str) -> Vec<String> {
    rendered
        .split("<sync>")
        .skip(1)
        .filter_map(|rest| rest.split("</sync>").next().map(str::to_owned))
        .collect()
}

/// A rendered line with the control markers taken out, which is what a reader
/// of the terminal would see.
fn visible(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut rest = line;
    while let Some(start) = rest.find('<') {
        out.push_str(&rest[..start]);
        match rest[start..].find('>') {
            Some(end) => rest = &rest[start + end + 1..],
            None => {
                rest = "";
                break;
            }
        }
    }
    out.push_str(rest);
    out
}

/// Names the control sequences so a snapshot can be reviewed.
fn readable(text: &str) -> String {
    text.replace("\x1b[1A", "<up>")
        .replace("\x1b[2K", "<erase>")
        .replace("\x1b[?25l", "<hide>")
        .replace("\x1b[?25h", "<show>")
        .replace("\x1b[?2026h", "<sync>")
        .replace("\x1b[?2026l", "</sync>")
        .replace('\r', "<cr>")
        .replace('\x1b', "<esc>")
}

fn install(component_name: &str) -> Vec<Event> {
    vec![
        Event::PhaseStarted {
            phase: Phase::Install,
            total: 1,
        },
        Event::ComponentStarted {
            component: component(component_name),
            phase: Phase::Install,
        },
        Event::ComponentFinished {
            component: component(component_name),
            phase: Phase::Install,
            outcome: Outcome::Succeeded,
        },
        Event::PhaseFinished {
            phase: Phase::Install,
            failed: 0,
        },
    ]
}

/// [R-TUI-020] exactly one cursor-up per line drawn. A frame that drew three
/// lines and walked back two would overwrite a permanent line on the next one,
/// and the damage shows up hours later as a garbled transcript.
#[test]
fn every_line_drawn_is_walked_back_over_exactly_once() {
    let events = vec![
        Event::ComponentStarted {
            component: component("a"),
            phase: Phase::Install,
        },
        Event::ComponentStarted {
            component: component("b"),
            phase: Phase::Install,
        },
        Event::ComponentStarted {
            component: component("c"),
            phase: Phase::Install,
        },
    ];
    let rendered = render(&events, caps(ColourDepth::None, true, 80, 24));

    // Three frames of one, two and three lines.
    let drawn: usize = frames(&rendered)
        .iter()
        .map(|f| f.matches("<erase>").count())
        .sum();
    assert_eq!(drawn, 1 + 2 + 3, "{rendered:?}");
    assert_eq!(
        rendered.matches("<up>").count(),
        drawn,
        "one cursor-up per line drawn, no more and no fewer: {rendered:?}"
    );
}

/// [R-TUI-020] a permanent line is written where the region was, not inside
/// it, so the region never scrolls a finished component away.
#[test]
fn a_finished_component_is_committed_above_the_region() {
    let rendered = render(&install("neovim"), caps(ColourDepth::None, true, 80, 24));
    insta::assert_snapshot!(rendered);
}

/// [R-TUI-022] the terminal belongs to the subprocess until it is released.
/// A renderer that kept drawing would fight the password prompt a cask puts
/// up, which is the case this exists for.
#[test]
fn the_region_stands_down_while_a_subprocess_owns_the_terminal() {
    let events = vec![
        Event::ComponentStarted {
            component: component("homebrew"),
            phase: Phase::Install,
        },
        Event::TerminalRequested,
        // Everything between these two must write nothing.
        Event::ProcessOutput {
            stream: Stream::Stdout,
            line: "Password:".to_owned(),
        },
        Event::OpApplied {
            kind: "write_file".to_owned(),
            target: "/tmp/x".to_owned(),
        },
        Event::TerminalReleased,
    ];
    let rendered = render(&events, caps(ColourDepth::None, true, 80, 24));

    let requested = rendered.find("<show>").expect("the cursor came back");
    let released = rendered.rfind("<hide>").expect("and went away again");
    let quiet = &rendered[requested..released];
    assert!(
        !quiet.contains("homebrew"),
        "nothing was drawn while the terminal was lent out: {quiet:?}"
    );
}

/// [R-TUI-023] a command that exits with a hidden cursor leaves the user's
/// shell broken, so the sink gives it back when it is finished with.
#[test]
fn the_cursor_is_restored_when_the_sink_finishes() {
    let rendered = render(&install("neovim"), caps(ColourDepth::None, true, 80, 24));
    assert!(rendered.contains("<hide>"), "{rendered:?}");
    assert!(rendered.trim_end().ends_with("<show>"), "{rendered:?}");
}

/// [R-TUI-023] and when it is dropped, which is the path a panic takes.
#[test]
fn the_cursor_is_restored_when_the_sink_is_dropped() {
    let captured = Captured::default();
    {
        let mut sink = LiveSink::new(
            Box::new(captured.clone()),
            Theme::new(caps(ColourDepth::None, true, 80, 24)),
        );
        sink.handle(&Event::ComponentStarted {
            component: component("neovim"),
            phase: Phase::Install,
        });
        // No `finish`: the drop is what has to put it back.
    }
    assert!(
        readable(&captured.text()).ends_with("<show>"),
        "{:?}",
        readable(&captured.text())
    );
}

/// [R-TUI-020] the region is capped to the viewport, or the cursor-up
/// arithmetic of the next frame walks off the top of the screen.
#[test]
fn the_region_is_capped_to_the_viewport() {
    let events: Vec<Event> = (0..20)
        .map(|n| Event::ComponentStarted {
            component: component(&format!("component-{n}")),
            phase: Phase::Install,
        })
        .collect();
    // Ten rows of viewport leaves six lines for the region.
    let rendered = render(&events, caps(ColourDepth::None, true, 80, 10));

    let last_frame = frames(&rendered).pop().expect("a frame");
    let lines = last_frame.matches("<erase>").count();
    assert_eq!(lines, 7, "six rows and an overflow line: {last_frame:?}");
    assert!(
        last_frame.contains("14 more"),
        "the hidden ones are counted: {last_frame:?}"
    );
}

/// [R-TUI-024] a component installing forty packages looks identical to one
/// installing none unless the sub-work reaches the screen, and it goes on the
/// component's own line rather than a third level of indent.
#[test]
fn sub_work_goes_on_the_components_own_line() {
    let events = vec![
        Event::ComponentStarted {
            component: component("developer-tools"),
            phase: Phase::Install,
        },
        Event::ProcessStarted {
            program: "brew".to_owned(),
            args: vec!["install".to_owned(), "git".to_owned()],
        },
    ];
    let rendered = render(&events, caps(ColourDepth::None, true, 80, 24));
    let last_frame = frames(&rendered).pop().expect("a frame");

    assert!(last_frame.contains("developer-tools"), "{last_frame:?}");
    assert!(last_frame.contains("brew install git"), "{last_frame:?}");
    for line in last_frame.lines().filter(|l| l.contains("developer-tools")) {
        let text = visible(line);
        let indent = text.len() - text.trim_start().len();
        assert_eq!(indent, 2, "one indent, not a third level: {line:?}");
    }
}

/// [R-TUI-021] captured output appears under its component while the process
/// runs, which is the whole reason it is captured rather than streamed.
#[test]
fn captured_output_appears_while_the_process_runs() {
    let events = vec![
        Event::ComponentStarted {
            component: component("neovim"),
            phase: Phase::Install,
        },
        Event::ProcessOutput {
            stream: Stream::Stdout,
            line: "downloading".to_owned(),
        },
    ];
    let rendered = render(&events, caps(ColourDepth::None, true, 80, 24));
    let last_frame = frames(&rendered).pop().expect("a frame");
    assert!(last_frame.contains("downloading"), "{last_frame:?}");
}

/// [R-TUI-025] a resize is picked up without a signal handler, because the
/// width is read again for every frame.
#[test]
fn a_narrow_terminal_truncates_rather_than_wraps() {
    let events = vec![Event::ComponentStarted {
        component: component("@stdlib//components/a-very-long-component-name"),
        phase: Phase::Install,
    }];
    let rendered = render(&events, caps(ColourDepth::None, true, 30, 24));

    for line in rendered.lines() {
        let text = visible(line);
        assert!(
            text.chars().count() <= 30,
            "a wrapped line breaks the next frame's arithmetic: {line:?} ({} wide)",
            text.chars().count()
        );
    }
}

/// [R-TUI-004] no sink puts the terminal into raw mode, and none of the
/// sequences that need it are written either: no alternate screen, no scroll
/// region, no cursor save and restore. That is where terminal support
/// diverges, and a hook shelling out needs the terminal it was given.
#[test]
fn only_the_portable_control_subset_is_written() {
    let rendered = render(
        &install("neovim"),
        caps(ColourDepth::TrueColour, true, 80, 24),
    );
    for forbidden in ["<esc>[?1049", "<esc>7", "<esc>8", "<esc>[r", "<esc>[?1h"] {
        assert!(!rendered.contains(forbidden), "{forbidden} in {rendered:?}");
    }
}

/// [R-TUI-081] the matrix, in one snapshot. `v0.1.0` checks a few cases by
/// hand, which is how a tier nobody thought about ships broken.
#[test]
fn the_capability_matrix_renders() {
    let mut out = String::new();
    for colour in [
        ColourDepth::None,
        ColourDepth::Ansi16,
        ColourDepth::Ansi256,
        ColourDepth::TrueColour,
    ] {
        for unicode in [true, false] {
            for width in [80_u16, 30] {
                let caps = caps(colour, unicode, width, 24);
                out.push_str(&format!(
                    "--- colour {colour:?}, unicode {unicode}, width {width}\n"
                ));
                out.push_str(&render(&install("@stdlib//components/neovim"), caps));
                out.push('\n');
            }
        }
    }
    insta::assert_snapshot!(out);
}

/// [R-TUI-026] the spinner advances on events rather than on a timer.
///
/// `v0.1.0` animates from a goroutine, which draws while nothing is
/// happening and stops drawing while something is. A frame tied to the stream
/// moves exactly when there is news, and a test can therefore see it move
/// without waiting.
#[test]
fn the_spinner_advances_on_events_and_not_on_time() {
    let started = vec![
        Event::ComponentStarted {
            component: component("zsh"),
            phase: Phase::Install,
        },
        Event::ProcessOutput {
            stream: Stream::Stdout,
            line: "one".to_owned(),
        },
    ];
    let terminal = caps(ColourDepth::Ansi256, true, 80, 24);
    let first = render(&started[..1], terminal);
    let second = render(&started, terminal);

    assert_ne!(
        first, second,
        "the spinner did not move when an event arrived"
    );

    // And it does not move on its own: rendering the same stream twice gives
    // the same frame, whatever the clock has done in between.
    assert_eq!(render(&started, terminal), second);
}

/// [R-TUI-020] a component name longer than the terminal is truncated, and
/// its note goes rather than wrapping.
///
/// A line that wraps costs the region a row it did not account for, and the
/// cursor-up that follows then walks to the wrong place and tears the frame.
#[test]
fn a_name_longer_than_the_terminal_is_truncated_and_loses_its_note() {
    let narrow = caps(ColourDepth::Ansi16, true, 24, 24);
    let long = "@stdlib//components/a-component-with-a-very-long-name";

    let rendered = render(
        &[
            Event::ComponentStarted {
                component: component(long),
                phase: Phase::Install,
            },
            Event::PathPrepended {
                directory: "/opt/homebrew/bin".to_owned(),
            },
        ],
        narrow,
    );

    // Truncated rather than wrapped: the ellipsis is the mark that the name
    // was cut, and the whole name is not there.
    assert!(rendered.contains('\u{2026}'), "{rendered}");
    assert!(
        !rendered.contains(long),
        "the whole name was drawn: {rendered}"
    );
    assert!(
        !rendered.contains("/opt/homebrew/bin"),
        "the note survived a name that filled the line: {rendered}"
    );
}

/// [R-TUI-020] a note is shown when there is room for it, so the width
/// arithmetic is not simply dropping everything.
#[test]
fn a_note_is_shown_when_the_line_has_room() {
    let wide = caps(ColourDepth::Ansi16, true, 120, 24);
    let rendered = render(
        &[
            Event::ComponentStarted {
                component: component("zsh"),
                phase: Phase::Install,
            },
            Event::PathPrepended {
                directory: "/opt/homebrew/bin".to_owned(),
            },
        ],
        wide,
    );

    assert!(
        rendered.contains("/opt/homebrew/bin"),
        "the note was dropped on a wide terminal: {rendered}"
    );
}

/// [R-TUI-024] a phase that failed says how many, and one that did not says
/// nothing. A count of zero failures is a line nobody needs.
#[test]
fn a_phase_reports_its_failures_and_only_when_there_are_some() {
    let terminal = caps(ColourDepth::Ansi16, true, 80, 24);

    let quiet = render(
        &[Event::PhaseFinished {
            phase: Phase::Install,
            failed: 0,
        }],
        terminal,
    );
    assert!(!quiet.contains("failed"), "{quiet}");

    let noisy = render(
        &[Event::PhaseFinished {
            phase: Phase::Install,
            failed: 2,
        }],
        terminal,
    );
    assert!(noisy.contains("2 failed"), "{noisy}");
}
