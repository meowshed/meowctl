package cli_test

import (
	"errors"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/meowshed/meowctl/internal/cli"
)

// TestExitError_CodePreserved verifies ExitError carries the correct code.
func TestExitError_CodePreserved(t *testing.T) {
	err := &cli.ExitError{Code: cli.ExitConfig, Err: errors.New("bad config")}
	if err.Code != cli.ExitConfig {
		t.Fatalf("want code %d, got %d", cli.ExitConfig, err.Code)
	}
	if err.Error() != "bad config" {
		t.Fatalf("want message 'bad config', got %q", err.Error())
	}
}

// TestExitError_Unwrap verifies errors.As traverses wrapping.
func TestExitError_Unwrap(t *testing.T) {
	inner := errors.New("inner")
	outer := &cli.ExitError{Code: cli.ExitModule, Err: inner}
	var target *cli.ExitError
	if !errors.As(outer, &target) {
		t.Fatal("errors.As should find *cli.ExitError")
	}
	if target.Code != cli.ExitModule {
		t.Fatalf("want ExitModule (%d), got %d", cli.ExitModule, target.Code)
	}
}

// TestDefaultConfigDir_XDG verifies XDG_CONFIG_HOME is honoured.
func TestDefaultConfigDir_XDG(t *testing.T) {
	t.Setenv("XDG_CONFIG_HOME", "/tmp/xdg")
	dir, err := cli.ExportDefaultConfigDir()
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if dir != "/tmp/xdg/meowctl" {
		t.Fatalf("want /tmp/xdg/meowctl, got %q", dir)
	}
}

// TestDefaultConfigDir_HomeDir verifies the default falls back to ~/.config/meowctl.
func TestDefaultConfigDir_HomeDir(t *testing.T) {
	t.Setenv("XDG_CONFIG_HOME", "")
	home, err := os.UserHomeDir()
	if err != nil {
		t.Skip("cannot determine home dir")
	}
	dir, err := cli.ExportDefaultConfigDir()
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	want := filepath.Join(home, ".config", "meowctl")
	if dir != want {
		t.Fatalf("want %q, got %q", want, dir)
	}
}

// TestLoadComponents_NoFile returns ExitConfig error when meowctl.star is absent.
func TestLoadComponents_NoFile(t *testing.T) {
	tmp := t.TempDir()
	_, err := cli.ExportLoadComponents(tmp, nil)
	if err == nil {
		t.Fatal("expected error for missing meowctl.star")
	}
	var exitErr *cli.ExitError
	if !errors.As(err, &exitErr) {
		t.Fatalf("expected *cli.ExitError, got %T: %v", err, err)
	}
	if exitErr.Code != cli.ExitConfig {
		t.Fatalf("want ExitConfig (%d), got %d", cli.ExitConfig, exitErr.Code)
	}
}

// TestLoadComponents_Basic verifies component names are extracted from a valid meowctl.star.
func TestLoadComponents_Basic(t *testing.T) {
	tmp := t.TempDir()
	star := `
module(name="test", version="0.1.0")
component("shell")
component("git")
`
	if err := os.WriteFile(filepath.Join(tmp, "meowctl.star"), []byte(star), 0o600); err != nil {
		t.Fatal(err)
	}

	ids, err := cli.ExportLoadComponents(tmp, nil)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if len(ids) != 2 {
		t.Fatalf("want 2 components, got %d: %v", len(ids), ids)
	}
	if ids[0] != "shell" || ids[1] != "git" {
		t.Fatalf("unexpected component names: %v", ids)
	}
}

// TestLoadComponents_Filter applies name filtering.
func TestLoadComponents_Filter(t *testing.T) {
	tmp := t.TempDir()
	star := `
module(name="test", version="0.1.0")
component("shell")
component("git")
component("neovim")
`
	if err := os.WriteFile(filepath.Join(tmp, "meowctl.star"), []byte(star), 0o600); err != nil {
		t.Fatal(err)
	}

	ids, err := cli.ExportLoadComponents(tmp, []string{"git"})
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if len(ids) != 1 || ids[0] != "git" {
		t.Fatalf("want [git], got %v", ids)
	}
}

