package tui_test

import (
	"bytes"
	"errors"
	"strings"
	"testing"

	"github.com/charmbracelet/colorprofile"
	"github.com/meowshed/meowctl/internal/tui"
)

// asciiCaps is a deterministic non-colour, non-live capability set so tests
// assert on text rather than on escape sequences.
func asciiCaps() tui.Caps {
	return tui.Caps{Unicode: false, Profile: colorprofile.Ascii, Width: 80}
}

func plainWriter() (*tui.PlainWriter, *bytes.Buffer) {
	buf := &bytes.Buffer{}
	return tui.NewPlainWriterWithCaps(buf, asciiCaps()), buf
}

func TestPlainWriter_ComponentStartIsSilent(t *testing.T) {
	w, buf := plainWriter()
	w.ComponentStart("brew")
	if buf.String() != "" {
		t.Fatalf("ComponentStart should not emit a line without motion, got %q", buf.String())
	}
}

func TestPlainWriter_ComponentDone_success(t *testing.T) {
	w, buf := plainWriter()
	w.ComponentDone("brew", nil)
	out := buf.String()
	if !strings.Contains(out, "brew") {
		t.Fatalf("expected component name, got %q", out)
	}
	if strings.Contains(strings.ToUpper(out), "FAIL") {
		t.Fatalf("unexpected FAIL in success output: %q", out)
	}
}

func TestPlainWriter_ComponentDone_failureShowsDetail(t *testing.T) {
	w, buf := plainWriter()
	w.ComponentDone("mise", errors.New("exit 1: could not reach registry"))
	out := buf.String()
	if !strings.Contains(out, "mise") {
		t.Fatalf("expected component name, got %q", out)
	}
	if !strings.Contains(out, "could not reach registry") {
		t.Fatalf("expected captured detail in output, got %q", out)
	}
}

// Skips are counted rather than listed: on a settled config they outnumber the
// components that ran by two orders of magnitude.
func TestPlainWriter_SkipsAreCounted(t *testing.T) {
	w, buf := plainWriter()
	for i := 0; i < 3; i++ {
		w.ComponentSkipped("x")
	}
	if buf.String() != "" {
		t.Fatalf("skips should not emit until flushed, got %q", buf.String())
	}
	if err := w.Close(); err != nil {
		t.Fatal(err)
	}
	if !strings.Contains(buf.String(), "3 already complete") {
		t.Fatalf("expected skip count, got %q", buf.String())
	}
}

func TestPlainWriter_SkipCountFlushesBeforeNextItem(t *testing.T) {
	w, buf := plainWriter()
	w.ComponentSkipped("a")
	w.ComponentDone("b", nil)
	out := buf.String()
	if strings.Index(out, "1 already complete") > strings.Index(out, "b") {
		t.Fatalf("skip count must precede the next item, got %q", out)
	}
}

func TestPlainWriter_PhaseStart(t *testing.T) {
	w, buf := plainWriter()
	w.PhaseStart("install", 4)
	out := buf.String()
	if !strings.Contains(out, "install") || !strings.Contains(out, "4") {
		t.Fatalf("expected phase name and count, got %q", out)
	}
}

func TestPlainWriter_Log(t *testing.T) {
	w, buf := plainWriter()
	w.Log("hello %s\n", "world")
	if buf.String() != "hello world\n" {
		t.Fatalf("unexpected log output: %q", buf.String())
	}
}

// A plain writer never owns the terminal, so handing it over is a no-op that
// must still be safe to call.
func TestPlainWriter_SuspendResumeAreSafe(t *testing.T) {
	w, buf := plainWriter()
	w.Suspend()
	w.Resume()
	if buf.String() != "" {
		t.Fatalf("suspend/resume should emit nothing, got %q", buf.String())
	}
}

func TestPlainWriter_NoEscapeSequencesWithoutColor(t *testing.T) {
	w, buf := plainWriter()
	w.PhaseStart("install", 1)
	w.ComponentDone("brew", errors.New("boom"))
	_ = w.Close()
	if strings.Contains(buf.String(), "\x1b") {
		t.Fatalf("Ascii profile must strip all SGR, got %q", buf.String())
	}
}

func TestPlainWriter_Close(t *testing.T) {
	w, _ := plainWriter()
	if err := w.Close(); err != nil {
		t.Fatalf("Close() returned error: %v", err)
	}
}
