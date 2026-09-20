//! The in-place renderer.
//!
//! A region of status lines that is erased and redrawn where it stands, with
//! finished work committed above it as permanent lines. The terminal never
//! goes into raw mode, because a hook shells out to commands that need the
//! real one; see [R-TUI-004].
//!
//! Only the portable control subset is used — cursor-up, erase-line, and
//! hide/show cursor — plus synchronised output, which a terminal that does not
//! know it ignores. No alternate screen, no scroll region, no cursor
//! save and restore: that is where terminal support actually diverges.

use std::io::Write;

use meowctl_common::{ComponentId, Event, Level, Outcome};

use crate::sink::{Sink, describe, role_of};
use crate::theme::{Role, Theme};

/// Move the cursor up one line.
const CURSOR_UP: &str = "\x1b[1A";
/// Erase the whole line the cursor is on.
const ERASE_LINE: &str = "\x1b[2K";
/// Put the cursor at column zero.
const CARRIAGE_RETURN: &str = "\r";
/// Hide the cursor.
const HIDE_CURSOR: &str = "\x1b[?25l";
/// Show it again.
const SHOW_CURSOR: &str = "\x1b[?25h";
/// Begin a frame a terminal should present atomically.
const SYNC_BEGIN: &str = "\x1b[?2026h";
/// End it.
const SYNC_END: &str = "\x1b[?2026l";

/// The width to assume when the terminal will not say.
const FALLBACK_WIDTH: u16 = 80;

/// The narrowest width worth laying out against.
const MINIMUM_WIDTH: u16 = 20;

/// How many status lines the region holds when the height is unknown.
const FALLBACK_ROWS: usize = 12;

/// One line of the live region.
///
/// Every row is a component that is running: one that finished leaves the
/// region and is committed above it as a permanent line, so there is no
/// status to carry here.
#[derive(Debug, Clone)]
struct Row {
    component: ComponentId,
    /// What the component is doing now, shown on its own line rather than by
    /// indenting further; see [R-TUI-024].
    note: String,
}

/// The renderer for a terminal that takes motion.
pub struct LiveSink {
    out: Box<dyn Write>,
    theme: Theme,
    /// The components currently in the region, in the order they started.
    rows: Vec<Row>,
    /// How many components were skipped, shown as a count rather than a line
    /// each; see [R-TUI-024].
    skipped: usize,
    /// How many lines the last frame drew, which is how many to walk back up.
    drawn: usize,
    /// Which spinner frame the next redraw uses; see [R-TUI-026].
    frame: usize,
    /// True between `TerminalRequested` and `TerminalReleased`, when the
    /// terminal belongs to a subprocess; see [R-TUI-022].
    paused: bool,
    /// True once the cursor has been hidden, so it is shown exactly once.
    hidden: bool,
}

impl std::fmt::Debug for LiveSink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LiveSink")
            .field("theme", &self.theme)
            .field("rows", &self.rows.len())
            .field("drawn", &self.drawn)
            .field("paused", &self.paused)
            .finish_non_exhaustive()
    }
}

impl LiveSink {
    /// A renderer writing to `out`.
    #[must_use]
    pub fn new(out: Box<dyn Write>, theme: Theme) -> Self {
        LiveSink {
            out,
            theme,
            rows: Vec::new(),
            skipped: 0,
            drawn: 0,
            frame: 0,
            paused: false,
            hidden: false,
        }
    }

    /// Writes bytes, ignoring a failure.
    ///
    /// A closed pipe is normal when output goes into `head`, and a run must
    /// not abort for it; see [R-TUI-070].
    fn raw(&mut self, text: &str) {
        let _ = self.out.write_all(text.as_bytes());
        let _ = self.out.flush();
    }

    /// Erases the region and leaves the cursor where it began.
    ///
    /// Exactly one cursor-up per line drawn, which is why `drawn` is a count
    /// of lines and not of rows: a frame that drew an overflow line has to
    /// walk back over it too; see [R-TUI-020].
    fn erase(&mut self) {
        if self.drawn == 0 {
            return;
        }
        let mut out = String::with_capacity(self.drawn * 12);
        for _ in 0..self.drawn {
            out.push_str(CURSOR_UP);
            out.push_str(ERASE_LINE);
            out.push_str(CARRIAGE_RETURN);
        }
        self.raw(&out);
        self.drawn = 0;
    }

    /// Writes a permanent line above the region.
    ///
    /// The region is erased first and redrawn afterwards, so a committed line
    /// never lands inside it; see [R-TUI-020].
    fn commit(&mut self, text: &str) {
        self.erase();
        let line = format!("{text}\n");
        self.raw(&line);
        self.draw();
    }

