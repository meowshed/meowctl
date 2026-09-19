# Terminal output

**Crate:** `meowctl-tui`
**Design:** `docs/design/0.2.0-rust-rewrite.md` §4
**v0.1.0 equivalent:** `internal/tui/`

## Scope

This component renders. It consumes the [`Event`](common.md) stream and knows
nothing about the engine that produces it, which is what lets it be tested
against a fixture stream with no terminal and no subprocess.

Terminal output is the one carve-out from the parity constraint. The vocabulary
carries over unchanged, the architecture does not; §4 of the design document
says why.

## Boundary

The three sinks, what each renders, the theme, capability detection, and the
`Interaction` trait.

## The design system

**[R-TUI-001]** One glyph MUST mean the same thing in every command. A
component that installed, a check that passed, and a dependency that resolved
carry the same success mark.

**[R-TUI-002]** Output MUST use at most two levels of indent: column zero for
the command speaking about itself, two spaces for an item, and captured
subprocess output indented under its item and dimmed.

**[R-TUI-003]** Colour MUST be redundant. Every state MUST be legible from its
symbol and its wording alone, so losing colour loses decoration and never
information.

**[R-TUI-004]** No sink MUST put the terminal into raw mode. Hooks shell out to
commands that need the real terminal, and the renderer's job is to stand down
rather than to own it.

## Sinks

**[R-TUI-010]** Three sinks MUST consume the same stream: `LiveSink` for a
capable terminal, `PlainSink` for everything else, and `JsonSink` for
`--format json`.

**[R-TUI-011]** A sink MUST be chosen once per command, from the detected
capabilities and the flags, and MUST NOT change mid-run.

**[R-TUI-012]** Every sink MUST render every event. A sink that silently drops
a variant makes a command's output depend on where it runs.

## The live sink

**[R-TUI-020]** `LiveSink` MUST redraw its region in place and MUST erase it
before writing anything permanent, issuing exactly one cursor-up per drawn
line. `internal/tui/live.go` is the behaviour, including capping the region to
the viewport so the arithmetic cannot exceed it.

**[R-TUI-021]** Captured subprocess output MUST appear under its component
while the process runs.

**[R-TUI-022]** On `TerminalRequested`, `LiveSink` MUST erase its region,
restore the cursor, and write nothing until `TerminalReleased`; see
[R-EXEC-021].

**[R-TUI-023]** The cursor MUST be restored on exit, on interruption, and on
suspension. A command that exits with a hidden cursor leaves the user's shell
broken.

**[R-TUI-024]** Sub-work within a component, such as packages being installed,
MUST be shown on the component's own status line rather than by indenting
further, so [R-TUI-002] holds.

**[R-TUI-025]** The terminal width MUST be re-read per frame, so a resize is
picked up without a signal handler. `Caps.Size` does this.

## The JSON sink

**[R-TUI-030]** `JsonSink` MUST emit one JSON object per event, one per line,
in the order the events arrived.

**[R-TUI-031]** `JsonSink` MUST emit no colour, no glyph, and no padding. It is
the interface another program reads; see [R-COMMON-043].

**[R-TUI-032]** Every command MUST support `--format json`. In `v0.1.0` only
`doctor` has a JSON form, hand-written; here it falls out of the sink.

## Capability detection

**[R-TUI-040]** Detection MUST resolve four things independently: whether the
destination is a terminal, whether motion is allowed, whether the locale
advertises UTF-8, and the colour depth.

**[R-TUI-041]** Motion MUST be disabled for a pipe, for `TERM` unset or `dumb`,
and when `CI` is set, even when a pty is present. `DetectCaps` is deliberately
conservative here, and cursor movement written into a CI transcript is
unreadable.

**[R-TUI-042]** `NO_COLOR` MUST disable colour, and colour MUST be downsampled
to the depth the terminal reports rather than assumed.

**[R-TUI-043]** A non-UTF-8 locale MUST select the ASCII glyph set, and the
ASCII set MUST carry the same distinctions as the Unicode one.

**[R-TUI-044]** `MEOWCTL_OUTPUT` MUST override detection with `live` or
`plain`, and an unrecognised value MUST fall back to detection rather than
erroring. `ParseMode` does this, and failing mid-apply over a typo in an
environment variable is the behaviour to avoid.

## The theme

**[R-TUI-050]** The palette MUST be loaded as data, with the Catppuccin values
as the built-in default. A user MUST be able to point at their own.

**[R-TUI-051]** Call sites MUST name a role, never a colour, so the palette
changes in one place.

**[R-TUI-052]** A theme file that is malformed MUST warn and fall back to the
default, not fail the command. Nobody's apply should stop because their colours
are wrong.

## Interaction

**[R-TUI-060]** Prompting MUST go through an `Interaction` trait, separate from
rendering.

**[R-TUI-061]** A confirmation MUST send its question to stderr, MUST treat a
bare Enter and end-of-input as no, and MUST treat only an explicit yes as yes.
`Printer.Confirm` is this behaviour, and it exists because `fmt.Scanln` failed
the command on an empty line.

**[R-TUI-062]** In a non-interactive session, a prompt MUST fail with a message
naming what it wanted, rather than blocking. A CI run that hangs on a prompt
until it times out is the failure to prevent.

## Failure paths

**[R-TUI-070]** A write to the output destination that fails MUST NOT panic and
MUST NOT abort a run in progress. A closed pipe is normal when output is piped
into `head`.

**[R-TUI-071]** An event a sink does not recognise MUST be rendered as
something rather than dropped, so a sink built against an older event set
degrades instead of hiding work.

## Verification

**[R-TUI-080]** Every sink MUST be testable against a recorded event stream
with no terminal, no subprocess, and no filesystem.

**[R-TUI-081]** Snapshots MUST cover the capability matrix: motion, colour
depth, glyph tier, and width. `v0.1.0`'s tests check a few cases by hand, which
is why the matrix is stated here as an obligation.

## Parity with v0.1.0

The three design-system rules, the never-raw-mode rule, the detection
conservatism, the `MEOWCTL_OUTPUT` override, the confirmation semantics, and
the region-erasure arithmetic all come from `internal/tui/` and carry over.

The architecture does not carry over, and this is the one component where that
is intended. `Writer` and `Printer` become sinks over one event stream,
`Writer.Log` is replaced by typed events, `JsonSink` is new, prompting leaves
the printer, and the theme becomes data. §4 of the design document holds the
reasoning.
