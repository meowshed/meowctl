//! Parsing, constructing, dispatching, and the exit code.

use std::collections::BTreeMap;
use std::io::Write as _;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::{Arc, Mutex};

use clap::{CommandFactory as _, Parser as _};
use meowctl_common::{Event, Level, Severity, paths};
use meowctl_config::Layout;
use meowctl_fs::FileSystem;
use meowctl_tui::{
    Caps, JsonSink, LiveSink, Mode, Palette, PlainSink, ShellSink, Sink, SystemEnv, Theme,
};

use crate::cli::{Cli, Command, Format};
use crate::{CliError, CliResult};

mod commands;
mod writing;

/// Parses, runs, and reports.
///
/// The binary's whole body: everything a process needs that a library must
/// not do happens here.
#[must_use]
pub fn main() -> ExitCode {
    let parsed = match Cli::try_parse() {
        Ok(parsed) => parsed,
        Err(error) => {
            // clap already renders usage for a usage error, and a help
            // request is a success; see [R-CLI-032] and [R-CLI-052].
            let _ = error.print();
            return if error.use_stderr() {
                ExitCode::from(Severity::Usage.exit_code())
            } else {
                ExitCode::SUCCESS
            };
        }
    };
    run(parsed)
}

/// Runs an already-parsed command line.
///
/// Separate from [`main`] so a test drives it without a process.
#[must_use]
pub fn run(cli: Cli) -> ExitCode {
    // Before anything is drawn, so an interrupt during the first frame still
    // puts the cursor back; see [R-CLI-014].
    let interrupted = crate::signals::watch_for_interruption();

    // Read before the sink is built, because the sink is chosen once and does
    // not change mid-run; see [R-TUI-011] and [R-TUI-056].
    let (palette, complaint) = read_palette(&cli);
    let mut sink = build_sink(&cli, palette);
    if let Some(complaint) = complaint {
        sink.handle(&Event::Message {
            level: Level::Warn,
            text: complaint,
        });
    }
    let outcome = dispatch(&cli, sink.as_mut(), interrupted);
    sink.finish();

    match outcome {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            report(&error);
            ExitCode::from(error.severity().exit_code())
        }
    }
}

/// Writes a failure to standard error.
///
/// To standard error so it cannot corrupt a piped standard output, and
/// without usage, because a wall of flags on top of a real error buries it;
/// see [R-CLI-032] and [R-CLI-033].
fn report(error: &CliError) {
    let mut stderr = std::io::stderr();
    match error.span() {
        Some(span) => {
            let _ = writeln!(stderr, "meowctl: {span}: {error}");
            if let Some(source) = &span.source_line {
                let _ = writeln!(stderr, "      {source}");
            }
        }
        None => {
            let _ = writeln!(stderr, "meowctl: {error}");
        }
    }
}

/// Chooses the sink, once, from the flags and what the terminal can take.
///
/// Once per command and never changed mid-run; see [R-TUI-011].
/// The palette to render with, and what to say about how it was reached.
///
/// Absence is silent: almost nobody has this file, and a warning on every
/// command for one the user never wrote is noise. Unreadable and malformed
/// both warn and fall back, because a user who wrote a theme and is not
/// seeing it needs to hear why; see [R-TUI-052] and [R-TUI-056].
fn read_palette(cli: &Cli) -> (Palette, Option<String>) {
    let Some(path) = theme_path(cli) else {
        return (meowctl_tui::theme::CATPPUCCIN, None);
    };
    let fs = meowctl_fs::RealFs;
    let bytes = match fs.read(&path) {
        Ok(bytes) => bytes,
        Err(meowctl_fs::FsError::NotFound { .. }) => {
            return (meowctl_tui::theme::CATPPUCCIN, None);
        }
        Err(e) => {
            return (
                meowctl_tui::theme::CATPPUCCIN,
                Some(format!("{}: {e}", path.display())),
            );
        }
    };

    match Palette::parse(&String::from_utf8_lossy(&bytes)) {
        Ok(palette) => (palette, None),
        Err(e) => (
            meowctl_tui::theme::CATPPUCCIN,
            Some(format!("{}: {e}", path.display())),
        ),
    }
}

/// Where the theme file is, when the configuration directory can be found.
fn theme_path(cli: &Cli) -> Option<PathBuf> {
    let root = match &cli.global.config {
        Some(given) => given.clone(),
        None => paths::config_dir(&paths::SystemEnv).ok()?,
    };
    Some(Layout::new(root).theme())
}

fn build_sink(cli: &Cli, palette: Palette) -> Box<dyn Sink> {
    let json = matches!(cli.global.format, Some(Format::Json)) || cli.command.wants_json();
    if json {
        return Box::new(JsonSink::new(Box::new(std::io::stdout())));
    }

    // `hook` writes shell code that a shell evaluates, so its sink writes
    // what the hooks emitted and nothing else. `--format json` above still
    // wins, because a program reading events is not a shell evaluating them;
    // see [R-TUI-013].
    if matches!(cli.command, Command::Hook { .. }) {
        return Box::new(ShellSink::new(Box::new(std::io::stdout())));
    }

    // A command whose standard output another program reads gets a sink that
    // writes nothing of its own there; see [R-CLI-021].
    let destination: Box<dyn std::io::Write> = if cli.command.stdout_is_an_interface() {
        Box::new(std::io::stderr())
    } else {
        Box::new(std::io::stdout())
    };

    let mode = match cli.global.format {
        Some(Format::Plain) => Mode::Plain,
        Some(Format::Json) => Mode::Json,
        Some(Format::Auto) | None => std::env::var("MEOWCTL_OUTPUT")
            .ok()
            .map_or(Mode::Auto, |value| Mode::parse(&value)),
    };
    let caps = Caps::detect(mode, &SystemEnv);
    let theme = Theme::with_palette(caps, palette);

    if caps.motion {
        Box::new(LiveSink::new(destination, theme))
    } else {
        Box::new(PlainSink::new(destination, theme))
    }
}

