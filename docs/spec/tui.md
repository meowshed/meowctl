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

**[R-TUI-010]** Four sinks MUST consume the same stream: `LiveSink` for a
capable terminal, `PlainSink` for everything else, `JsonSink` for
`--format json`, and `ShellSink` for `meowctl hook`.

An earlier draft named three. `ShellSink` was missed because `hook` was the
last command to be written, and it is not a rendering of a run at all: its
output is evaluated by a shell.

**[R-TUI-011]** A sink MUST be chosen once per command, from the detected
capabilities and the flags, and MUST NOT change mid-run.

**[R-TUI-012]** Every sink that renders a run MUST render every event. A sink
that silently drops a variant makes a command's output depend on where it
runs.

`ShellSink` is the exception and is not a rendering of a run. It MUST write
each `ShellLine` verbatim, with no decoration, no indent and no colour, and
MUST write nothing for any other event, because whatever it wrote the shell
would evaluate; see [R-CLI-061] and [R-COMMON-044].

**[R-TUI-013]** `ShellSink` MUST NOT be selectable by a flag. `hook` chooses
it, and `--format json` on `hook` MUST still produce the event stream, because
a program reading events is not a shell evaluating them.

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

The sink MUST restore it when it is finished with and when it is dropped,
which covers a normal exit and a panic. A signal arrives at the process rather
than at the sink, so `meowctl-cli` installs the handler and calls the sink;
see [R-CLI-014].

**[R-TUI-024]** Sub-work within a component, such as packages being installed,
MUST be shown on the component's own status line rather than by indenting
further, so [R-TUI-002] holds.

**[R-TUI-025]** The terminal width MUST be re-read per frame, so a resize is
picked up without a signal handler. `Caps.Size` does this.

**[R-TUI-026]** The spinner MUST advance on the events the sink receives, and
MUST NOT be driven by a timer. `v0.1.0` animates it from a goroutine, which
means a renderer owns a thread, a mutex and a shutdown path, and a test of the
output has to wait for wall-clock time to pass.

The cost is real and bounded: a component whose subprocess runs silently for a
minute shows a still spinner for that minute. Captured output and every other
event redraw it, so the case is a command that produces nothing at all.

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

**[R-TUI-045]** `COLORTERM` naming `truecolor` or `24bit` MUST select the
24-bit depth, and a `TERM` containing `256` the 256-colour one. Anything else
MUST be the 16-colour depth.

These two variables are how a terminal reports what it takes, which is the
"reports" in [R-TUI-042]. Naming them here rather than leaving the rule
abstract: a reader whose theme looks wrong needs to know which variable to
check, and a `COLORTERM` this misreads is indistinguishable from a palette
that is simply ugly.

**[R-TUI-046]** `CLICOLOR_FORCE`, set to anything but `0`, MUST turn colour
back on for a destination that is not a terminal. `NO_COLOR` MUST still win
over it.

A caller that renders meowctl's output itself -- a CI log viewer, a pager
invoked deliberately -- has no way to say so otherwise, because from here it
looks exactly like a pipe to a file. The precedence is the convention's:
`NO_COLOR` is a user saying they do not want colour anywhere, and that
outranks a caller saying this particular pipe can take it.

**[R-TUI-043]** A non-UTF-8 locale MUST select the ASCII glyph set, and the
ASCII set MUST carry the same distinctions as the Unicode one.

**[R-TUI-044]** `MEOWCTL_OUTPUT` MUST override detection with `live` or
`plain`, and an unrecognised value MUST fall back to detection rather than
erroring. `ParseMode` does this, and failing mid-apply over a typo in an
environment variable is the behaviour to avoid.

## The theme

**[R-TUI-050]** The palette MUST be data rather than colours at call sites,
with the Catppuccin values as the built-in default, and a user MUST be able to
point at their own.

**[R-TUI-051]** Call sites MUST name a role, never a colour, so the palette
changes in one place.

**[R-TUI-052]** A theme file that is malformed MUST warn and fall back to the
default, not fail the command. Nobody's apply should stop because their
colours are wrong.

**[R-TUI-053]** The theme file MUST be `theme.toml` in the configuration
directory, and MUST be a table per role naming `r`, `g`, `b` and `ansi16`.

```toml
[accent]
r = 203
g = 166
b = 247
ansi16 = 35
```

**[R-TUI-054]** A role the file does not name MUST keep its default, so a user
who wants one colour changed writes one table. A role it names MUST be
replaced whole: a partial colour is four numbers with one missing, and
guessing which default to mix in produces a colour nobody chose.

**[R-TUI-055]** Parsing MUST NOT read a file. `meowctl-tui` depends on
`meowctl-common` and on nothing else in the workspace, so it takes the text
and the caller brings it; see [R-CLI-010].

**[R-TUI-056]** The theme MUST be read once, before the sink is built, and a
read that fails for any reason -- absent, unreadable, malformed -- MUST leave
the default in place. Absence MUST be silent and the other two MUST warn:
almost nobody has this file, and a warning on every command for a file the
user never wrote is noise.

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

**[R-TUI-063]** The trait MUST also ask a free-text question and return the
answer, because `ctx.prompt` returns a string; see [R-CTX-026]. It MUST send
the question to stderr for the same reason a confirmation does, and MUST treat
end-of-input as an empty answer, which is what `starPrompt` does.

Two methods rather than a confirmation built on the free-text one: a
confirmation has a default and a fixed vocabulary, and the sink that renders it
and the session that cannot answer it both need to know which was asked.

## Failure paths

**[R-TUI-070]** A write to the output destination that fails MUST NOT panic and
MUST NOT abort a run in progress. A closed pipe is normal when output is piped
into `head`.

**[R-TUI-071]** Adding a variant to `Event` MUST NOT compile until every sink
that renders a run handles it.

This replaces a requirement that asked for an unrecognised event to be
rendered as something rather than dropped. Nothing can be unrecognised:
`Event` is not `#[non_exhaustive]` and every sink matches it exhaustively, so
a new variant is a compile error in each one. That is stronger than the
requirement asked for and leaves it nothing to test, which is how it was
found -- it was one of the requirements `/verify` reported with no test, and
the reason turned out to be that there was no test to write.

`ShellSink` is outside this, as [R-TUI-012] says: it renders one variant on
purpose and is not a rendering of a run.

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
