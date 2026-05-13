package tui

import (
	"fmt"
	"io"
)

// PlainWriter writes uncoloured, line-buffered progress output. It is used in
// non-TTY environments (pipes, CI) where ANSI codes would pollute the output.
type PlainWriter struct {
	out io.Writer
}

// NewPlainWriter returns a [PlainWriter] writing to out. Prefer [New] in
// production code; use NewPlainWriter directly in tests.
func NewPlainWriter(out io.Writer) *PlainWriter {
	return &PlainWriter{out: out}
}

// ComponentStart emits a "starting" line for the named component.
func (w *PlainWriter) ComponentStart(name string) {
	_, _ = fmt.Fprintf(w.out, "  -> %s\n", name)
}

// ComponentDone emits a success or failure line for the named component.
func (w *PlainWriter) ComponentDone(name string, err error) {
	if err != nil {
		_, _ = fmt.Fprintf(w.out, "  FAIL %s: %v\n", name, err)
		return
	}
	_, _ = fmt.Fprintf(w.out, "  ok   %s\n", name)
}

// Log emits a formatted log line.
func (w *PlainWriter) Log(format string, args ...any) {
	_, _ = fmt.Fprintf(w.out, format, args...)
}

// Close is a no-op for PlainWriter.
func (w *PlainWriter) Close() error { return nil }
