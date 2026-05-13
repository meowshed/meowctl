package tui_test

import (
	"bytes"
	"errors"
	"strings"
	"testing"

	"github.com/meowshed/meowctl/internal/tui"
)

func plainWriter(t *testing.T) (*tui.PlainWriter, *bytes.Buffer) {
	t.Helper()
	buf := &bytes.Buffer{}
	return tui.NewPlainWriter(buf), buf
}

func TestPlainWriter_ComponentStart(t *testing.T) {
	w, buf := plainWriter(t)
	w.ComponentStart("brew")
	if !strings.Contains(buf.String(), "brew") {
		t.Fatalf("expected component name in output, got %q", buf.String())
	}
}

func TestPlainWriter_ComponentDone_success(t *testing.T) {
	w, buf := plainWriter(t)
	w.ComponentDone("brew", nil)
	out := buf.String()
	if !strings.Contains(out, "brew") {
		t.Fatalf("expected component name in output, got %q", out)
	}
	if strings.Contains(strings.ToUpper(out), "FAIL") {
		t.Fatalf("unexpected FAIL in success output: %q", out)
	}
}

func TestPlainWriter_ComponentDone_failure(t *testing.T) {
	w, buf := plainWriter(t)
	w.ComponentDone("mise", errors.New("exit 1"))
	out := buf.String()
	if !strings.Contains(out, "mise") {
		t.Fatalf("expected component name in output, got %q", out)
	}
	if !strings.Contains(out, "exit 1") {
		t.Fatalf("expected error text in output, got %q", out)
	}
}

func TestPlainWriter_Log(t *testing.T) {
	w, buf := plainWriter(t)
	w.Log("hello %s\n", "world")
	if buf.String() != "hello world\n" {
		t.Fatalf("unexpected log output: %q", buf.String())
	}
}

func TestPlainWriter_Close(t *testing.T) {
	w, _ := plainWriter(t)
	if err := w.Close(); err != nil {
		t.Fatalf("Close() returned error: %v", err)
	}
}
