//! Shared vocabulary for the meowctl workspace.
//!
//! This crate sits at the bottom of the dependency graph. It depends on no
//! other workspace crate and performs no input or output, which is what lets
//! every layer above it name the same things: component identifiers, lifecycle
//! phases, module references, the error and exit-code taxonomy, and the
//! [`Event`] vocabulary the engine emits and the sinks render.
//!
//! The crate is deliberately empty for now. Its types arrive with the
//! specifications that describe them, one component at a time, so that nothing
//! here exists before a requirement in `docs/spec/` says what it must do.
//!
//! [`Event`]: https://github.com/meowshed/meowctl/blob/rust-rewrite/docs/design/0.2.0-rust-rewrite.md

#[cfg(test)]
mod tests {
    /// The workspace, the toolchain, and the test runner agree with each
    /// other. Until this crate has behaviour, that is the only thing worth
    /// asserting, and a suite that runs zero tests cannot prove it.
    #[test]
    fn the_test_harness_runs() {
        assert_eq!(2 + 2, 4);
    }
}
