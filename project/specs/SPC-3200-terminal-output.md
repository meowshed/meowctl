---
id: SPC-3200
artifact: spec
status: live
revised: 2026-09-27
checked-at:
states: [REQ-3201, REQ-3202, REQ-3203, REQ-3204, REQ-3210, REQ-3211, REQ-3212, REQ-3213, REQ-3220, REQ-3221, REQ-3222, REQ-3223, REQ-3224, REQ-3225, REQ-3226, REQ-3230, REQ-3231, REQ-3232, REQ-3240, REQ-3241, REQ-3242, REQ-3243, REQ-3244, REQ-3245, REQ-3246, REQ-3250, REQ-3251, REQ-3252, REQ-3253, REQ-3254, REQ-3255, REQ-3256, REQ-3260, REQ-3261, REQ-3262, REQ-3263, REQ-3270, REQ-3271, REQ-3280, REQ-3281, REQ-3300, REQ-3301, REQ-3302, REQ-3303, REQ-3304, REQ-3305, REQ-3306, REQ-3307, REQ-3308, REQ-3309, REQ-3310, REQ-3311, REQ-3312, REQ-3313, REQ-3314, REQ-3315, REQ-3316, REQ-3317, REQ-3318, REQ-3319, REQ-3320, REQ-3321, REQ-3322, REQ-3323, REQ-3340, REQ-3341, REQ-3342, REQ-3370]
---

# Terminal output

## Scope

Terminal output lives in the `meowctl-tui` crate, its design is
`docs/design/0.2.0-rust-rewrite.md` §4, and its `v0.1.0` equivalent is
`internal/tui/` (from docs/spec/tui.md, high).

This component renders (from docs/spec/tui.md, high). It consumes the `Event`
stream and knows nothing about the engine that produces it, which is what lets
it be tested against a fixture stream with no terminal and no subprocess (from
docs/spec/tui.md, high). The event vocabulary belongs to the common
specification, and the engine that emits it to SPC-3000 (from docs/spec/tui.md,
high).

Terminal output is the one carve-out from the parity constraint: the vocabulary
carries over unchanged, and the architecture does not (from docs/spec/tui.md,
high).

## Boundary

The component exposes the four sinks, what each renders, the theme, capability
detection, and the `Interaction` trait (from docs/spec/tui.md, high).

| Surface | What is observable |
| --- | --- |
| `LiveSink` | A redrawn region on a capable terminal [REQ-3210] (from docs/spec/tui.md, high) |
| `PlainSink` | Output for every destination that is not a capable terminal [REQ-3210] (from docs/spec/tui.md, high) |
| `JsonSink` | One JSON object per event, one per line, for `--format json` [REQ-3230] (from docs/spec/tui.md, high) |
| `ShellSink` | Each `ShellLine` verbatim, for `meowctl hook` [REQ-3302] (from docs/spec/tui.md, high) |
| `MEOWCTL_OUTPUT`, `NO_COLOR`, `CLICOLOR_FORCE`, `COLORTERM`, `TERM`, `CI` | The environment variables detection reads [REQ-3241] [REQ-3242] [REQ-3244] [REQ-3245] [REQ-3246] (from docs/spec/tui.md, high) |
| `theme.toml` | The theme file in the configuration directory [REQ-3253] (from docs/spec/tui.md, high) |
| `Interaction` | Confirmations and free-text questions, asked on stderr [REQ-3260] [REQ-3261] [REQ-3321] (from docs/spec/tui.md, high) |

## Behaviour

### The design system

One glyph means the same thing in every command [REQ-3201] (from
docs/spec/tui.md, high).

Output uses at most two levels of indent: column zero for the command speaking
about itself, two spaces for an item, and captured subprocess output indented
under its item and dimmed [REQ-3202] (from docs/spec/tui.md, high).

Colour is redundant [REQ-3203]: every state is legible from its symbol and its
wording alone [REQ-3300] (from docs/spec/tui.md, high).

No sink puts the terminal into raw mode [REQ-3204] (from docs/spec/tui.md,
high).

