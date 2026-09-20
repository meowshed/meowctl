//! Giving the terminal back when the process is stopped.
//!
//! A signal arrives at the process, not at the sink, so this lives here: a
//! library that installed a handler would be taking a process-wide resource
//! its callers did not ask it to take; see [R-CLI-014].
//!
//! The work happens on a thread rather than in a signal handler. A handler
//! runs between any two instructions and may call only async-signal-safe
//! functions, which rules out most of the standard library; a thread woken by
//! a signal may do anything. `signal-hook`'s iterator is what turns one into
//! the other.

/// Watches for the signals that stop a process, and restores the cursor.
///
/// A machine that will not let us watch is one where the cursor stays hidden
/// after an interrupt, which is worth less than refusing to run at all, so a
/// failure here is ignored.
pub fn restore_cursor_on_signal() {
    #[cfg(unix)]
    {
        /// The sequence that puts the cursor back.
        ///
        /// Written rather than asked of the sink, because the sink belongs to
        /// the thread that was interrupted and taking its lock here would
        /// deadlock against it.
        const SHOW_CURSOR: &[u8] = b"\x1b[?25h";

        use signal_hook::consts::{SIGHUP, SIGINT, SIGTERM, SIGTSTP};
        use signal_hook::iterator::Signals;

        let Ok(mut signals) = Signals::new([SIGINT, SIGTERM, SIGHUP, SIGTSTP]) else {
            return;
        };
        std::thread::spawn(move || {
            for signal in &mut signals {
                use std::io::Write as _;
                let mut out = std::io::stdout();
                let _ = out.write_all(SHOW_CURSOR);
                let _ = out.flush();

                // Then die the way the shell expects: a process that swallows
                // its own interrupt is one nobody can stop.
                let _ = signal_hook::low_level::emulate_default_handler(signal);
            }
        });
    }
}
