//! Turning a signal into something the run can act on.
//!
//! A signal arrives at the process, not at the sink and not at the engine, so
//! this lives here: a library that installed a handler would be taking a
//! process-wide resource its callers did not ask it to take; see
//! [R-CLI-014].
//!
//! The work happens on a thread rather than in a signal handler. A handler
//! runs between any two instructions and may call only async-signal-safe
//! functions, which rules out most of the standard library; a thread woken by
//! a signal may do anything. `signal-hook`'s iterator is what turns one into
//! the other.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// The sequence that puts the cursor back.
///
/// Written rather than asked of the sink, because the sink belongs to the
/// thread that was interrupted and taking its lock here would deadlock
/// against it.
#[cfg(unix)]
const SHOW_CURSOR: &[u8] = b"\x1b[?25h";

/// Watches for the signals that stop a process.
///
/// The returned flag is set by the first interrupt and read by the engine
/// between components, which is what makes a run stop rather than die; see
/// [R-ENGINE-062]. A second interrupt stops the process at once, because a
/// user who has asked twice is not asking for a tidier stop; see
/// [R-CLI-053].
///
/// A machine that will not let us watch is one where an interrupt kills the
/// process outright and the cursor stays hidden, which is worth less than
/// refusing to run at all, so a failure here is ignored.
#[must_use]
pub fn watch_for_interruption() -> Arc<AtomicBool> {
    let interrupted = Arc::new(AtomicBool::new(false));

    #[cfg(unix)]
    {
        use signal_hook::consts::{SIGHUP, SIGINT, SIGTERM, SIGTSTP};
        use signal_hook::iterator::Signals;

        let Ok(mut signals) = Signals::new([SIGINT, SIGTERM, SIGHUP, SIGTSTP]) else {
            return interrupted;
        };
        let flag = Arc::clone(&interrupted);
        std::thread::spawn(move || {
            for signal in &mut signals {
                restore_cursor();

                // Asking the run to stop is only meaningful for an interrupt,
                // and only the first one. A suspend has to suspend, a
                // terminate has to terminate, and a second interrupt has to
                // work whatever the run is doing.
                if signal == SIGINT && !flag.swap(true, Ordering::Relaxed) {
                    continue;
                }

                // Then die the way the shell expects: a process that swallows
                // its own interrupt is one nobody can stop.
                let _ = signal_hook::low_level::emulate_default_handler(signal);
            }
        });
    }

    interrupted
}

/// Puts the cursor back where the live region hid it.
#[cfg(unix)]
fn restore_cursor() {
    use std::io::Write as _;
    let mut out = std::io::stdout();
    let _ = out.write_all(SHOW_CURSOR);
    let _ = out.flush();
}
