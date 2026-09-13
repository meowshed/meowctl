package tui

// SilentWriter discards all output. Used by meowctl hook where the only
// stdout output must come from ctx.emit() calls in Starlark components.
type SilentWriter struct{}

// NewSilentWriter returns a Writer that discards all progress and status events.
func NewSilentWriter() *SilentWriter { return &SilentWriter{} }

// PhaseStart is a no-op.
func (w *SilentWriter) PhaseStart(_ string, _ int) {}

// ComponentStart is a no-op.
func (w *SilentWriter) ComponentStart(_ string) {}

// ComponentSkipped is a no-op.
func (w *SilentWriter) ComponentSkipped(_ string) {}

// ComponentDone is a no-op.
func (w *SilentWriter) ComponentDone(_ string, _ error) {}

// Log is a no-op.
func (w *SilentWriter) Log(_ string, _ ...any) {}

// Suspend is a no-op.
func (w *SilentWriter) Suspend() {}

// Resume is a no-op.
func (w *SilentWriter) Resume() {}

// Close is a no-op.
func (w *SilentWriter) Close() error { return nil }
