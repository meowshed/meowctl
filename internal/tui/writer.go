package tui

import (
	"io"
	"os"
)

// Writer is the output abstraction for progress and status events.
// Implementations are TTY-aware; callers do not need to check the terminal.
type Writer interface {
	// PhaseStart opens a lifecycle phase. total is the number of components
	// scheduled for it, or 0 when unknown.
	PhaseStart(phase string, total int)
	// ComponentStart marks a component as in-progress.
	ComponentStart(name string)
	// ComponentSkipped marks a component as skipped (already completed).
	ComponentSkipped(name string)
	// ComponentDone marks a component as completed. A non-nil err indicates
	// failure.
	ComponentDone(name string, err error)
	// Log emits a free-form log line. The format string must include any
	// desired trailing newline.
	Log(format string, args ...any)
	// Suspend releases the terminal so a subprocess or prompt can use it.
	// Implementations that never take the terminal may no-op.
	Suspend()
	// Resume resumes rendering after a Suspend.
	Resume()
	// Close flushes output and shuts down the underlying program (if any).
	Close() error
}

// New returns a Writer suitable for out, resolving the renderer from the
// terminal's capabilities and the MEOWCTL_OUTPUT override.
func New(out io.Writer) Writer {
	return NewWithMode(out, ParseMode(os.Getenv("MEOWCTL_OUTPUT")))
}

// NewWithMode is New with an explicit mode, for flags and tests.
func NewWithMode(out io.Writer, mode Mode) Writer {
	caps := DetectCaps(out, os.Environ(), mode)
	if caps.Live {
		return NewLiveWriter(out, caps)
	}
	return NewPlainWriterWithCaps(out, caps)
}
