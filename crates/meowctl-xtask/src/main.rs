//! Developer tasks for meowctl.
//!
//! The only task so far is the compatibility corpus, which is the oracle the
//! whole rewrite is verified against. `record` captures what the `v0.1.0` Go
//! binary does to a sandbox; `check` replays the same commands under the Rust
//! binary and diffs the results.
//!
//! Rendered output is deliberately not compared. Terminal output is the one
//! carve-out from the parity constraint, so the corpus compares exit codes, the
//! sandbox file tree, and the contents of every file meowctl wrote. The
//! exception is a command whose stdout is a machine interface: `shell` emits
//! code the shell evaluates, so its stdout is compared byte for byte.

// The workspace denies printing because output belongs to the sinks in
// `meowctl-tui`, which own the live region and would be corrupted by a stray
// write. Neither half of that applies here: xtask is a developer tool, it does
// not ship in the binary, and it has no renderer to corrupt. Printing is its
// entire interface.
#![allow(clippy::print_stdout, clippy::print_stderr)]

mod compat;

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let task: Vec<&str> = args.iter().map(String::as_str).collect();

    let result = match task.as_slice() {
        ["compat", "record"] => compat::record(),
        ["compat", "check"] => compat::check(),
        ["compat", "list"] => compat::list(),
        _ => {
            eprintln!("{USAGE}");
            return ExitCode::from(2);
        }
    };

    match result {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(e) => {
            eprintln!("xtask: {e:#}");
            ExitCode::FAILURE
        }
    }
}

const USAGE: &str = "\
usage: cargo xtask <task>

tasks:
  compat record   Run the v0.1.0 binary over the corpus and store the fixtures
  compat check    Run the Rust binary over the corpus and diff against them
  compat list     Show the cases and commands the corpus covers

environment:
  MEOWCTL_COMPAT_EXTRA   Path to an extra config directory to include. Copied
                         into the sandbox, never written to in place.";