    /// Redraws the region.
    fn draw(&mut self) {
        if self.paused {
            return;
        }
        self.erase();
        let lines = self.frame_lines();
        if lines.is_empty() {
            return;
        }
        // Advanced only when something was drawn, so a redraw that had
        // nothing to show does not skip a frame; see [R-TUI-026].
        self.frame = self.frame.wrapping_add(1);
        if !self.hidden {
            self.raw(HIDE_CURSOR);
            self.hidden = true;
        }

        let mut out = String::from(SYNC_BEGIN);
        for line in &lines {
            out.push_str(ERASE_LINE);
            out.push_str(line);
            out.push('\n');
        }
        out.push_str(SYNC_END);
        self.raw(&out);
        self.drawn = lines.len();
    }

    /// The region's content for this frame.
    fn frame_lines(&self) -> Vec<String> {
        let mut lines: Vec<String> = self.rows.iter().map(|row| self.row_line(row)).collect();

        // Capped to the viewport, so the cursor-up arithmetic of the next
        // frame can never walk past the top of the screen; see [R-TUI-020].
        let limit = self.max_rows();
        if lines.len() > limit {
            let hidden = lines.len() - limit;
            lines.truncate(limit);
            lines.push(self.muted(&format!("  {} {hidden} more", self.theme.symbols().bullet)));
        }
        if self.skipped > 0 {
            lines.push(self.muted(&format!("  {} already complete", self.skipped)));
        }
        lines
    }

    /// One component's line.
    fn row_line(&self, row: &Row) -> String {
        let frames = self.theme.symbols().spinner;
        let mark = self
            .theme
            .paint(Role::Accent, frames[self.frame % frames.len()]);

        // Two spaces of indent, the mark, a space, the name, then two spaces
        // before a note. Nothing indents further; see [R-TUI-002].
        let budget = usize::from(self.width().saturating_sub(4));
        let name = row.component.to_string();
        let (name, note) = if display_width(&name) > budget {
            (truncate(&name, budget), String::new())
        } else {
            let remaining = budget.saturating_sub(display_width(&name) + 2);
            let note = if row.note.is_empty() || remaining < 4 {
                String::new()
            } else {
                truncate(&row.note, remaining)
            };
            (name, note)
        };

        let mut line = format!("  {mark} {name}");
        if !note.is_empty() {
            line.push_str("  ");
            line.push_str(&self.theme.paint(Role::Muted, &note));
        }
        line
    }

    /// A muted line.
    fn muted(&self, text: &str) -> String {
        self.theme.paint(Role::Muted, text)
    }

    /// The width now.
    ///
    /// Re-read per frame, so a resize is picked up without a signal handler;
    /// see [R-TUI-025].
    fn width(&self) -> u16 {
        self.theme
            .caps
            .current_width()
            .filter(|w| *w >= MINIMUM_WIDTH)
            .unwrap_or(FALLBACK_WIDTH)
    }

    /// How many component lines the region may hold.
    ///
    /// Four lines short of the viewport, so the command has somewhere to
    /// commit to and the region never fills the screen.
    fn max_rows(&self) -> usize {
        self.theme
            .caps
            .current_height()
            .map_or(FALLBACK_ROWS, |h| usize::from(h).saturating_sub(4).max(1))
    }

    /// Puts the cursor back and stops drawing.
    fn stand_down(&mut self) {
        self.erase();
        if self.hidden {
            self.raw(SHOW_CURSOR);
            self.hidden = false;
        }
    }
}

