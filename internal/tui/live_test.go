package tui_test

import (
	"bytes"
	"errors"
	"strings"
	"testing"

	"github.com/charmbracelet/colorprofile"
	"github.com/mattn/go-runewidth"

	"github.com/meowshed/meowctl/internal/tui"
)

const (
	hideCursor = "\x1b[?25l"
	showCursor = "\x1b[?25h"
	cursorUp   = "\x1b[1A"
	eraseLine  = "\x1b[2K"
)

// liveCaps forces the live renderer while keeping output colourless, so tests
// assert on control sequences and text rather than on the palette.
func liveCaps() tui.Caps {
	return tui.Caps{Live: true, TTY: true, Unicode: true, Profile: colorprofile.Ascii, Width: 60}
}

func liveWriter() (*tui.LiveWriter, *bytes.Buffer) {
	buf := &bytes.Buffer{}
	return tui.NewLiveWriter(buf, liveCaps()), buf
}

// The cursor must always be restored — a command that exits with the cursor
// hidden leaves the user's shell broken.
func TestLiveWriter_RestoresCursorOnClose(t *testing.T) {
	w, buf := liveWriter()
	if !strings.HasPrefix(buf.String(), hideCursor) {
		t.Fatalf("expected the cursor to be hidden on start, got %q", buf.String())
	}
	if err := w.Close(); err != nil {
		t.Fatal(err)
	}
	if !strings.HasSuffix(buf.String(), showCursor) {
		t.Fatalf("expected the cursor to be restored on close, got %q", buf.String())
	}
}

// Close runs from a defer as well as explicitly in some paths.
func TestLiveWriter_CloseIsIdempotent(t *testing.T) {
	w, _ := liveWriter()
	if err := w.Close(); err != nil {
		t.Fatal(err)
	}
	if err := w.Close(); err != nil {
		t.Fatalf("second Close must be a no-op, got %v", err)
	}
}

// Finished work is committed above the live region so it survives in
// scrollback rather than being erased by the next frame.
func TestLiveWriter_CommitsFinishedComponents(t *testing.T) {
	w, buf := liveWriter()
	w.ComponentStart("brew")
	w.ComponentDone("brew", nil)
	_ = w.Close()
	if !strings.Contains(buf.String(), "brew") {
		t.Fatalf("expected the finished component to be committed, got %q", buf.String())
	}
}

func TestLiveWriter_FailureCommitsDetail(t *testing.T) {
	w, buf := liveWriter()
	w.ComponentStart("mise")
	w.ComponentDone("mise", errors.New("registry unreachable"))
	_ = w.Close()
	out := buf.String()
	if !strings.Contains(out, "mise") || !strings.Contains(out, "registry unreachable") {
		t.Fatalf("expected component and detail in output, got %q", out)
	}
}

// Erasing the region must issue exactly one cursor-up per drawn line;
// miscounting is what smears a live renderer across the scrollback.
func TestLiveWriter_ErasesEveryDrawnLine(t *testing.T) {
	w, buf := liveWriter()
	w.ComponentStart("a")
	w.ComponentStart("b")
	drawn := strings.Count(buf.String(), eraseLine)
	if drawn == 0 {
		t.Fatal("expected the region to be drawn")
	}
	buf.Reset()
	_ = w.Close()
	if got := strings.Count(buf.String(), cursorUp); got != 2 {
		t.Fatalf("expected 2 cursor-up for a 2-line region, got %d in %q", got, buf.String())
	}
}

// Suspend hands the terminal to a subprocess: the region must be erased, the
// cursor restored, and nothing further drawn until Resume.
func TestLiveWriter_SuspendReleasesTerminal(t *testing.T) {
	w, buf := liveWriter()
	w.ComponentStart("brew")
	buf.Reset()

	w.Suspend()
	if !strings.Contains(buf.String(), showCursor) {
		t.Fatalf("Suspend must restore the cursor, got %q", buf.String())
	}
	buf.Reset()

	w.ComponentStart("mise") // events during suspension must not draw
	if strings.Contains(buf.String(), eraseLine) {
		t.Fatalf("no drawing may happen while suspended, got %q", buf.String())
	}

	w.Resume()
	if !strings.Contains(buf.String(), hideCursor) {
		t.Fatalf("Resume must re-hide the cursor, got %q", buf.String())
	}
	_ = w.Close()
}

func TestLiveWriter_SuspendIsIdempotent(t *testing.T) {
	w, buf := liveWriter()
	w.ComponentStart("brew")
	w.Suspend()
	w.Suspend() // nested suspension must not double-erase the region
	w.Resume()
	buf.Reset()
	w.Resume() // an unmatched Resume must not redraw
	if strings.Contains(buf.String(), eraseLine) {
		t.Fatalf("unmatched Resume must be a no-op, got %q", buf.String())
	}
	_ = w.Close()
}

// Lines wider than the terminal would wrap, and a wrapped line breaks the
// cursor-up arithmetic on the next frame.
func TestLiveWriter_TruncatesToWidth(t *testing.T) {
	buf := &bytes.Buffer{}
	caps := liveCaps()
	caps.Width = 30
	w := tui.NewLiveWriter(buf, caps)
	w.ComponentStart(strings.Repeat("x", 200))
	_ = w.Close()

	// Width is measured in display columns, not bytes: the glyphs and the
	// ellipsis are multibyte, and it is the column count that wraps.
	for _, line := range strings.Split(stripANSI(buf.String()), "\n") {
		if w := runewidth.StringWidth(line); w > 30 {
			t.Fatalf("line exceeds terminal width: %d cols, %q", w, line)
		}
	}
}

func TestLiveWriter_SkipsAreCounted(t *testing.T) {
	w, buf := liveWriter()
	w.ComponentSkipped("a")
	w.ComponentSkipped("b")
	if !strings.Contains(stripANSI(buf.String()), "2 already complete") {
		t.Fatalf("expected a skip count in the region, got %q", buf.String())
	}
	_ = w.Close()
}

// Log output belongs above the live region, not inside it.
func TestLiveWriter_LogCommitsAboveRegion(t *testing.T) {
	w, buf := liveWriter()
	w.ComponentStart("brew")
	buf.Reset()
	w.Log("warning: something happened\n")
	if !strings.Contains(buf.String(), "warning: something happened") {
		t.Fatalf("expected the log line to be committed, got %q", buf.String())
	}
	_ = w.Close()
}

// stripANSI removes CSI sequences so assertions can measure visible text.
func stripANSI(s string) string {
	var b strings.Builder
	for i := 0; i < len(s); {
		if s[i] == 0x1b && i+1 < len(s) && s[i+1] == '[' {
			i += 2
			for i < len(s) && (s[i] < 0x40 || s[i] > 0x7e) {
				i++
			}
			i++
			continue
		}
		b.WriteByte(s[i])
		i++
	}
	return b.String()
}