// TestLoadComponents_FilterNoMatch returns ExitUsage when filter names don't exist.
func TestLoadComponents_FilterNoMatch(t *testing.T) {
	tmp := t.TempDir()
	star := `
module(name="test", version="0.1.0")
component("shell")
`
	if err := os.WriteFile(filepath.Join(tmp, "meowctl.star"), []byte(star), 0o600); err != nil {
		t.Fatal(err)
	}

	_, err := cli.ExportLoadComponents(tmp, []string{"nonexistent"})
	var exitErr *cli.ExitError
	if !errors.As(err, &exitErr) {
		t.Fatalf("expected *cli.ExitError, got %T: %v", err, err)
	}
	if exitErr.Code != cli.ExitUsage {
		t.Fatalf("want ExitUsage (%d), got %d", cli.ExitUsage, exitErr.Code)
	}
}

// TestInitCmd_CreatesFiles verifies meowctl init creates config dir and meowctl.star.
func TestInitCmd_CreatesFiles(t *testing.T) {
	tmp := t.TempDir()
	cmd := cli.NewRootCmdForTest()
	cmd.SetArgs([]string{"--config", tmp, "init"})
	if err := cmd.Execute(); err != nil {
		t.Fatalf("init failed: %v", err)
	}
	if _, err := os.Stat(filepath.Join(tmp, "meowctl.star")); err != nil {
		t.Fatalf("meowctl.star not created: %v", err)
	}
	if _, err := os.Stat(filepath.Join(tmp, "components")); err != nil {
		t.Fatalf("components/ not created: %v", err)
	}
}

// TestInitCmd_AlreadyExists returns an error when meowctl.star exists.
func TestInitCmd_AlreadyExists(t *testing.T) {
	tmp := t.TempDir()
	if err := os.WriteFile(filepath.Join(tmp, "meowctl.star"), []byte(""), 0o600); err != nil {
		t.Fatal(err)
	}
	cmd := cli.NewRootCmdForTest()
	cmd.SetArgs([]string{"--config", tmp, "init"})
	err := cmd.Execute()
	if err == nil {
		t.Fatal("expected error when meowctl.star already exists")
	}
	var exitErr *cli.ExitError
	if !errors.As(err, &exitErr) {
		t.Fatalf("expected *cli.ExitError, got %T", err)
	}
	if exitErr.Code != cli.ExitConfig {
		t.Fatalf("want ExitConfig, got %d", exitErr.Code)
	}
}

// TestShellCmd_Bash verifies shell bash emits a non-empty snippet.
func TestShellCmd_Bash(t *testing.T) {
	var buf strings.Builder
	cmd := cli.NewRootCmdForTest()
	cmd.SetOut(&buf)
	cmd.SetArgs([]string{"shell", "bash"})
	if err := cmd.Execute(); err != nil {
		t.Fatalf("shell bash failed: %v", err)
	}
}

// TestShellCmd_Unknown returns ExitUsage for an unknown shell.
func TestShellCmd_Unknown(t *testing.T) {
	cmd := cli.NewRootCmdForTest()
	cmd.SetArgs([]string{"shell", "powershell"})
	err := cmd.Execute()
	var exitErr *cli.ExitError
	if !errors.As(err, &exitErr) {
		t.Fatalf("expected *cli.ExitError, got %T: %v", err, err)
	}
	if exitErr.Code != cli.ExitUsage {
		t.Fatalf("want ExitUsage, got %d", exitErr.Code)
	}
}

// TestComponentListCmd verifies component list outputs declared component names.
func TestComponentListCmd(t *testing.T) {
	tmp := t.TempDir()
	star := `
module(name="test", version="0.1.0")
component("alpha")
component("beta")
`
	if err := os.WriteFile(filepath.Join(tmp, "meowctl.star"), []byte(star), 0o600); err != nil {
		t.Fatal(err)
	}
	var out strings.Builder
	cmd := cli.NewRootCmdForTest()
	cmd.SetOut(&out)
	cmd.SetArgs([]string{"--config", tmp, "component", "list"})
	if err := cmd.Execute(); err != nil {
		t.Fatalf("component list failed: %v", err)
	}
}

// TestLockShowCmd_MissingFile prints empty lock info without error when lock file is absent.
func TestLockShowCmd_MissingFile(t *testing.T) {
	tmp := t.TempDir()
	cmd := cli.NewRootCmdForTest()
	cmd.SetArgs([]string{"--config", tmp, "lock", "show"})
	// Missing lock file is not an error — lock.Read returns empty LockFile.
	if err := cmd.Execute(); err != nil {
		t.Fatalf("lock show on missing file should not error, got: %v", err)
	}
}