impl Sink for LiveSink {
    fn handle(&mut self, event: &Event) {
        match event {
            Event::PlanComputed { phase_set, steps } => {
                // One entry per component, for the reason the plain sink
                // gives: three hundred lines to say what a hundred say.
                let planned = crate::sink::by_component(steps);
                let running = planned.iter().filter(|(_, skip)| skip.is_none()).count();
                let heading = self.theme.paint(
                    Role::Accent,
                    &format!("{phase_set}: {running} of {} component(s)", planned.len()),
                );
                self.commit(&heading);
                for (component, skipped) in planned {
                    if let Some(reason) = skipped {
                        let separator = self.theme.symbols().separator;
                        let line = self.item(
                            Role::Muted,
                            &format!("{component} {separator} {}", describe(&reason)),
                        );
                        self.commit(&line);
                    }
                }
            }

            Event::PhaseStarted { phase, total } => {
                let heading = self
                    .theme
                    .paint(Role::Accent, &format!("{phase} ({total})"));
                self.commit(&heading);
                self.rows.clear();
                self.skipped = 0;
            }

            Event::PhaseFinished { phase, failed } => {
                self.rows.clear();
                self.skipped = 0;
                self.erase();
                if *failed > 0 {
                    let line = self.item(Role::Failure, &format!("{phase}: {failed} failed"));
                    self.commit(&line);
                }
            }

            Event::ComponentStarted { component, .. } => {
                self.rows.push(Row {
                    component: component.clone(),
                    note: String::new(),
                });
                self.draw();
            }

            // A skip is a count on its own line rather than a line each: on a
            // configuration whose components come from an aggregate module
            // there are a hundred of them; see [R-TUI-024].
            Event::ComponentSkipped { .. } => {
                self.skipped += 1;
                self.draw();
            }

            Event::ComponentFinished {
                component, outcome, ..
            } => {
                self.rows.retain(|r| &r.component != component);
                match outcome {
                    Outcome::Succeeded => {
                        let line = self.item(Role::Success, &component.to_string());
                        self.commit(&line);
                    }
                    // Nothing to do is a success with nothing to say; see
                    // [R-ENGINE-033].
                    Outcome::NothingToDo => self.draw(),
                    Outcome::Failed { error } => {
                        let line = self.item(
                            Role::Failure,
                            &format!("{component}: {}", first_line(error)),
                        );
                        self.commit(&line);
                    }
                }
            }

            // Sub-work goes on the component's status line, which keeps the
            // two-indent rule; see [R-TUI-024].
            Event::OpApplied { kind, target } => {
                self.set_note(&format!("{kind} {target}"));
            }

            Event::ProcessStarted { program, args } => {
                self.set_note(&format!("$ {program} {}", args.join(" ")));
            }

            // Captured output appears under its component while the process
            // runs, on the component's own line; see [R-TUI-021].
            Event::ProcessOutput { line, .. } => {
                self.set_note(line);
            }

            Event::ProcessFinished { .. } => {
                self.set_note("");
            }

            // The terminal belongs to the subprocess until it is released:
            // the region goes, the cursor comes back, and nothing is written
            // in between; see [R-TUI-022].
            Event::TerminalRequested => {
                self.stand_down();
                self.paused = true;
            }

            Event::TerminalReleased => {
                self.paused = false;
                self.draw();
            }

            Event::Diagnostic {
                level,
                message,
                span,
            } => {
                let role = role_of(*level);
                match span {
                    Some(span) => {
                        let line = self.item(role, &format!("{span}: {message}"));
                        self.commit(&line);
                        if let Some(source) = &span.source_line {
                            let line = self.muted(&format!("      {source}"));
                            self.commit(&line);
                        }
                    }
                    None => {
                        let line = self.item(role, message);
                        self.commit(&line);
                    }
                }
            }

            Event::ShellLine { line } => {
                let text = self.muted(&format!("      {line}"));
                self.commit(&text);
            }

            Event::PathPrepended { directory } => {
                self.set_note(&format!("PATH += {directory}"));
            }

            Event::Message { level, text } => match level {
                // Verbose detail is held back rather than shown, because a
                // live region that says everything says nothing.
                Level::Debug => {}
                level => {
                    let line = self.item(role_of(*level), text);
                    self.commit(&line);
                }
            },
        }
    }

    /// Erases the region and gives the cursor back; see [R-TUI-023].
    fn finish(&mut self) {
        self.rows.clear();
        self.skipped = 0;
        self.stand_down();
    }
}

impl Drop for LiveSink {
    /// A panic between drawing and finishing would otherwise leave the user's
    /// shell without a cursor; see [R-TUI-023].
    fn drop(&mut self) {
        self.stand_down();
    }
}

impl LiveSink {
    /// A line at one indent: one item.
    fn item(&self, role: Role, text: &str) -> String {
        let symbol = self.theme.paint(role, self.theme.symbol(role));
        format!("  {symbol} {text}")
    }

    /// Puts a note on the newest running row and redraws.
    fn set_note(&mut self, note: &str) {
        let Some(row) = self.rows.last_mut() else {
            return;
        };
        row.note = note.to_owned();
        self.draw();
    }
}

/// The first line of a message.
///
/// A failure's first line is what fits on a status line; the rest is in the
/// error the command reports at the end.
fn first_line(text: &str) -> &str {
    text.split('\n').next().unwrap_or(text)
}

/// How wide a string renders.
///
/// Counted in characters rather than in grapheme clusters or East Asian
/// widths: a wrapped line breaks the cursor-up arithmetic of the next frame,
/// so the count has to be an under-estimate at worst, and a component name is
/// a path.
fn display_width(text: &str) -> usize {
    text.chars().count()
}

/// Cuts a string to fit, marking that it was cut.
fn truncate(text: &str, budget: usize) -> String {
    if display_width(text) <= budget {
        return text.to_owned();
    }
    if budget <= 1 {
        return String::new();
    }
    let kept: String = text.chars().take(budget - 1).collect();
    format!("{kept}…")
}