/// Everything a command needs that this crate constructs.
pub struct Session<'a> {
    /// Where the configuration is.
    pub layout: Layout,
    /// The filesystem every effect goes through.
    pub fs: Arc<dyn FileSystem + Send + Sync>,
    /// Where subprocesses go.
    pub exec: Arc<dyn meowctl_exec::Executor + Send + Sync>,
    /// Where requests go.
    pub http: Arc<dyn meowctl_net::Http + Send + Sync>,
    /// Where prompts go.
    pub interaction: Arc<Mutex<dyn meowctl_tui::Interaction + Send>>,
    /// Where events go.
    pub events: Arc<Mutex<dyn FnMut(Event) + Send>>,
    /// The machine.
    pub platform: meowctl_starlark::Platform,
    /// The process environment.
    pub environment: BTreeMap<String, String>,
    /// `$HOME`.
    pub home: PathBuf,
    /// The module cache.
    pub cache: PathBuf,
    /// Whether this run writes nothing.
    pub dry_run: bool,
    /// Whether the caller wants the detail.
    ///
    /// Read where the events are rendered: a `Debug` message is held back
    /// without it, which is what `--verbose` turns on.
    #[allow(dead_code, reason = "read by the renderer below, not by a command")]
    pub verbose: bool,
    /// The sink, for a command that renders something itself.
    pub sink: &'a mut dyn Sink,
    /// Set when the user has asked the run to stop; see [R-CLI-051].
    pub interrupted: Arc<std::sync::atomic::AtomicBool>,
}

impl std::fmt::Debug for Session<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session")
            .field("root", &self.layout.root())
            .field("dry_run", &self.dry_run)
            .finish_non_exhaustive()
    }
}

/// Constructs the effects and hands them to the command.
///
/// This is the only place a `FileSystem` or an `Executor` is constructed, and
/// `--dry-run` chooses which, rather than being passed down as a boolean; see
/// [R-CLI-010] and [R-CLI-011].
fn dispatch(
    cli: &Cli,
    sink: &mut dyn Sink,
    interrupted: Arc<std::sync::atomic::AtomicBool>,
) -> CliResult<()> {
    let environment: BTreeMap<String, String> = std::env::vars().collect();
    let env = paths::SystemEnv;
    let home = paths::home_dir(&env)?;
    let root = match &cli.global.config {
        Some(given) => given.clone(),
        None => paths::config_dir(&env)?,
    };
    let cache = paths::cache_dir(&env)?;
    let dry_run = cli.command.dry_run();

    let real = Arc::new(meowctl_fs::RealFs::new());
    let fs: Arc<dyn FileSystem + Send + Sync> = if dry_run {
        Arc::new(meowctl_fs::DryRunFs::new(Box::new(Arc::clone(&real))))
    } else {
        real
    };

    // Events reach the sink through a closure, so nothing below this crate
    // holds a renderer; see [R-ENGINE-051].
    let collected: Arc<Mutex<Vec<Event>>> = Arc::new(Mutex::new(Vec::new()));
    let recorder = Arc::clone(&collected);
    let verbose = cli.global.verbose;

    let mut session = Session {
        layout: Layout::new(root),
        fs,
        exec: Arc::new(meowctl_exec::RealExecutor::new()),
        http: Arc::new(meowctl_net::RealHttp::new()),
        interaction: Arc::new(Mutex::new(meowctl_tui::Prompt::new(
            std::io::BufReader::new(std::io::stdin()),
            std::io::stderr(),
            std::io::IsTerminal::is_terminal(&std::io::stdin()),
        ))),
        events: Arc::new(Mutex::new(move |event| {
            if let Ok(mut held) = recorder.lock() {
                held.push(event);
            }
        })),
        platform: meowctl_starlark::Platform::current(),
        environment,
        home,
        cache,
        dry_run,
        verbose,
        sink,
        interrupted,
    };

    let outcome = commands::run(cli, &mut session);

    // Everything the run produced, rendered in the order it happened.
    // Collected rather than streamed because a command may decide what to
    // render only once it knows how the whole thing went.
    let events = collected
        .lock()
        .map(|mut held| std::mem::take(&mut *held))
        .unwrap_or_default();
    for event in &events {
        if !verbose
            && matches!(
                event,
                Event::Message {
                    level: Level::Debug,
                    ..
                }
            )
        {
            continue;
        }
        session.sink.handle(event);
    }
    outcome
}

/// Writes the completion script for a shell.
pub(crate) fn completions(shell: clap_complete::Shell) {
    let mut command = Cli::command();
    clap_complete::generate(shell, &mut command, "meowctl", &mut std::io::stdout());
}