### Sinks

Four sinks consume the same stream: `LiveSink` for a capable terminal,
`PlainSink` for everything else, `JsonSink` for `--format json`, and `ShellSink`
for `meowctl hook` [REQ-3210] (from docs/spec/tui.md, high).

The command chooses a sink once, from the detected capabilities and the flags
[REQ-3211], and the sink does not change mid-run [REQ-3301] (from
docs/spec/tui.md, high).

Every sink that renders a run renders every event [REQ-3212] (from
docs/spec/tui.md, high). `ShellSink` is not a rendering of a run: it writes each
`ShellLine` verbatim, with no decoration, no indent and no colour [REQ-3302],
and writes nothing for any other event [REQ-3303] (from docs/spec/tui.md, high).

No flag selects `ShellSink`, because `hook` chooses it [REQ-3213].
`--format json` on `hook` still produces the event stream [REQ-3304] (from
docs/spec/tui.md, high).

### The live sink

`LiveSink` redraws its region in place [REQ-3220], and erases it before writing
anything permanent, issuing exactly one cursor-up per drawn line [REQ-3305]
(from docs/spec/tui.md, high). It caps the region to the viewport, so the
arithmetic cannot exceed it [REQ-3305] (from docs/spec/tui.md, high).

Captured subprocess output appears under its component while the process runs
[REQ-3221] (from docs/spec/tui.md, high).

On `TerminalRequested`, `LiveSink` erases its region, restores the cursor, and
writes nothing until `TerminalReleased` [REQ-3222] (from docs/spec/tui.md,
high).

The cursor is restored on exit, on interruption and on suspension [REQ-3223]
(from docs/spec/tui.md, high). The sink restores it when it is finished with and
when it is dropped, which covers a normal exit and a panic [REQ-3306], and
`meowctl-cli` installs the signal handler and calls the sink (from
docs/spec/tui.md, high).

Sub-work within a component, such as packages being installed, shows on the
component's own status line, not on a further indent [REQ-3224] (from
docs/spec/tui.md, high).

