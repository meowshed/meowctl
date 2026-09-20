//! The symbol vocabulary and the palette.
//!
//! Three ideas hold the output together, and they carry over from `v0.1.0`
//! unchanged, because they are the reason it is readable:
//!
//! 1. **One symbol vocabulary.** A glyph means the same thing in every
//!    command. Before the design system there were four competing ones:
//!    `ok`/`FAIL`, `->`/`--`, `+`/`-`, and `✓`; see [R-TUI-001].
//! 2. **Two levels of indent, never more.** Column zero is the command
//!    talking about itself, two spaces is one item, and captured subprocess
//!    output sits under its item and dimmed. Deeper stops being scannable;
//!    see [R-TUI-002].
//! 3. **Colour is redundant.** Every state is legible from its symbol and its
//!    wording alone, so a pipe, `NO_COLOR`, or a monochrome terminal loses
//!    decoration and never information; see [R-TUI-003].
//!
//! What does not carry over is the palette being compiled in. It is data
//! here, with the Catppuccin values as the default, so a user whose terminal
//! clashes has somewhere to go; see [R-TUI-050].

use crate::caps::{Caps, ColourDepth};

/// What a line is saying.
///
/// A role, never a colour: call sites name one of these, so the palette
/// changes in one place; see [R-TUI-051].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// Something finished as it should.
    Success,
    /// Something failed.
    Failure,
    /// Something to notice that did not fail.
    Warning,
    /// The subject of the line.
    Accent,
    /// Information.
    Info,
    /// Present but not the point: a path, a count, captured output.
    Muted,
}

/// The glyph set for one capability tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Symbols {
    /// A component that installed, a check that passed, a dep that resolved.
    pub success: &'static str,
    /// Anything that failed.
    pub failure: &'static str,
    /// Something to notice.
    pub warning: &'static str,
    /// Information.
    pub info: &'static str,
    /// Waiting to run.
    pub pending: &'static str,
    /// Not run, with a reason.
    pub skipped: &'static str,
    /// Running now.
    pub running: &'static str,
    /// Being added.
    pub added: &'static str,
    /// Being removed.
    pub removed: &'static str,
    /// From, to.
    pub arrow: &'static str,
    /// A list item.
    pub bullet: &'static str,
    /// The frames a running item cycles through.
    ///
    /// Part of the tier for the same reason the separator is: a braille frame
    /// on a terminal that cannot show one is a mojibake risk on every redraw.
    pub spinner: &'static [&'static str],
    /// Between a subject and a remark about it.
    ///
    /// Part of the tier rather than written into the text, because a line
    /// rendered for a non-UTF-8 terminal has to be ASCII all the way through:
    /// one em dash in a message makes the whole line a mojibake risk.
    pub separator: &'static str,
}

/// The glyphs from `internal/tui/theme.go`.
pub const UNICODE: Symbols = Symbols {
    success: "✓",
    failure: "✗",
    warning: "!",
    info: "i",
    pending: "·",
    skipped: "–",
    running: "▸",
    added: "+",
    removed: "−",
    arrow: "→",
    bullet: "·",
    spinner: &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
    separator: "—",
};

/// The same distinctions without Unicode.
///
/// Every state stays distinguishable: an ASCII terminal loses shape and not
/// meaning; see [R-TUI-043].
pub const ASCII: Symbols = Symbols {
    success: "+",
    failure: "x",
    warning: "!",
    info: "i",
    pending: ".",
    skipped: "-",
    running: ">",
    added: "+",
    removed: "-",
    arrow: "->",
    bullet: "*",
    spinner: &["|", "/", "-", "\\"],
    separator: "--",
};

/// A colour, as the palette stores it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Colour {
    /// Red.
    pub r: u8,
    /// Green.
    pub g: u8,
    /// Blue.
    pub b: u8,
    /// The nearest of the sixteen ANSI colours, for a terminal that takes no
    /// more. Chosen rather than computed, because the nearest by distance is
    /// not always the one that reads correctly.
    pub ansi16: u8,
}

