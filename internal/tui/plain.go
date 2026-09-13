package tui

import (
	"fmt"
	"io"
	"os"
	"strings"
	"sync"

	"github.com/charmbracelet/colorprofile"
)

// PlainWriter writes one line per event and never moves the cursor. It is the
// renderer for pipes, files, CI, dumb terminals, and MEOWCTL_OUTPUT=plain.
//
// It shares the theme with LiveWriter, so the symbol vocabulary is identical
// across renderers; only motion is dropped. Colour still applies when the
// destination supports it — a pager with -R is not a TTY but renders SGR
// fine — and colorprofile strips it when it does not.
type PlainWriter struct {
	mu      sync.Mutex
	out     io.Writer
	theme   Theme
	caps    Caps
	skipped int
}

// NewPlainWriter returns a PlainWriter writing to out. When out is nil,
// os.Stdout is used. Prefer New in production code; use this directly in
// tests that want deterministic, motion-free output.
func NewPlainWriter(out io.Writer) *PlainWriter {
	if out == nil {
		out = os.Stdout
	}
	return NewPlainWriterWithCaps(out, DetectCaps(out, os.Environ(), ModePlain))
}

// NewPlainWriterWithCaps builds a PlainWriter with pre-detected capabilities.
func NewPlainWriterWithCaps(out io.Writer, caps Caps) *PlainWriter {
	if out == nil {
		out = os.Stdout
	}
	return &PlainWriter{
		out:   &colorprofile.Writer{Forward: out, Profile: caps.Profile},
		theme: NewTheme(caps),
		caps:  caps,
	}
}

func (w *PlainWriter) line(s string) {
	_, _ = io.WriteString(w.out, s+"\n")
}

// PhaseStart emits a heading for the phase.
func (w *PlainWriter) PhaseStart(phase string, total int) {
	w.mu.Lock()
	defer w.mu.Unlock()
	w.flushSkipped()
	if total > 0 {
		w.line(w.theme.Heading(phase) + w.theme.Muted(fmt.Sprintf("  %d component(s)", total)))
		return
	}
	w.line(w.theme.Heading(phase))
}

// ComponentStart is a no-op: without motion, a "starting" line would double
// every component and the completion line already names it.
func (w *PlainWriter) ComponentStart(_ string) {}

// ComponentSkipped accumulates skips, reported as a count.
func (w *PlainWriter) ComponentSkipped(_ string) {
	w.mu.Lock()
	defer w.mu.Unlock()
	w.skipped++
}

// ComponentDone emits a success or failure line.
func (w *PlainWriter) ComponentDone(name string, err error) {
	w.mu.Lock()
	defer w.mu.Unlock()
	w.flushSkipped()
	if err != nil {
		w.line(w.theme.Item(StatusFailure, name, ""))
		for _, l := range detailLines(err.Error(), w.width()) {
			w.line(w.theme.Detail(l))
		}
		return
	}
	w.line(w.theme.Item(StatusSuccess, name, ""))
}

// Log emits a formatted log line.
func (w *PlainWriter) Log(format string, args ...any) {
	w.mu.Lock()
	defer w.mu.Unlock()
	w.flushSkipped()
	text := strings.TrimRight(fmt.Sprintf(format, args...), "\n")
	for _, l := range strings.Split(text, "\n") {
		w.line(l)
	}
}

// flushSkipped emits the pending skip count. The caller must hold mu.
func (w *PlainWriter) flushSkipped() {
	if w.skipped == 0 {
		return
	}
	w.line(w.theme.Countf("  %d already complete", w.skipped))
	w.skipped = 0
}

// Suspend is a no-op: PlainWriter never owns the terminal.
func (w *PlainWriter) Suspend() {}

// Resume is a no-op.
func (w *PlainWriter) Resume() {}

// Close flushes any pending skip count.
func (w *PlainWriter) Close() error {
	w.mu.Lock()
	defer w.mu.Unlock()
	w.flushSkipped()
	return nil
}

func (w *PlainWriter) width() int {
	if n := w.caps.Size(); n >= minWidth {
		return n
	}
	return 80
}
