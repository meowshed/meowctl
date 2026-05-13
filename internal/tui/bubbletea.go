package tui

import "io"

// BubbleTeaWriter runs a Bubble Tea v2 program inline (no alt-screen) and
// renders per-component spinner rows with Lip Gloss styling.
// Use [New] to obtain a BubbleTeaWriter; it is returned when stdout is a TTY.
type BubbleTeaWriter struct {
	// TODO(m7): implement in commit 4
	plain *PlainWriter
}

func newBubbleTeaWriter(out io.Writer) *BubbleTeaWriter {
	// Placeholder: falls back to PlainWriter until the Bubble Tea model is
	// implemented in commit 4.
	return &BubbleTeaWriter{plain: NewPlainWriter(out)}
}

// ComponentStart marks a component as in-progress.
func (w *BubbleTeaWriter) ComponentStart(name string) { w.plain.ComponentStart(name) }

// ComponentDone marks a component as completed.
func (w *BubbleTeaWriter) ComponentDone(name string, err error) { w.plain.ComponentDone(name, err) }

// Log emits a formatted log line.
func (w *BubbleTeaWriter) Log(format string, args ...any) { w.plain.Log(format, args...) }

// Close shuts down the Bubble Tea program.
func (w *BubbleTeaWriter) Close() error { return nil }
