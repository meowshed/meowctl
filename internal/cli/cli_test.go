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

// TestLoadComponents_NoFile returns ExitConfig error when init.star is absent.
func TestLoadComponents_NoFile(t *testing.T) {
	tmp := t.TempDir()
	_, err := cli.ExportLoadComponents(tmp, nil)
	if err == nil {
		t.Fatal("expected error for missing init.star")
	}
	var exitErr *cli.ExitError
	if !errors.As(err, &exitErr) {
		t.Fatalf("expected *cli.ExitError, got %T: %v", err, err)
	}
	if exitErr.Code != cli.ExitConfig {
		t.Fatalf("want ExitConfig (%d), got %d", cli.ExitConfig, exitErr.Code)
	}
}

// TestLoadComponents_Basic verifies component names are extracted from a valid init.star.
func TestLoadComponents_Basic(t *testing.T) {
	tmp := t.TempDir()
	star := `
component("shell")
component("git")
`
	if err := os.WriteFile(filepath.Join(tmp, "init.star"), []byte(star), 0o600); err != nil {
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
component("shell")
component("git")
component("neovim")
`
	if err := os.WriteFile(filepath.Join(tmp, "init.star"), []byte(star), 0o600); err != nil {
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
component("shell")
`
	if err := os.WriteFile(filepath.Join(tmp, "init.star"), []byte(star), 0o600); err != nil {
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

// TestInitCmd_CreatesFiles verifies meowctl init creates config dir and init.star.
func TestInitCmd_CreatesFiles(t *testing.T) {
	tmp := t.TempDir()
	cmd := cli.NewRootCmdForTest()
	cmd.SetArgs([]string{"--config", tmp, "init"})
	if err := cmd.Execute(); err != nil {
		t.Fatalf("init failed: %v", err)
	}
	if _, err := os.Stat(filepath.Join(tmp, "init.star")); err != nil {
		t.Fatalf("init.star not created: %v", err)
	}
	if _, err := os.Stat(filepath.Join(tmp, "components")); err != nil {
		t.Fatalf("components/ not created: %v", err)
	}
}

// TestInitCmd_AlreadyExists returns an error when init.star exists.
func TestInitCmd_AlreadyExists(t *testing.T) {
	tmp := t.TempDir()
	if err := os.WriteFile(filepath.Join(tmp, "init.star"), []byte(""), 0o600); err != nil {
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
component("alpha")
component("beta")
`
	if err := os.WriteFile(filepath.Join(tmp, "init.star"), []byte(star), 0o600); err != nil {
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

// TestComputeToInstall_FullSet installs all declared components when none are installed.
func TestComputeToInstall_FullSet(t *testing.T) {
	order := []string{"brew", "shell", "git"}
	installed := map[string]bool{}
	scope := map[string]bool{}
	result := cli.ExportComputeToInstall(order, installed, scope, false)
	if len(result) != 3 {
		t.Fatalf("want 3 to install, got %d: %v", len(result), result)
	}
}

// TestComputeToInstall_NothingNew returns empty when all declared are already installed.
func TestComputeToInstall_NothingNew(t *testing.T) {
	order := []string{"brew", "shell"}
	installed := map[string]bool{"brew": true, "shell": true}
	scope := map[string]bool{}
	result := cli.ExportComputeToInstall(order, installed, scope, false)
	if len(result) != 0 {
		t.Fatalf("want nothing to install, got %v", result)
	}
}

// TestComputeToInstall_Scoped installs only components in the scope set.
func TestComputeToInstall_Scoped(t *testing.T) {
	order := []string{"brew", "shell", "git"}
	installed := map[string]bool{}
	scope := map[string]bool{"git": true}
	result := cli.ExportComputeToInstall(order, installed, scope, true)
	if len(result) != 1 || result[0] != "git" {
		t.Fatalf("want [git], got %v", result)
	}
}

// TestComputeToUninstall_RemovedFromInstalled returns components installed but not declared.
func TestComputeToUninstall_RemovedFromInstalled(t *testing.T) {
	// "old-tool" is installed but not declared — should be uninstalled.
	order := []string{"brew", "shell"}
	declared := map[string]bool{"brew": true, "shell": true}
	installed := map[string]bool{"brew": true, "shell": true, "old-tool": true}
	installedList := []string{"brew", "shell", "old-tool"}
	result := cli.ExportComputeToUninstall(order, declared, installed, installedList)
	if len(result) != 1 || result[0] != "old-tool" {
		t.Fatalf("want [old-tool], got %v", result)
	}
}

// TestComputeToUninstall_NothingToRemove returns empty when installed == declared.
func TestComputeToUninstall_NothingToRemove(t *testing.T) {
	order := []string{"brew", "shell"}
	declared := map[string]bool{"brew": true, "shell": true}
	installed := map[string]bool{"brew": true, "shell": true}
	installedList := []string{"brew", "shell"}
	result := cli.ExportComputeToUninstall(order, declared, installed, installedList)
	if len(result) != 0 {
		t.Fatalf("want nothing to uninstall, got %v", result)
	}
}

// TestComputeNewInstalled_FullApply returns allOrder as the new installed list.
func TestComputeNewInstalled_FullApply(t *testing.T) {
	allOrder := []string{"brew", "shell", "git"}
	installed := map[string]bool{"brew": true}
	toInstall := []string{"shell", "git"}
	result := cli.ExportComputeNewInstalled(false, allOrder, installed, toInstall)
	if len(result) != 3 {
		t.Fatalf("want 3 installed, got %d: %v", len(result), result)
	}
}

// TestComputeNewInstalled_ScopedApply merges newly installed into existing installed set.
func TestComputeNewInstalled_ScopedApply(t *testing.T) {
	allOrder := []string{"brew", "shell", "git"}
	installed := map[string]bool{"brew": true}
	toInstall := []string{"shell"}
	result := cli.ExportComputeNewInstalled(true, allOrder, installed, toInstall)
	// brew + shell; git not included (not in toInstall and not scoped)
	resultSet := make(map[string]bool, len(result))
	for _, r := range result {
		resultSet[r] = true
	}
	if !resultSet["brew"] || !resultSet["shell"] {
		t.Fatalf("want brew and shell in result, got %v", result)
	}
	if resultSet["git"] {
		t.Fatalf("git should not be in result, got %v", result)
	}
}

// TestInstalledLock_RoundTrip verifies write then read produces the same components.
func TestInstalledLock_RoundTrip(t *testing.T) {
	tmp := t.TempDir()
	components := []string{"git", "brew", "shell"}
	if err := cli.ExportWriteInstalledLock(tmp, components); err != nil {
		t.Fatalf("write: %v", err)
	}
	il, err := cli.ExportReadInstalledLock(tmp)
	if err != nil {
		t.Fatalf("read: %v", err)
	}
	// writeInstalledLock sorts; verify all names present
	if len(il.Components) != 3 {
		t.Fatalf("want 3 components, got %d: %v", len(il.Components), il.Components)
	}
	got := make(map[string]bool, len(il.Components))
	for _, c := range il.Components {
		got[c] = true
	}
	for _, want := range components {
		if !got[want] {
			t.Fatalf("missing component %q in lock: %v", want, il.Components)
		}
	}
}

// TestInstalledLock_MissingFile returns empty lock without error.
func TestInstalledLock_MissingFile(t *testing.T) {
	tmp := t.TempDir()
	il, err := cli.ExportReadInstalledLock(tmp)
	if err != nil {
		t.Fatalf("unexpected error for missing file: %v", err)
	}
	if len(il.Components) != 0 {
		t.Fatalf("want empty components, got %v", il.Components)
	}
}

// TestLoadComponents_Uninstall_SyntheticDepsExcluded verifies that synthetic
// (auto-discovered) transitive deps are included in the install-path order but
// excluded from the uninstall-path order. This prevents PM components like brew
// or mise from running their own uninstall hooks when only a dependent tool is
// being uninstalled.
func TestLoadComponents_Uninstall_SyntheticDepsExcluded(t *testing.T) {
	tmp := t.TempDir()
	// "tool" declares an after= dep on "pm", but "pm" is NOT declared in init.star.
	// "pm" is therefore synthetic — auto-discovered by expandWithFileDeps.
	star := `
component("tool", after=["pm"])
`
	if err := os.WriteFile(filepath.Join(tmp, "init.star"), []byte(star), 0o600); err != nil {
		t.Fatal(err)
	}

	// Install path: synthetic dep "pm" must appear before "tool".
	installIDs, err := cli.ExportLoadComponents(tmp, nil)
	if err != nil {
		t.Fatalf("install path: unexpected error: %v", err)
	}
	foundPM := false
	for _, id := range installIDs {
		if id == "pm" {
			foundPM = true
		}
	}
	if !foundPM {
		t.Fatalf("install path: expected synthetic dep 'pm' in order, got %v", installIDs)
	}

	// Uninstall path: synthetic dep "pm" must NOT appear — only declared "tool".
	uninstallIDs, err := cli.ExportLoadComponentsForUninstall(tmp, nil)
	if err != nil {
		t.Fatalf("uninstall path: unexpected error: %v", err)
	}
	for _, id := range uninstallIDs {
		if id == "pm" {
			t.Fatalf("uninstall path: synthetic dep 'pm' must not appear in order, got %v", uninstallIDs)
		}
	}
	if len(uninstallIDs) != 1 || uninstallIDs[0] != "tool" {
		t.Fatalf("uninstall path: expected only ['tool'], got %v", uninstallIDs)
	}
}
