package tui

import (
	"fmt"
	"io"
	"os"
	"strings"
	"text/tabwriter"

	"github.com/charmbracelet/colorprofile"
)

// Printer renders command output — everything that is not lifecycle progress.
//
// It exists so no command formats its own line. Before this, output carried
// three competing prefixes ("meowctl: ", two spaces, nothing at all), mixed
// sentence case with lowercase, and sent warnings to stdout where they
// corrupted piped output. Those are formatting decisions, and formatting
// decisions belong in one place.
//
// Routing follows the Unix convention: results on stdout, diagnostics on
// stderr. That is what makes `meowctl dep list | grep` behave.
type Printer struct {
	out   io.Writer
	err   io.Writer
	theme Theme
	caps  Caps
}

// NewPrinter builds a Printer for the given streams. Either may be nil, in
// which case os.Stdout / os.Stderr are used.
func NewPrinter(out, errOut io.Writer) *Printer {
	if out == nil {
		out = os.Stdout
	}
	if errOut == nil {
		errOut = os.Stderr
	}
	caps := DetectCaps(out, os.Environ(), ModeAuto)
	return NewPrinterWithCaps(out, errOut, caps)
}

// NewPrinterWithCaps builds a Printer with pre-detected capabilities.
func NewPrinterWithCaps(out, errOut io.Writer, caps Caps) *Printer {
	if out == nil {
		out = os.Stdout
	}
	if errOut == nil {
		errOut = os.Stderr
	}
	return &Printer{
		out:   &colorprofile.Writer{Forward: out, Profile: caps.Profile},
		err:   &colorprofile.Writer{Forward: errOut, Profile: caps.Profile},
		theme: caps.theme(),
		caps:  caps,
	}
}

// Theme exposes the printer's theme for callers that build their own lines.
func (p *Printer) Theme() Theme { return p.theme }

func (p *Printer) writeln(w io.Writer, s string) {
	_, _ = io.WriteString(w, s+"\n")
}

// Heading prints a section heading at column 0. Headings name what the command
// is doing, in the imperative present, with no trailing punctuation.
func (p *Printer) Heading(format string, args ...any) {
	p.writeln(p.out, p.theme.Heading(fmt.Sprintf(format, args...)))
}

// Item prints one indented result line.
func (p *Printer) Item(kind Status, label, note string) {
	p.writeln(p.out, p.theme.Item(kind, label, note))
}

// Success prints an item that completed.
func (p *Printer) Success(label, note string) { p.Item(StatusSuccess, label, note) }

// Failure prints an item that failed.
func (p *Printer) Failure(label, note string) { p.Item(StatusFailure, label, note) }

// Skipped prints an item that was deliberately not acted on.
func (p *Printer) Skipped(label, note string) { p.Item(StatusSkipped, label, note) }

// Pending prints an item that is queued.
func (p *Printer) Pending(label, note string) { p.Item(StatusPending, label, note) }

// Added prints an addition line for plans and diffs.
func (p *Printer) Added(label string) {
	p.writeln(p.out, "  "+p.theme.Success(p.theme.sym.Added)+" "+label)
}

// Removed prints a removal line for plans and diffs.
func (p *Printer) Removed(label string) {
	p.writeln(p.out, "  "+p.theme.Failure(p.theme.sym.Removed)+" "+label)
}

// Change prints a transition line, e.g. a version bump.
func (p *Printer) Change(label, from, to string) {
	p.writeln(p.out, "  "+p.theme.Mark(StatusSuccess)+" "+label+"  "+
		p.theme.Muted(from+" "+p.theme.sym.Arrow+" "+to))
}

// Detail prints captured or supplementary output, indented under its item.
func (p *Printer) Detail(text string) {
	for _, line := range strings.Split(strings.TrimRight(text, "\n"), "\n") {
		p.writeln(p.out, p.theme.Detail(line))
	}
}

// Note prints a dimmed line at column 0 — counts, totals, and the "nothing
// happened" cases that are results rather than warnings.
func (p *Printer) Note(format string, args ...any) {
	p.writeln(p.out, p.theme.Countf(format, args...))
}

// Warn prints a warning to stderr. Warnings never go to stdout: they would
// corrupt output a caller is piping.
func (p *Printer) Warn(format string, args ...any) {
	p.writeln(p.err, "  "+p.theme.Mark(StatusWarning)+" "+fmt.Sprintf(format, args...))
}

// Error prints an error to stderr.
func (p *Printer) Error(format string, args ...any) {
	p.writeln(p.err, "  "+p.theme.Mark(StatusFailure)+" "+fmt.Sprintf(format, args...))
}

// Blank prints a separating blank line.
func (p *Printer) Blank() { p.writeln(p.out, "") }

// Table renders aligned columns. header may be nil for an unheaded table.
// Columns are separated by two spaces so the result stays greppable.
func (p *Printer) Table(header []string, rows [][]string) {
	tw := tabwriter.NewWriter(p.out, 0, 0, 2, ' ', 0)
	if len(header) > 0 {
		cells := make([]string, len(header))
		for i, h := range header {
			cells[i] = p.theme.Muted(strings.ToUpper(h))
		}
		_, _ = fmt.Fprintln(tw, strings.Join(cells, "\t"))
	}
	for _, row := range rows {
		_, _ = fmt.Fprintln(tw, strings.Join(row, "\t"))
	}
	_ = tw.Flush()
}

// KeyValue renders aligned label/value pairs, the shape used by status and
// version output.
func (p *Printer) KeyValue(pairs [][2]string) {
	width := 0
	for _, kv := range pairs {
		if n := len(kv[0]); n > width {
			width = n
		}
	}
	for _, kv := range pairs {
		p.writeln(p.out, p.theme.Muted(fmt.Sprintf("%-*s", width, kv[0]))+"  "+kv[1])
	}
}

// theme builds the Theme for these capabilities.
func (c Caps) theme() Theme { return NewTheme(c) }

// ItemSpec is one row for ItemList.
type ItemSpec struct {
	Status Status
	Label  string
	Note   string
}

// ItemList renders items with their labels padded to a common width, so the
// notes form a column. Use it whenever several items are printed together;
// ragged notes are markedly harder to scan than aligned ones.
func (p *Printer) ItemList(items []ItemSpec) {
	width := 0
	for _, it := range items {
		if n := len(it.Label); n > width {
			width = n
		}
	}
	for _, it := range items {
		label := it.Label
		if it.Note != "" {
			label = fmt.Sprintf("%-*s", width, label)
		}
		p.writeln(p.out, p.theme.Item(it.Status, label, it.Note))
	}
}

// Default is the process-wide Printer used by the package-level helpers.
// Commands that already hold a Printer should prefer it; these exist so a
// one-line message does not need to construct one.
var Default = NewPrinter(nil, nil)

// Note prints a dimmed status line via Default.
func Note(format string, args ...any) { Default.Note(format, args...) }

// Warn prints a warning to stderr via Default.
func Warn(format string, args ...any) { Default.Warn(format, args...) }

// Heading prints a section heading via Default.
func Heading(format string, args ...any) { Default.Heading(format, args...) }
