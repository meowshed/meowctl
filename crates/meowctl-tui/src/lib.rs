//! Rendering what a command produced.
//!
//! This crate consumes the [`Event`] stream and knows nothing about the engine
//! that produces it, which is what lets the sinks be tested against a fixture
//! stream with no terminal, no subprocess, and no filesystem; see
//! [R-TUI-080].
//!
//! Terminal output is the one deliberate carve-out from the parity
//! constraint. The vocabulary carries over from `v0.1.0` unchanged — see
//! [`theme`] — and the architecture does not: `Writer` and `Printer` become
//! sinks over one stream, and `JsonSink` is new.
//!
//! [`Event`]: meowctl_common::Event

pub mod caps;
pub mod theme;

mod interaction;
mod live;
mod sink;

pub use caps::{Caps, ColourDepth, Env, Mode, SystemEnv};
pub use interaction::{Always, Interaction, InteractionError, Prompt};
pub use live::LiveSink;
pub use sink::{JsonSink, PlainSink, Sink};
pub use theme::{Palette, Role, Symbols, Theme};