The sink re-reads the terminal width per frame, so it picks up a resize without
a signal handler [REQ-3225] (from docs/spec/tui.md, high). No row it draws
exceeds that width [REQ-3370], because a wrapped row breaks the cursor-up
arithmetic of the next frame; a wide character counts as two columns (from
https://github.com/meowshed/meowctl/pull/71, high). The shipped sink counts
characters, which BUG-0021 records (from crates/meowctl-tui/src/live.rs:458-466,
medium).

The spinner advances on the events the sink receives [REQ-3226], and no timer
drives it [REQ-3307] (from docs/spec/tui.md, high).

### The JSON sink

`JsonSink` emits one JSON object per event, one per line, in the order the
events arrived [REQ-3230], with no colour, no glyph and no padding [REQ-3231]
(from docs/spec/tui.md, high).

Every command supports `--format json` [REQ-3232] (from docs/spec/tui.md, high).

### Capability detection

Detection resolves four things independently: whether the destination is a
terminal, whether motion is allowed, whether the locale advertises UTF-8, and
the colour depth [REQ-3240] (from docs/spec/tui.md, high).

Motion is disabled for a pipe, for `TERM` unset or `dumb`, and when `CI` is set,
even when a pty is present [REQ-3241] (from docs/spec/tui.md, high).

`NO_COLOR` disables colour [REQ-3242], and colour is downsampled to the depth
the terminal reports [REQ-3308] (from docs/spec/tui.md, high).

`COLORTERM` naming `truecolor` or `24bit` selects the 24-bit depth, and a `TERM`
containing `256` the 256-colour one [REQ-3245]. Anything else is the 16-colour
depth [REQ-3311] (from docs/spec/tui.md, high).

`CLICOLOR_FORCE`, set to anything but `0`, turns colour back on for a
destination that is not a terminal [REQ-3246], and `NO_COLOR` still wins over it
[REQ-3312] (from docs/spec/tui.md, high).

A non-UTF-8 locale selects the ASCII glyph set [REQ-3243], and the ASCII set
carries the same distinctions as the Unicode one [REQ-3309] (from
docs/spec/tui.md, high).

`MEOWCTL_OUTPUT` overrides detection with `live` or `plain` [REQ-3244] (from
docs/spec/tui.md, high).

### The theme

The palette is data, with the Catppuccin values as the built-in default
[REQ-3250], and a user can point at their own [REQ-3313] (from docs/spec/tui.md,
high).

Call sites name a role, never a colour, so the palette changes in one place
[REQ-3251] (from docs/spec/tui.md, high).

The theme file is `theme.toml` in the configuration directory [REQ-3253], with a
table per role naming `r`, `g`, `b` and `ansi16` [REQ-3314] (from
docs/spec/tui.md, high):

```toml
[accent]
r = 203
g = 166
b = 247
ansi16 = 35
```

A role the file does not name keeps its default [REQ-3254], and a role it names
is replaced whole [REQ-3315] (from docs/spec/tui.md, high).

Theme parsing reads no file: it takes the text, and the caller brings it
[REQ-3255] (from docs/spec/tui.md, high).

The theme is read once, before the sink is built [REQ-3256] (from
docs/spec/tui.md, high).

### Interaction

Prompting goes through an `Interaction` trait, separate from rendering
[REQ-3260] (from docs/spec/tui.md, high).

A confirmation sends its question to stderr [REQ-3261], treats a bare Enter and
end-of-input as no [REQ-3319], and treats only an explicit yes as yes [REQ-3320]
(from docs/spec/tui.md, high).

The trait also asks a free-text question and returns the answer [REQ-3263]. It
sends that question to stderr [REQ-3321] and treats end-of-input as an empty
answer [REQ-3322] (from docs/spec/tui.md, high).

### Verification

Every sink is testable against a recorded event stream with no terminal, no
subprocess and no filesystem [REQ-3280] (from docs/spec/tui.md, high).

Snapshots cover the capability matrix: motion, colour depth, glyph tier and
width [REQ-3281] (from docs/spec/tui.md, high).

Adding a variant to `Event` does not compile until every sink that renders a run
handles it [REQ-3271] (from docs/spec/tui.md, high).

## Failure paths

| Condition | What happens |
| --- | --- |
| A write to the output destination fails, such as a closed pipe into `head` | The sink does not panic [REQ-3270] and does not abort the run in progress [REQ-3323] (from docs/spec/tui.md, high). |
| `MEOWCTL_OUTPUT` holds an unrecognised value | Detection decides the sink, and nothing errors [REQ-3310] (from docs/spec/tui.md, high). |
| The theme file is absent | The default palette stays in place [REQ-3316], and nothing is printed [REQ-3317] (from docs/spec/tui.md, high). |
| The theme file is unreadable or malformed | The command warns [REQ-3318] and falls back to the default palette [REQ-3252] [REQ-3316], and does not fail (from docs/spec/tui.md, high). |
| A prompt runs in a non-interactive session | The prompt fails with a message naming what it wanted, and does not block [REQ-3262] (from docs/spec/tui.md, high). |
| The question can't be written to stderr | The prompt fails without reading input, naming the question [REQ-3262] [REQ-3341] (from crates/meowctl-tui/src/interaction.rs:81-137, high). |
| stderr isn't a terminal | The prompt fails as a non-interactive prompt does [REQ-3262] [REQ-3341]. |
| The terminal reports zero width | The live sink keeps the last known width [REQ-3342] (from crates/meowctl-tui/src/caps.rs:143-150, high). |
| The terminal is narrower than the minimum width | The live sink stops redrawing in place and prints lines as the plain sink does, until the terminal widens [REQ-3340]. |
| A confirmation reads a bare Enter or end-of-input | The answer is no [REQ-3319] (from docs/spec/tui.md, high). |
| A free-text question reads end-of-input | The answer is empty [REQ-3322] (from docs/spec/tui.md, high). |
| A new `Event` variant has no arm in a sink that renders a run | The build fails [REQ-3271] (from docs/spec/tui.md, high). |
