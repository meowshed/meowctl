package cli

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/meowshed/meowctl/internal/ctx"
	starlarkpkg "github.com/meowshed/meowctl/internal/starlark"
)

// TestCallExtensionHook_NoFile verifies that callExtensionHook returns nil
// when no hooks/<name>.star file exists.
func TestCallExtensionHook_NoFile(t *testing.T) {
	configDir := t.TempDir()
	caps := &ctx.Capabilities{Phase: "install", Component: "vim"}
	eval := &starlarkpkg.Evaluator{}

	err := callExtensionHook(configDir, "vim", "install", caps, eval)
	if err != nil {
		t.Fatalf("expected nil, got %v", err)
	}
}

// TestCallExtensionHook_ComponentDirOverride verifies that the extension hook
// receives ctx.component_dir = <configDir>/hooks/<componentID>, not the baseCaps value.
func TestCallExtensionHook_ComponentDirOverride(t *testing.T) {
	configDir := t.TempDir()
	hooksDir := filepath.Join(configDir, "hooks")
	if err := os.MkdirAll(hooksDir, 0o750); err != nil {
		t.Fatal(err)
	}

	// Write a hooks/vim.star that captures ctx.component_dir via ctx.run (echo).
	// We use a RunFunc to intercept the run call and capture the value.
	var capturedDir string
	extFile := filepath.Join(hooksDir, "vim.star")
	star := `
def install(ctx):
    ctx.log(ctx.component_dir)
`
	if err := os.WriteFile(extFile, []byte(star), 0o600); err != nil {
		t.Fatal(err)
	}

	caps := &ctx.Capabilities{
		Phase:        "install",
		Component:    "vim",
		ComponentDir: "/wrong/dir", // must be overridden
		Log: func(msg string) {
			capturedDir = msg
		},
	}
	eval := &starlarkpkg.Evaluator{}

	if err := callExtensionHook(configDir, "vim", "install", caps, eval); err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	want := filepath.Join(configDir, "hooks", "vim")
	if capturedDir != want {
		t.Fatalf("ctx.component_dir: want %q, got %q", want, capturedDir)
	}
}

// TestCallExtensionHook_BaseCapsUnmodified verifies that baseCaps.ComponentDir
// is not mutated by callExtensionHook.
func TestCallExtensionHook_BaseCapsUnmodified(t *testing.T) {
	configDir := t.TempDir()
	hooksDir := filepath.Join(configDir, "hooks")
	if err := os.MkdirAll(hooksDir, 0o750); err != nil {
		t.Fatal(err)
	}
	extFile := filepath.Join(hooksDir, "vim.star")
	if err := os.WriteFile(extFile, []byte(`def install(ctx): pass`), 0o600); err != nil {
		t.Fatal(err)
	}

	originalDir := "/original/component/dir"
	caps := &ctx.Capabilities{
		Phase:        "install",
		Component:    "vim",
		ComponentDir: originalDir,
	}
	eval := &starlarkpkg.Evaluator{}

	if err := callExtensionHook(configDir, "vim", "install", caps, eval); err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if caps.ComponentDir != originalDir {
		t.Fatalf("baseCaps.ComponentDir was mutated: got %q, want %q", caps.ComponentDir, originalDir)
	}
}

// TestRuntimeHookCaller_ExtensionComponentDir verifies runtimeHookCaller passes
// ctx.component_dir = <configDir>/hooks/<componentID>/ to the extension hook.
func TestRuntimeHookCaller_ExtensionComponentDir(t *testing.T) {
	configDir := t.TempDir()

	// No components/vim.star — callComponentHook will skip silently.
	// Create hooks/vim.star that captures component_dir via ctx.run.
	hooksDir := filepath.Join(configDir, "hooks")
	if err := os.MkdirAll(hooksDir, 0o750); err != nil {
		t.Fatal(err)
	}

	var capturedDir string
	extFile := filepath.Join(hooksDir, "vim.star")
	star := `
def shell(ctx):
    ctx.log(ctx.component_dir)
`
	if err := os.WriteFile(extFile, []byte(star), 0o600); err != nil {
		t.Fatal(err)
	}
	_ = extFile

	// We can't inject RunFunc into runtimeHookCaller directly, so we test that
	// callExtensionHook (which runtimeHookCaller delegates to) sets component_dir
	// correctly by constructing the caps the same way runtimeHookCaller.buildCaps does.
	caps := &ctx.Capabilities{
		RuntimeHook: true,
		Phase:       "shell",
		Component:   "vim",
		Log: func(msg string) {
			capturedDir = msg
		},
	}
	eval := &starlarkpkg.Evaluator{}

	if err := callExtensionHook(configDir, "vim", "shell", caps, eval); err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	want := filepath.Join(configDir, "hooks", "vim")
	if capturedDir != want {
		t.Fatalf("extension hook ctx.component_dir: want %q, got %q", want, capturedDir)
	}
}
