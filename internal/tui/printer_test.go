package tui_test

import (
	"bytes"
	"strings"
	"testing"

	"github.com/charmbracelet/colorprofile"
	"github.com/meowshed/meowctl/internal/tui"
)

func printer() (*tui.Printer, *bytes.Buffer, *bytes.Buffer) {
	out, errOut := &bytes.Buffer{}, &bytes.Buffer{}
	caps := tui.Caps{Unicode: false, Profile: colorprofile.Ascii, Width: 80}
	return tui.NewPrinterWithCaps(out, errOut, caps), out, errOut
}

// Diagnostics must never land on stdout: a caller piping `meowctl dep list`
// would get warnings mixed into the data.
func TestPrinter_WarningsGoToStderr(t *testing.T) {
	p, out, errOut := printer()
	p.Warn("something is off")
	if out.Len() != 0 {
		t.Fatalf("warning leaked to stdout: %q", out.String())
	}
	if !strings.Contains(errOut.String(), "something is off") {
		t.Fatalf("expected the warning on stderr, got %q", errOut.String())
	}
}

func TestPrinter_ResultsGoToStdout(t *testing.T) {
	p, out, errOut := printer()
	p.Success("brew", "0.2s")
	if errOut.Len() != 0 {
		t.Fatalf("result leaked to stderr: %q", errOut.String())
	}
	if !strings.Contains(out.String(), "brew") {
		t.Fatalf("expected the result on stdout, got %q", out.String())
	}
}

func TestPrinter_TableAligns(t *testing.T) {
	p, out, _ := printer()
	p.Table([]string{"name", "version"}, [][]string{
		{"dotmeow", "0.3.18"},
		{"a", "1"},
	})
	lines := strings.Split(strings.TrimRight(out.String(), "\n"), "\n")
	if len(lines) != 3 {
		t.Fatalf("expected a header and two rows, got %q", out.String())
	}
	if !strings.Contains(lines[0], "NAME") {
		t.Fatalf("header should be upper-cased, got %q", lines[0])
	}
	if strings.Index(lines[1], "0.3.18") != strings.Index(lines[2], "1") {
		t.Fatalf("columns are not aligned:\n%q\n%q", lines[1], lines[2])
	}
}

func TestPrinter_ItemListAlignsNotes(t *testing.T) {
	p, out, _ := printer()
	p.ItemList([]tui.ItemSpec{
		{Status: tui.StatusSuccess, Label: "init.star", Note: "ok"},
		{Status: tui.StatusSuccess, Label: "a", Note: "ok"},
	})
	lines := strings.Split(strings.TrimRight(out.String(), "\n"), "\n")
	if strings.Index(lines[0], "ok") != strings.Index(lines[1], "ok") {
		t.Fatalf("notes are not aligned:\n%q\n%q", lines[0], lines[1])
	}
}