/// The palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    /// Something finished as it should.
    pub success: Colour,
    /// Something failed.
    pub failure: Colour,
    /// Something to notice.
    pub warning: Colour,
    /// The subject of the line.
    pub accent: Colour,
    /// Information.
    pub info: Colour,
    /// Present but not the point.
    pub muted: Colour,
}

/// Catppuccin, which is what `v0.1.0` compiles in and what the rest of this
/// configuration uses.
pub const CATPPUCCIN: Palette = Palette {
    success: Colour {
        r: 166,
        g: 227,
        b: 161,
        ansi16: 32,
    },
    failure: Colour {
        r: 243,
        g: 139,
        b: 168,
        ansi16: 31,
    },
    warning: Colour {
        r: 249,
        g: 226,
        b: 175,
        ansi16: 33,
    },
    accent: Colour {
        r: 203,
        g: 166,
        b: 247,
        ansi16: 35,
    },
    info: Colour {
        r: 137,
        g: 180,
        b: 250,
        ansi16: 34,
    },
    muted: Colour {
        r: 127,
        g: 132,
        b: 156,
        ansi16: 90,
    },
};

/// How a line is rendered.
#[derive(Debug, Clone, Copy)]
pub struct Theme {
    /// What the destination can take.
    pub caps: Caps,
    /// The colours.
    pub palette: Palette,
}

impl Theme {
    /// A theme for these capabilities, with the default palette.
    #[must_use]
    pub const fn new(caps: Caps) -> Self {
        Theme {
            caps,
            palette: CATPPUCCIN,
        }
    }

    /// The glyphs this destination can show.
    #[must_use]
    pub const fn symbols(&self) -> Symbols {
        if self.caps.unicode { UNICODE } else { ASCII }
    }

    /// The glyph for a role.
    #[must_use]
    pub const fn symbol(&self, role: Role) -> &'static str {
        let s = self.symbols();
        match role {
            Role::Success => s.success,
            Role::Failure => s.failure,
            Role::Warning => s.warning,
            Role::Accent | Role::Info => s.info,
            Role::Muted => s.bullet,
        }
    }

    /// Wraps text in the escape for a role, or returns it unchanged when the
    /// destination takes no colour.
    #[must_use]
    pub fn paint(&self, role: Role, text: &str) -> String {
        let colour = match role {
            Role::Success => self.palette.success,
            Role::Failure => self.palette.failure,
            Role::Warning => self.palette.warning,
            Role::Accent => self.palette.accent,
            Role::Info => self.palette.info,
            Role::Muted => self.palette.muted,
        };

        // Downsampled to what the terminal reports rather than assumed; see
        // [R-TUI-042].
        match self.caps.colour {
            ColourDepth::None => text.to_owned(),
            ColourDepth::Ansi16 => format!("\x1b[{}m{text}\x1b[0m", colour.ansi16),
            ColourDepth::Ansi256 => {
                format!("\x1b[38;5;{}m{text}\x1b[0m", nearest_256(colour))
            }
            ColourDepth::TrueColour => format!(
                "\x1b[38;2;{};{};{}m{text}\x1b[0m",
                colour.r, colour.g, colour.b
            ),
        }
    }
}

