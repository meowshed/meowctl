package tui

import "fmt"

// The design system.
//
// Three ideas hold the output together:
//
//  1. One symbol vocabulary. A glyph means the same thing in every command:
//     a component that installed, a check that passed and a dep that resolved
//     all carry the same mark. Before this package there were four competing
//     vocabularies (ok/FAIL, ->/--, +/-, ✓).
//
//  2. Two levels of indent, never more. Column 0 is the command talking about
//     itself; two spaces is one item; captured subprocess output is indented
//     under its item and dimmed. Nesting deeper stops being scannable.
//
//  3. Colour is redundant. Every state is legible from its symbol and wording
//     alone, so a monochrome terminal, NO_COLOR, or a pipe loses decoration
//     but never information.
//
// Colours are written as truecolor SGR and downsampled by colorprofile to
// whatever the terminal actually supports, down to being stripped entirely.
// The palette is Catppuccin, matching the rest of this configuration.

// Symbols is the glyph set for one capability tier.
type Symbols struct {
	Success string
	Failure string
	Warning string
	Info    string
	Pending string
	Skipped string
	Running string
	Added   string
	Removed string
	Arrow   string
	Bullet  string
	Spinner []string
}

var unicodeSymbols = Symbols{
	Success: "✓",
	Failure: "✗",
	Warning: "!",
	Info:    "i",
	Pending: "·",
	Skipped: "–",
	Running: "▸",
	Added:   "+",
	Removed: "−",
	Arrow:   "→",
	Bullet:  "·",
	Spinner: []string{"⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"},
}

var asciiSymbols = Symbols{
	Success: "+",
	Failure: "x",
	Warning: "!",
	Info:    "i",
	Pending: ".",
	Skipped: "-",
	Running: ">",
	Added:   "+",
	Removed: "-",
	Arrow:   "->",
	Bullet:  "*",
	Spinner: []string{"|", "/", "-", "\\"},
}

// Colour roles. Semantic names only — call sites never name a colour, so the
// palette can change in one place.
const (
	sgrReset  = "\x1b[0m"
	fgSuccess = "\x1b[38;2;166;227;161m" // green
	fgFailure = "\x1b[38;2;243;139;168m" // red
	fgWarning = "\x1b[38;2;249;226;175m" // yellow
	fgAccent  = "\x1b[38;2;203;166;247m" // mauve
	fgInfo    = "\x1b[38;2;137;180;250m" // blue
	fgMuted   = "\x1b[38;2;127;132;156m" // overlay
	sgrBold   = "\x1b[1m"
)

// Theme renders styled fragments for a given capability set.
type Theme struct {
	sym   Symbols
	color bool
}

// NewTheme builds the theme matching caps.
func NewTheme(caps Caps) Theme {
	sym := asciiSymbols
	if caps.Unicode {
		sym = unicodeSymbols
	}
	return Theme{sym: sym, color: caps.Color()}
}

// Symbols exposes the active glyph set.
func (t Theme) Symbols() Symbols { return t.sym }

// paint wraps s in an SGR sequence when colour is available.
func (t Theme) paint(sgr, s string) string {
	if !t.color || s == "" {
		return s
	}
	return sgr + s + sgrReset
}

// Success paints s in the success role.
func (t Theme) Success(s string) string { return t.paint(fgSuccess, s) }

// Failure paints s in the failure role.
func (t Theme) Failure(s string) string { return t.paint(fgFailure, s) }

// Warning paints s in the warning role.
func (t Theme) Warning(s string) string { return t.paint(fgWarning, s) }

// Accent paints s in the accent role, used for in-progress work.
func (t Theme) Accent(s string) string { return t.paint(fgAccent, s) }

// Info paints s in the informational role.
func (t Theme) Info(s string) string { return t.paint(fgInfo, s) }

// Muted paints s in the de-emphasised role, used for notes and detail.
func (t Theme) Muted(s string) string { return t.paint(fgMuted, s) }

// Bold emboldens s.
func (t Theme) Bold(s string) string { return t.paint(sgrBold, s) }

// Mark renders a status glyph in its role colour.
func (t Theme) Mark(kind Status) string {
	switch kind {
	case StatusSuccess:
		return t.Success(t.sym.Success)
	case StatusFailure:
		return t.Failure(t.sym.Failure)
	case StatusWarning:
		return t.Warning(t.sym.Warning)
	case StatusSkipped:
		return t.Muted(t.sym.Skipped)
	case StatusRunning:
		return t.Accent(t.sym.Running)
	case StatusPending:
		return t.Muted(t.sym.Pending)
	default:
		return t.Info(t.sym.Info)
	}
}

// Status is the state of one reported item.
type Status int

// The states an item can report. Each has a distinct glyph in both symbol
// sets, so colour is never load-bearing.
const (
	StatusInfo Status = iota
	StatusSuccess
	StatusFailure
	StatusWarning
	StatusSkipped
	StatusRunning
	StatusPending
)

// Item renders one indented item line: two spaces, a mark, the label, and an
// optional dimmed note. No trailing newline.
func (t Theme) Item(kind Status, label, note string) string {
	line := "  " + t.Mark(kind) + " " + label
	if note != "" {
		line += "  " + t.Muted(note)
	}
	return line
}

// Heading renders a command-level heading at column 0.
func (t Theme) Heading(text string) string {
	return t.Bold(text)
}

// Detail renders captured subprocess output indented under its item and
// dimmed, so a wall of brew output never competes with meowctl's own lines.
func (t Theme) Detail(text string) string {
	return t.Muted("      " + text)
}

// Countf renders a summary count line at column 0.
func (t Theme) Countf(format string, args ...any) string {
	return t.Muted(fmt.Sprintf(format, args...))
}
