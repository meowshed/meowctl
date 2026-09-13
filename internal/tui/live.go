package tui

import (
	"fmt"
	"io"
	"strings"
	"sync"
	"time"

	"github.com/charmbracelet/colorprofile"
	"github.com/mattn/go-runewidth"
)

// Terminal control. Deliberately the smallest portable subset:
//
//	CUU  \x1b[<n>A   cursor up          — universal
//	EL   \x1b[2K     erase whole line   — universal
//	DECTCEM \x1b[?25l/h  hide/show cursor — universal
//	SYNC \x1b[?2026h/l   synchronised output — modern terminals; unknown
//	                     private modes are ignored by everything else, so
//	                     this is safe to emit unconditionally.
//
// Nothing here uses the alternate screen, scroll regions, cursor save/restore
// (ambiguous between DECSC and SCOSC), or OSC sequences — those are where
// terminal support actually diverges.
const (
	ansiEraseLine   = "\x1b[2K"
	ansiHideCursor  = "\x1b[?25l"
	ansiShowCursor  = "\x1b[?25h"
	ansiSyncStart   = "\x1b[?2026h"
	ansiSyncEnd     = "\x1b[?2026l"
	spinnerInterval = 90 * time.Millisecond
	// minWidth guards the truncation arithmetic on absurdly narrow terminals.
	minWidth = 20
)

// row is one component's state within the live region.
type row struct {
	name    string
	status  Status
	note    string
	started time.Time
	elapsed time.Duration
}

// LiveWriter renders an in-place progress region without putting the terminal
// into raw mode.
//
// Not using raw mode is the whole point. Lifecycle hooks shell out through
// ctx.run, and those subprocesses may need the real terminal — a cask asking
// for an admin password, git asking for credentials, gpg driving pinentry. A
// renderer that owns the terminal has to hand it back for every one of those;
// this one simply erases its region, stops drawing, and lets the child write
// wherever it likes. The worst failure is a smeared frame, never a swallowed
// password prompt.
type LiveWriter struct {
	mu      sync.Mutex
	out     io.Writer
	caps    Caps
	theme   Theme
	rows    []*row
	index   map[string]*row
	phase   string
	drawn   int // lines currently occupying the live region
	frame   int
	stopped bool
	paused  bool
	done    chan struct{}
	wg      sync.WaitGroup
	skipped int
}

// NewLiveWriter returns a writer that renders in place on out.
func NewLiveWriter(out io.Writer, caps Caps) *LiveWriter {
	w := &LiveWriter{
		out:   &colorprofile.Writer{Forward: out, Profile: caps.Profile},
		caps:  caps,
		theme: NewTheme(caps),
		index: map[string]*row{},
		done:  make(chan struct{}),
	}
	w.raw(ansiHideCursor)
	w.wg.Add(1)
	go w.tick()
	return w
}

// tick animates the spinner while at least one row is running.
func (w *LiveWriter) tick() {
	defer w.wg.Done()
	t := time.NewTicker(spinnerInterval)
	defer t.Stop()
	for {
		select {
		case <-w.done:
			return
		case <-t.C:
			w.mu.Lock()
			if !w.paused && !w.stopped && w.anyRunning() {
				w.frame++
				w.draw()
			}
			w.mu.Unlock()
		}
	}
}

func (w *LiveWriter) anyRunning() bool {
	for _, r := range w.rows {
		if r.status == StatusRunning {
			return true
		}
	}
	return false
}

// raw writes a control string directly, bypassing the colour writer.
func (w *LiveWriter) raw(s string) {
	_, _ = io.WriteString(w.out, s)
}

// PhaseStart opens a new live region for a lifecycle phase.
func (w *LiveWriter) PhaseStart(phase string, _ int) {
	w.mu.Lock()
	defer w.mu.Unlock()
	w.clear()
	w.phase = phase
	w.rows = nil
	w.index = map[string]*row{}
	w.skipped = 0
	w.draw()
}

