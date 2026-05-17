package cli

import (
	"os"
	"path/filepath"
	"strings"
	"testing"
)

// ---------------------------------------------------------------------------
// hook .hook-error flag file (tests unexported helpers — must be package cli)
// ---------------------------------------------------------------------------

func TestWriteHookError_CreatesFile(t *testing.T) {
	tmp := t.TempDir()
	err := os.ErrInvalid // any non-nil error
	writeHookError(tmp, err)
	path := filepath.Join(tmp, configHookErrorFile)
	data, readErr := os.ReadFile(path) // #nosec G304
	if readErr != nil {
		t.Fatalf("expected .hook-error to be created, got: %v", readErr)
	}
	if !strings.Contains(string(data), err.Error()) {
		t.Fatalf(".hook-error does not contain error text: %s", data)
	}
}

func TestClearHookError_RemovesFile(t *testing.T) {
	tmp := t.TempDir()
	path := filepath.Join(tmp, configHookErrorFile)
	_ = os.WriteFile(path, []byte("err"), 0o600)
	clearHookError(tmp)
	if _, statErr := os.Lstat(path); !os.IsNotExist(statErr) {
		t.Fatal("expected .hook-error to be removed after clearHookError")
	}
}

func TestClearHookError_NoopWhenAbsent(t *testing.T) {
	tmp := t.TempDir()
	// Should not panic when file doesn't exist.
	clearHookError(tmp)
}

func TestHookErrorExists_TrueWhenPresent(t *testing.T) {
	tmp := t.TempDir()
	_ = os.WriteFile(filepath.Join(tmp, configHookErrorFile), []byte("err"), 0o600)
	if !hookErrorExists(tmp) {
		t.Fatal("hookErrorExists should return true when file present")
	}
}

func TestHookErrorExists_FalseWhenAbsent(t *testing.T) {
	tmp := t.TempDir()
	if hookErrorExists(tmp) {
		t.Fatal("hookErrorExists should return false when file absent")
	}
}

// ---------------------------------------------------------------------------
// add module check (tests unexported helper — must be package cli)
// ---------------------------------------------------------------------------

func TestCheckModuleDeclared_BareNamePassesThrough(t *testing.T) {
	tmp := t.TempDir()
	if err := checkModuleDeclared(tmp, "mycomponent"); err != nil {
		t.Fatalf("bare component name should pass module check, got: %v", err)
	}
}

func TestCheckModuleDeclared_ModuleInSharedMod(t *testing.T) {
	tmp := t.TempDir()
	writeTestModfile(t, filepath.Join(tmp, "deps.mod"), `dep(name = "stdlib", version = "0.1.0")`)
	if err := checkModuleDeclared(tmp, "@stdlib//components/node"); err != nil {
		t.Fatalf("module in deps.mod should pass check, got: %v", err)
	}
}

func TestCheckModuleDeclared_ModuleInLocalMod(t *testing.T) {
	tmp := t.TempDir()
	writeTestModfile(t, filepath.Join(tmp, "deps.local.mod"), `dep(name = "mypkg", version = "0.1.0")`)
	if err := checkModuleDeclared(tmp, "@mypkg//components/foo"); err != nil {
		t.Fatalf("module in deps.local.mod should pass check, got: %v", err)
	}
}

func TestCheckModuleDeclared_ModuleMissing_ReturnsError(t *testing.T) {
	tmp := t.TempDir()
	err := checkModuleDeclared(tmp, "@missingmod//components/thing")
	if err == nil {
		t.Fatal("expected error when module not in any modfile")
	}
	if !strings.Contains(err.Error(), "missingmod") {
		t.Fatalf("error should mention module name, got: %v", err)
	}
	if !strings.Contains(err.Error(), "dep add") {
		t.Fatalf("error should suggest 'dep add', got: %v", err)
	}
}

// writeTestModfile is a local helper (avoids conflict with cli_test.go's writeModfile).
func writeTestModfile(t *testing.T, path, content string) {
	t.Helper()
	if err := os.WriteFile(path, []byte(content), 0o600); err != nil {
		t.Fatalf("writeTestModfile: %v", err)
	}
}
