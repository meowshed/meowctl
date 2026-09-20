//! What the output destination can render.
//!
//! Four things, resolved independently, because a CI job with a pty, a UTF-8
//! pipe, and a 16-colour terminal each need a different answer; see
//! [R-TUI-040].

use std::io::IsTerminal as _;

/// How much colour a destination takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ColourDepth {
    /// None. A pipe, `NO_COLOR`, or a terminal that says so.
    None,
    /// The sixteen ANSI colours.
    Ansi16,
    /// Two hundred and fifty-six.
    Ansi256,
    /// Twenty-four bit.
    TrueColour,
}

/// Which renderer to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    /// Decide from the destination.
    #[default]
    Auto,
    /// The in-place renderer, whatever detection says.
    Live,
    /// One line per event, whatever detection says.
    Plain,
    /// One JSON object per event.
    Json,
}

impl Mode {
    /// Reads the `MEOWCTL_OUTPUT` value.
    ///
    /// An unrecognised value falls back to detection rather than failing: a
    /// typo in an environment variable should not stop an apply halfway
    /// through, which is what `ParseMode` does too; see [R-TUI-044].
    #[must_use]
    pub fn parse(value: &str) -> Mode {
        match value.trim().to_ascii_lowercase().as_str() {
            "live" => Mode::Live,
            "plain" => Mode::Plain,
            "json" => Mode::Json,
            _ => Mode::Auto,
        }
    }
}

/// What the destination can do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Caps {
    /// Whether it is an interactive terminal.
    pub tty: bool,
    /// Whether the renderer may move the cursor.
    pub motion: bool,
    /// Whether the locale advertises UTF-8.
    pub unicode: bool,
    /// How much colour it takes.
    pub colour: ColourDepth,
    /// Its width in columns, when that is known.
    pub width: Option<u16>,
    /// Its height in rows, when that is known.
    ///
    /// Only the live region uses it, to cap itself to the viewport; see
    /// [R-TUI-020].
    pub height: Option<u16>,
}

impl Default for Caps {
    /// What a pipe can do, which is the safe assumption.
    fn default() -> Self {
        Caps {
            tty: false,
            motion: false,
            unicode: false,
            colour: ColourDepth::None,
            width: None,
            height: None,
        }
    }
}

/// How the environment is read, so a test needs no process-wide variables.
pub trait Env {
    /// A variable's value, or `None` when it is unset or empty.
    fn var(&self, key: &str) -> Option<String>;
}

/// The real environment.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemEnv;

impl Env for SystemEnv {
    fn var(&self, key: &str) -> Option<String> {
        std::env::var(key).ok().filter(|v| !v.is_empty())
    }
}

impl Caps {
    /// Resolves what standard output can do.
    #[must_use]
    pub fn detect(mode: Mode, env: &impl Env) -> Caps {
        let tty = std::io::stdout().is_terminal();
        let size = tty.then(terminal_size::terminal_size).flatten();
        Caps {
            height: size.map(|(_, h)| h.0),
            ..Caps::resolve(mode, env, tty, size.map(|(w, _)| w.0))
        }
    }

    /// Resolves from facts a test can supply.
    #[must_use]
    pub fn resolve(mode: Mode, env: &impl Env, tty: bool, width: Option<u16>) -> Caps {
        let term = env.var("TERM").unwrap_or_default();
        let dumb = term.is_empty() || term == "dumb";
        // Most providers capture stdout into a log, so motion is off even
        // when they hand out a pty; see [R-TUI-041].
        let ci = env.var("CI").is_some();

        let motion = match mode {
            Mode::Live => true,
            Mode::Plain | Mode::Json => false,
            Mode::Auto => tty && !dumb && !ci,
        };

        Caps {
            tty,
            motion,
            unicode: supports_unicode(env),
            colour: colour_depth(env, tty, dumb),
            width,
            height: None,
        }
    }

    /// The width now, so a resize is picked up without a signal handler; see
    /// [R-TUI-025].
    #[must_use]
    pub fn current_width(self) -> Option<u16> {
        if !self.tty {
            return self.width;
        }
        terminal_size::terminal_size()
            .map(|(w, _)| w.0)
            .or(self.width)
    }