// ComponentStart marks a component as running.
func (w *LiveWriter) ComponentStart(name string) {
	w.mu.Lock()
	defer w.mu.Unlock()
	r := &row{name: name, status: StatusRunning, started: time.Now()}
	w.rows = append(w.rows, r)
	w.index[name] = r
	w.draw()
}

// ComponentSkipped records a component the runner will not execute. Skips are
// counted rather than listed: on a settled configuration they are the large
// majority, and printing hundreds of them buries the handful that ran.
func (w *LiveWriter) ComponentSkipped(_ string) {
	w.mu.Lock()
	defer w.mu.Unlock()
	w.skipped++
	w.draw()
}

// ComponentDone resolves a running component and commits its line to
// scrollback so the finished work survives above the live region.
func (w *LiveWriter) ComponentDone(name string, err error) {
	w.mu.Lock()
	defer w.mu.Unlock()
	r, ok := w.index[name]
	if !ok {
		r = &row{name: name, started: time.Now()}
		w.rows = append(w.rows, r)
		w.index[name] = r
	}
	r.elapsed = time.Since(r.started)
	r.status = StatusSuccess
	if err != nil {
		r.status = StatusFailure
		r.note = err.Error()
	}

	w.clear()
	w.commitLine(w.rowLine(r, false))
	if err != nil {
		for _, line := range detailLines(err.Error(), w.width()) {
			w.commitLine(w.theme.Detail(line))
		}
	}
	w.dropRow(name)
	w.draw()
}

// dropRow removes a committed row from the live region.
func (w *LiveWriter) dropRow(name string) {
	delete(w.index, name)
	out := w.rows[:0]
	for _, r := range w.rows {
		if r.name != name {
			out = append(out, r)
		}
	}
	w.rows = out
}

// Log emits a free-form line above the live region.
func (w *LiveWriter) Log(format string, args ...any) {
	w.mu.Lock()
	defer w.mu.Unlock()
	w.clear()
	text := strings.TrimRight(fmt.Sprintf(format, args...), "\n")
	for _, line := range strings.Split(text, "\n") {
		w.commitLine(line)
	}
	w.draw()
}

// Suspend erases the live region and stops drawing, handing the terminal to a
// subprocess or a prompt. Safe to nest; Resume must be called once per call.
func (w *LiveWriter) Suspend() {
	w.mu.Lock()
	defer w.mu.Unlock()
	if w.paused {
		return
	}
	w.clear()
	w.raw(ansiShowCursor)
	w.paused = true
}

// Resume redraws the live region after a Suspend.
func (w *LiveWriter) Resume() {
	w.mu.Lock()
	defer w.mu.Unlock()
	if !w.paused {
		return
	}
	w.paused = false
	w.raw(ansiHideCursor)
	w.draw()
}

// Close tears the region down and restores the cursor. Idempotent, so a
// deferred Close after an explicit one is harmless.
func (w *LiveWriter) Close() error {
	w.mu.Lock()
	if w.stopped {
		w.mu.Unlock()
		return nil
	}
	w.stopped = true
	w.clear()
	w.raw(ansiShowCursor)
	w.mu.Unlock()

	close(w.done)
	w.wg.Wait()
	return nil
}

// commitLine writes one permanent line. The caller must hold mu and have
// cleared the live region.
func (w *LiveWriter) commitLine(s string) {
	_, _ = io.WriteString(w.out, s+"\n")
}

// clear erases the live region and parks the cursor at its first column.
func (w *LiveWriter) clear() {
	if w.drawn == 0 {
		return
	}
	var b strings.Builder
	for i := 0; i < w.drawn; i++ {
		b.WriteString("\x1b[1A")
		b.WriteString(ansiEraseLine)
		b.WriteString("\r")
	}
	w.raw(b.String())
	w.drawn = 0
}

// draw renders the current region. The caller must hold mu.
func (w *LiveWriter) draw() {
	if w.paused || w.stopped || !w.caps.Live {
		return
	}
	w.clear()
	lines := w.frameLines()
	if len(lines) == 0 {
		return
	}
	var b strings.Builder
	b.WriteString(ansiSyncStart)
	for _, line := range lines {
		b.WriteString(ansiEraseLine)
		b.WriteString(line)
		b.WriteString("\n")
	}
	b.WriteString(ansiSyncEnd)
	w.raw(b.String())
	w.drawn = len(lines)
}