/// The nearest colour in the 256-colour cube.
///
/// The 6×6×6 cube only, skipping the greyscale ramp: every palette colour here
/// is chromatic, and a grey that happened to be nearer would read as a
/// different thing.
fn nearest_256(colour: Colour) -> u8 {
    let level = |c: u8| -> u8 {
        // The cube's levels are 0, 95, 135, 175, 215, 255.
        const LEVELS: [u8; 6] = [0, 95, 135, 175, 215, 255];
        let mut best = 0u8;
        let mut best_distance = u16::MAX;
        for (index, value) in LEVELS.iter().enumerate() {
            let distance = u16::from(c.abs_diff(*value));
            if distance < best_distance {
                best_distance = distance;
                best = u8::try_from(index).unwrap_or(0);
            }
        }
        best
    };
    16 + 36 * level(colour.r) + 6 * level(colour.g) + level(colour.b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::caps::Caps;

    fn theme(colour: ColourDepth, unicode: bool) -> Theme {
        Theme::new(Caps {
            height: None,
            tty: true,
            motion: false,
            unicode,
            colour,
            width: Some(80),
        })
    }

    /// [R-TUI-001] a glyph means one thing everywhere, which is what the
    /// vocabulary is for.
    #[test]
    fn every_state_has_a_distinct_glyph_in_both_tiers() {
        for symbols in [UNICODE, ASCII] {
            let all = [
                symbols.success,
                symbols.failure,
                symbols.pending,
                symbols.skipped,
                symbols.running,
            ];
            let unique: std::collections::BTreeSet<&str> = all.iter().copied().collect();
            assert_eq!(unique.len(), all.len(), "{symbols:?} reuses a glyph");
        }
    }

    /// [R-TUI-043] an ASCII terminal loses shape, not meaning.
    #[test]
    fn a_non_unicode_locale_gets_the_ascii_set() {
        assert_eq!(theme(ColourDepth::None, false).symbols(), ASCII);
        assert_eq!(theme(ColourDepth::None, true).symbols(), UNICODE);
    }

    /// Every glyph in the ASCII tier, not only the ones somebody remembered:
    /// one non-ASCII character makes the whole line a mojibake risk.
    #[test]
    fn the_ascii_tier_is_entirely_ascii() {
        for glyph in [
            ASCII.success,
            ASCII.failure,
            ASCII.warning,
            ASCII.info,
            ASCII.pending,
            ASCII.skipped,
            ASCII.running,
            ASCII.added,
            ASCII.removed,
            ASCII.arrow,
            ASCII.bullet,
            ASCII.separator,
        ] {
            assert!(glyph.is_ascii(), "{glyph:?} is not ASCII");
        }
    }

    /// [R-TUI-003] a monochrome terminal loses decoration and never
    /// information, so the text is the text.
    #[test]
    fn no_colour_returns_the_text_unchanged() {
        assert_eq!(
            theme(ColourDepth::None, true).paint(Role::Success, "done"),
            "done"
        );
    }

    /// [R-TUI-042] truecolor on a 16-colour terminal is garbage, so the
    /// escape matches the depth.
    #[test]
    fn the_escape_matches_the_depth() {
        assert_eq!(
            theme(ColourDepth::TrueColour, true).paint(Role::Success, "x"),
            "\x1b[38;2;166;227;161mx\x1b[0m"
        );
        assert!(
            theme(ColourDepth::Ansi256, true)
                .paint(Role::Success, "x")
                .starts_with("\x1b[38;5;")
        );
        assert_eq!(
            theme(ColourDepth::Ansi16, true).paint(Role::Failure, "x"),
            "\x1b[31mx\x1b[0m"
        );
    }

    /// Every painted string closes what it opened, or the colour runs on into
    /// whatever the shell prints next.
    #[test]
    fn every_escape_is_closed() {
        let theme = theme(ColourDepth::TrueColour, true);
        for role in [
            Role::Success,
            Role::Failure,
            Role::Warning,
            Role::Accent,
            Role::Info,
            Role::Muted,
        ] {
            assert!(theme.paint(role, "x").ends_with("\x1b[0m"), "{role:?}");
        }
    }

    /// The cube, not the greyscale ramp: a grey that happened to be nearer
    /// would read as a different thing.
    #[test]
    fn the_256_colour_approximation_stays_in_the_cube() {
        for colour in [
            CATPPUCCIN.success,
            CATPPUCCIN.failure,
            CATPPUCCIN.warning,
            CATPPUCCIN.accent,
            CATPPUCCIN.info,
            CATPPUCCIN.muted,
        ] {
            let index = nearest_256(colour);
            assert!((16..232).contains(&index), "{colour:?} mapped to {index}");
        }
    }
}
