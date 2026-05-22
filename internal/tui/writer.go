package tui

import (
	"io"
)

// Writer is the output abstraction for progress and status events.
// Implementations are TTY-aware; callers do not need to check the terminal.
type Writer interface {
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
	// Close flushes output and shuts down the underlying program (if any).
	Close() error
}

// New returns a [Writer] suitable for out. When out is a TTY it returns a
// [BubbleTeaWriter]; otherwise it returns a [PlainWriter].
//
// TODO(retran): BubbleTeaWriter puts the terminal in raw mode which breaks
// subprocesses and ctx.prompt() calls during lifecycle hooks. Use PlainWriter
// unconditionally until BubbleTea can suspend/resume around hook execution.
func New(out io.Writer) Writer {
	return NewPlainWriter(out)
}