    /// The height now, for capping the live region to the viewport.
    ///
    /// Re-read per frame, like the width; see [R-TUI-025].
    #[must_use]
    pub fn current_height(self) -> Option<u16> {
        if !self.tty {
            return self.height;
        }
        terminal_size::terminal_size()
            .map(|(_, h)| h.0)
            .or(self.height)
    }
}

/// Whether the locale advertises UTF-8.
///
/// The same three variables `supportsUnicode` reads, in the same order, and
/// the same fallback: a machine with no locale set is assumed modern on Unix
/// and assumed a code page on Windows; see [R-TUI-043].
fn supports_unicode(env: &impl Env) -> bool {
    for key in ["LC_ALL", "LC_CTYPE", "LANG"] {
        let Some(value) = env.var(key) else {
            continue;
        };
        let value = value.to_ascii_uppercase();
        return value.contains("UTF-8") || value.contains("UTF8");
    }
    cfg!(unix)
}

/// How much colour the destination takes.
///
/// `NO_COLOR` wins over everything, because that is what the convention is
/// for; `CLICOLOR_FORCE` turns it back on for a pipe, which is how a caller
/// says it is rendering the output itself; see [R-TUI-042].
fn colour_depth(env: &impl Env, tty: bool, dumb: bool) -> ColourDepth {
    if env.var("NO_COLOR").is_some() || dumb {
        return ColourDepth::None;
    }
    let forced = env.var("CLICOLOR_FORCE").is_some_and(|v| v != "0");
    if !tty && !forced {
        return ColourDepth::None;
    }

    let colorterm = env
        .var("COLORTERM")
        .unwrap_or_default()
        .to_ascii_lowercase();
    if colorterm.contains("truecolor") || colorterm.contains("24bit") {
        return ColourDepth::TrueColour;
    }
    let term = env.var("TERM").unwrap_or_default();
    if term.contains("256") {
        return ColourDepth::Ansi256;
    }
    ColourDepth::Ansi16
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    struct FakeEnv(HashMap<&'static str, &'static str>);

    impl FakeEnv {
        fn new(pairs: &[(&'static str, &'static str)]) -> Self {
            FakeEnv(pairs.iter().copied().collect())
        }
    }

    impl Env for FakeEnv {
        fn var(&self, key: &str) -> Option<String> {
            self.0
                .get(key)
                .map(|v| (*v).to_owned())
                .filter(|v| !v.is_empty())
        }
    }

    /// [R-TUI-041] cursor movement written into a CI transcript is
    /// unreadable, so motion is off there even with a pty.
    #[test]
    fn motion_is_off_in_ci_even_on_a_terminal() {
        let env = FakeEnv::new(&[("TERM", "xterm-256color"), ("CI", "true")]);
        assert!(!Caps::resolve(Mode::Auto, &env, true, Some(80)).motion);
    }

    /// [R-TUI-041] and off for a pipe, and off for a terminal that says it is
    /// dumb.
    #[test]
    fn motion_is_off_for_a_pipe_and_for_a_dumb_terminal() {
        let env = FakeEnv::new(&[("TERM", "xterm-256color")]);
        assert!(!Caps::resolve(Mode::Auto, &env, false, None).motion);
        assert!(Caps::resolve(Mode::Auto, &env, true, Some(80)).motion);

        let dumb = FakeEnv::new(&[("TERM", "dumb")]);
        assert!(!Caps::resolve(Mode::Auto, &dumb, true, Some(80)).motion);

        let unset = FakeEnv::new(&[]);
        assert!(!Caps::resolve(Mode::Auto, &unset, true, Some(80)).motion);
    }

    /// [R-TUI-042] the convention exists so a user can turn it off.
    #[test]
    fn no_color_wins_over_everything() {
        let env = FakeEnv::new(&[
            ("TERM", "xterm-256color"),
            ("COLORTERM", "truecolor"),
            ("NO_COLOR", "1"),
        ]);
        assert_eq!(
            Caps::resolve(Mode::Auto, &env, true, Some(80)).colour,
            ColourDepth::None
        );
    }

    /// [R-TUI-042] and [R-TUI-045]: the depth is what the terminal reports
    /// rather than what we hope, because truecolor on a 16-colour terminal is
    /// garbage.
    #[test]
    fn the_colour_depth_is_what_the_terminal_says() {
        let truecolor = FakeEnv::new(&[("TERM", "xterm-256color"), ("COLORTERM", "truecolor")]);
        assert_eq!(
            Caps::resolve(Mode::Auto, &truecolor, true, Some(80)).colour,
            ColourDepth::TrueColour
        );

        // The other spelling terminals use, and the reason the check is a
        // substring rather than an equality; see [R-TUI-045].
        let bits = FakeEnv::new(&[("TERM", "xterm-256color"), ("COLORTERM", "24bit")]);
        assert_eq!(
            Caps::resolve(Mode::Auto, &bits, true, Some(80)).colour,
            ColourDepth::TrueColour
        );

        let ansi256 = FakeEnv::new(&[("TERM", "xterm-256color")]);
        assert_eq!(
            Caps::resolve(Mode::Auto, &ansi256, true, Some(80)).colour,
            ColourDepth::Ansi256
        );

        let plain = FakeEnv::new(&[("TERM", "xterm")]);
        assert_eq!(
            Caps::resolve(Mode::Auto, &plain, true, Some(80)).colour,
            ColourDepth::Ansi16
        );
    }

    /// [R-TUI-046] a pipe takes no colour, unless the caller says it is
    /// rendering the output itself.
    #[test]
    fn a_pipe_takes_no_colour_unless_it_is_forced() {
        let env = FakeEnv::new(&[("TERM", "xterm-256color")]);
        assert_eq!(
            Caps::resolve(Mode::Auto, &env, false, None).colour,
            ColourDepth::None
        );

        let forced = FakeEnv::new(&[("TERM", "xterm-256color"), ("CLICOLOR_FORCE", "1")]);
        assert_eq!(
            Caps::resolve(Mode::Auto, &forced, false, None).colour,
            ColourDepth::Ansi256
        );
    }

    /// [R-TUI-046] `0` is the one value that means no, which is the
    /// convention: everything else, including the empty string, forces.
    #[test]
    fn clicolor_force_of_zero_forces_nothing() {
        let env = FakeEnv::new(&[("TERM", "xterm-256color"), ("CLICOLOR_FORCE", "0")]);
        assert_eq!(
            Caps::resolve(Mode::Auto, &env, false, None).colour,
            ColourDepth::None
        );
    }

    /// [R-TUI-046] and a user who turned colour off outranks a caller that
    /// says this pipe can take it.
    #[test]
    fn no_color_outranks_clicolor_force() {
        let env = FakeEnv::new(&[
            ("TERM", "xterm-256color"),
            ("CLICOLOR_FORCE", "1"),
            ("NO_COLOR", "1"),
        ]);
        assert_eq!(
            Caps::resolve(Mode::Auto, &env, false, None).colour,
            ColourDepth::None
        );
    }

    /// [R-TUI-043] replacement characters where the status glyph should be is
    /// the failure this prevents.
    #[test]
    fn unicode_follows_the_locale() {
        assert!(supports_unicode(&FakeEnv::new(&[("LANG", "en_US.UTF-8")])));
        assert!(supports_unicode(&FakeEnv::new(&[("LC_ALL", "C.utf8")])));
        assert!(!supports_unicode(&FakeEnv::new(&[("LANG", "C")])));
    }

    /// The first variable that is set wins, which is the order `supportsUnicode`
    /// reads them in.
    #[test]
    fn lc_all_wins_over_lang() {
        let env = FakeEnv::new(&[("LC_ALL", "C"), ("LANG", "en_US.UTF-8")]);
        assert!(!supports_unicode(&env));
    }

    /// [R-TUI-044] a typo in an environment variable should not stop an apply
    /// halfway through.
    #[test]
    fn an_unrecognised_output_mode_falls_back_to_detection() {
        assert_eq!(Mode::parse("live"), Mode::Live);
        assert_eq!(Mode::parse(" PLAIN "), Mode::Plain);
        assert_eq!(Mode::parse("json"), Mode::Json);
        assert_eq!(Mode::parse("liv"), Mode::Auto);
        assert_eq!(Mode::parse(""), Mode::Auto);
    }

    /// A forced mode overrides what the destination says, which is what makes
    /// a recording reproducible.
    #[test]
    fn a_forced_mode_overrides_detection() {
        let env = FakeEnv::new(&[("CI", "true")]);
        assert!(Caps::resolve(Mode::Live, &env, false, None).motion);

        let terminal = FakeEnv::new(&[("TERM", "xterm-256color")]);
        assert!(!Caps::resolve(Mode::Plain, &terminal, true, Some(80)).motion);
    }
}
