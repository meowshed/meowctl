package tui

import (
	"io"
	"os"

	"golang.org/x/term"
)

// Writer is the output abstraction for progress and status events.
// Implementations are TTY-aware; callers do not need to check the terminal.
type Writer interface {
	// ComponentStart marks a component as in-progress.
	ComponentStart(name string)
	// ComponentDone marks a component as completed. A non-nil err indicates
	// failure.
	ComponentDone(name string, err error)
	// Log emits a free-form log line.
	Log(format string, args ...any)
	// Close flushes output and shuts down the underlying program (if any).
	Close() error
}

// New returns a [Writer] suitable for out. When out is a TTY it returns a
// [BubbleTeaWriter]; otherwise it returns a [PlainWriter].
func New(out io.Writer) Writer {
	if f, ok := out.(*os.File); ok && term.IsTerminal(int(f.Fd())) { //nolint:gosec // G115: uintptr→int safe for fd values
		return newBubbleTeaWriter(out)
	}
	return NewPlainWriter(out)
}
