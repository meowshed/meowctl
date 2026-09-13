package tui_test

import (
	"bytes"
	"strings"
	"testing"

	"github.com/meowshed/meowctl/internal/tui"
)

// A bytes.Buffer is never a terminal, which is the case that matters most:
// cursor movement written into a pipe or a CI log is unreadable.
func TestDetectCaps_NonTTYIsNeverLive(t *testing.T) {
	c := tui.DetectCaps(&bytes.Buffer{}, []string{"TERM=xterm-256color"}, tui.ModeAuto)
	if c.TTY {
		t.Fatal("a buffer must not be reported as a TTY")
	}
	if c.Live {
		t.Fatal("non-TTY output must not use the live renderer")
	}
}

func TestDetectCaps_DumbTerminalIsNotLive(t *testing.T) {
	c := tui.DetectCaps(&bytes.Buffer{}, []string{"TERM=dumb"}, tui.ModeAuto)
	if c.Live {
		t.Fatal("TERM=dumb must not use the live renderer")
	}
}

// CI systems capture stdout into a log file even when they allocate a pty.
func TestDetectCaps_CIForcesPlain(t *testing.T) {
	c := tui.DetectCaps(&bytes.Buffer{}, []string{"TERM=xterm-256color", "CI=true"}, tui.ModeAuto)
	if c.Live {
		t.Fatal("CI must not use the live renderer")
	}
}

func TestDetectCaps_ModeOverrides(t *testing.T) {
	env := []string{"TERM=xterm-256color"}
	if !tui.DetectCaps(&bytes.Buffer{}, env, tui.ModeLive).Live {
		t.Fatal("ModeLive must force the live renderer")
	}
	if tui.DetectCaps(&bytes.Buffer{}, env, tui.ModePlain).Live {
		t.Fatal("ModePlain must force the plain renderer")
	}
}

func TestDetectCaps_NoColorStripsColor(t *testing.T) {
	c := tui.DetectCaps(&bytes.Buffer{}, []string{"TERM=xterm-256color", "NO_COLOR=1"}, tui.ModeAuto)
	if c.Color() {
		t.Fatal("NO_COLOR must disable colour")
	}
}

// A non-UTF-8 locale reliably means braille and box drawing render as
// mojibake, so the theme must fall back to ASCII.
func TestDetectCaps_NonUTF8LocaleFallsBackToASCII(t *testing.T) {
	c := tui.DetectCaps(&bytes.Buffer{}, []string{"TERM=xterm", "LANG=C"}, tui.ModeAuto)
	if c.Unicode {
		t.Fatal("LANG=C must not advertise unicode")
	}
	if got := tui.NewTheme(c).Symbols().Success; got != "+" {
		t.Fatalf("expected ASCII symbol set, got %q", got)
	}
}

func TestDetectCaps_UTF8LocaleUsesUnicode(t *testing.T) {
	c := tui.DetectCaps(&bytes.Buffer{}, []string{"TERM=xterm", "LANG=en_US.UTF-8"}, tui.ModeAuto)
	if !c.Unicode {
		t.Fatal("a UTF-8 locale should advertise unicode")
	}
	if got := tui.NewTheme(c).Symbols().Success; got != "✓" {
		t.Fatalf("expected unicode symbol set, got %q", got)
	}
}

func TestParseMode(t *testing.T) {
	for in, want := range map[string]tui.Mode{
		"live":     tui.ModeLive,
		"LIVE":     tui.ModeLive,
		" plain ":  tui.ModePlain,
		"":         tui.ModeAuto,
		"nonsense": tui.ModeAuto, // a typo degrades to detection, never an error
	} {
		if got := tui.ParseMode(in); got != want {
			t.Fatalf("ParseMode(%q) = %v, want %v", in, got, want)
		}
	}
}

// Every state must be legible without colour, so no two statuses may share a
// glyph in either symbol set.
func TestTheme_StatusGlyphsAreDistinct(t *testing.T) {
	for _, unicode := range []bool{true, false} {
		theme := tui.NewTheme(tui.Caps{Unicode: unicode})
		seen := map[string]tui.Status{}
		for _, st := range []tui.Status{
			tui.StatusSuccess, tui.StatusFailure, tui.StatusWarning,
			tui.StatusSkipped, tui.StatusRunning, tui.StatusInfo,
		} {
			mark := theme.Mark(st)
			if prev, dup := seen[mark]; dup {
				t.Fatalf("unicode=%v: status %v and %v share glyph %q", unicode, prev, st, mark)
			}
			seen[mark] = st
		}
	}
}

func TestTheme_ItemIndentIsStable(t *testing.T) {
	theme := tui.NewTheme(tui.Caps{})
	line := theme.Item(tui.StatusSuccess, "brew", "0.2s")
	if !strings.HasPrefix(line, "  ") {
		t.Fatalf("items must be indented two spaces, got %q", line)
	}
	if !strings.Contains(line, "brew") || !strings.Contains(line, "0.2s") {
		t.Fatalf("item must carry label and note, got %q", line)
	}
}