// frameLines builds the current region content.
func (w *LiveWriter) frameLines() []string {
	var lines []string
	for _, r := range w.rows {
		lines = append(lines, w.rowLine(r, true))
	}
	// Cap the region so cursor-up arithmetic can never exceed the viewport.
	if limit := w.maxRows(); len(lines) > limit {
		hidden := len(lines) - limit
		lines = lines[:limit]
		lines = append(lines, w.theme.Countf("  %s %d more", w.theme.Symbols().Bullet, hidden))
	}
	if w.skipped > 0 {
		lines = append(lines, w.theme.Countf("  %d already complete", w.skipped))
	}
	return lines
}

// maxRows keeps the live region comfortably inside the viewport.
func (w *LiveWriter) maxRows() int {
	const fallback = 12
	if w.caps.fd < 0 {
		return fallback
	}
	_, h, err := termSize(w.caps.fd)
	if err != nil || h <= 0 {
		return fallback
	}
	if n := h - 4; n > 0 {
		return n
	}
	return 1
}

// rowLine renders one component row, animating it when live.
func (w *LiveWriter) rowLine(r *row, live bool) string {
	mark := w.theme.Mark(r.status)
	if live && r.status == StatusRunning {
		frames := w.theme.Symbols().Spinner
		mark = w.theme.Accent(frames[w.frame%len(frames)])
	}

	note := r.note
	if r.status == StatusSuccess && r.elapsed > 0 {
		note = formatDuration(r.elapsed)
	}
	if r.status == StatusRunning {
		note = ""
	}
	if r.status == StatusFailure {
		note = firstLine(note)
	}

	width := w.width()
	// 2 indent + 1 mark + 1 space, then the label, then 2 spaces before a note.
	budget := width - 4
	name := r.name
	if budget > 0 && runewidth.StringWidth(name) > budget {
		name = runewidth.Truncate(name, budget, "…")
		note = ""
	}
	if note != "" {
		remaining := budget - runewidth.StringWidth(name) - 2
		if remaining < 4 {
			note = ""
		} else if runewidth.StringWidth(note) > remaining {
			note = runewidth.Truncate(note, remaining, "…")
		}
	}

	line := "  " + mark + " " + name
	if note != "" {
		line += "  " + w.theme.Muted(note)
	}
	return line
}

func (w *LiveWriter) width() int {
	if n := w.caps.Size(); n >= minWidth {
		return n
	}
	if w.caps.Width >= minWidth {
		return w.caps.Width
	}
	return 80
}

// detailLines wraps captured output for display under a failed item.
func detailLines(text string, width int) []string {
	budget := width - 6
	if budget < minWidth {
		budget = minWidth
	}
	var out []string
	for _, line := range strings.Split(strings.TrimRight(text, "\n"), "\n") {
		line = strings.TrimRight(line, "\r")
		if line == "" {
			continue
		}
		for runewidth.StringWidth(line) > budget {
			cut := runewidth.Truncate(line, budget, "")
			out = append(out, cut)
			line = strings.TrimPrefix(line, cut)
		}
		out = append(out, line)
	}
	return out
}

func firstLine(s string) string {
	if i := strings.IndexByte(s, '\n'); i >= 0 {
		return s[:i]
	}
	return s
}

// formatDuration renders a wall time compactly: sub-second work is noise at
// millisecond precision, and anything over a minute is what a reader is
// actually scanning for.
func formatDuration(d time.Duration) string {
	switch {
	case d < time.Second:
		return fmt.Sprintf("%dms", d.Milliseconds())
	case d < time.Minute:
		return fmt.Sprintf("%.1fs", d.Seconds())
	default:
		return fmt.Sprintf("%dm%02ds", int(d.Minutes()), int(d.Seconds())%60)
	}
}
