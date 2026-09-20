//! The binary.
//!
//! Everything it does is in [`meowctl_cli`]; this is here so the crate that
//! holds the command surface is a library a test can drive without a
//! process.

fn main() -> std::process::ExitCode {
    meowctl_cli::main()
}
