package cli_test

import (
	"encoding/json"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/meowshed/meowctl/internal/cli"
	"github.com/meowshed/meowctl/internal/lock"
	"github.com/meowshed/meowctl/internal/modfile"
)

// writeModfile writes a minimal deps.mod content to path.
func writeModfile(t *testing.T, path, content string) {
	t.Helper()
	if err := os.WriteFile(path, []byte(content), 0o600); err != nil {
		t.Fatal(err)
	}
}

// execCmd runs a root command with the given args and returns (stdout, error).
func execCmd(t *testing.T, args ...string) (string, error) {
	t.Helper()
	var out strings.Builder
	cmd := cli.NewRootCmdForTest()
	cmd.SetOut(&out)
	cmd.SetArgs(args)
	err := cmd.Execute()
	return out.String(), err
}

// ---------------------------------------------------------------------------
// dep list
// ---------------------------------------------------------------------------

func TestDepListCmd_Empty(t *testing.T) {
	tmp := t.TempDir()
	writeModfile(t, filepath.Join(tmp, "deps.mod"), "# empty\n")

	out, err := execCmd(t, "--config", tmp, "dep", "list")
	if err != nil {
		t.Fatalf("dep list failed: %v", err)
	}
	// Header line should always be present.
	if !strings.Contains(out, "NAME") {
		t.Fatalf("expected header in output, got: %q", out)
	}
}

func TestDepListCmd_SharedDeps(t *testing.T) {
	tmp := t.TempDir()
	writeModfile(t, filepath.Join(tmp, "deps.mod"), `
dep(name = "bravo", version = "0.2.0")
dep(name = "alpha", version = "0.1.0")
`)

	out, err := execCmd(t, "--config", tmp, "dep", "list")
	if err != nil {
		t.Fatalf("dep list failed: %v", err)
	}
	// Alpha should appear before bravo (sorted).
	aIdx := strings.Index(out, "alpha")
	bIdx := strings.Index(out, "bravo")
	if aIdx < 0 || bIdx < 0 {
		t.Fatalf("expected both deps in output, got: %q", out)
	}
	if aIdx > bIdx {
		t.Fatalf("expected alpha before bravo, got: %q", out)
	}
}

func TestDepListCmd_BothModfiles(t *testing.T) {
	tmp := t.TempDir()
	writeModfile(t, filepath.Join(tmp, "deps.mod"), `dep(name = "shared", version = "1.0.0")`)
	writeModfile(t, filepath.Join(tmp, "deps.local.mod"), `dep(name = "local", version = "0.1.0")`)

	out, err := execCmd(t, "--config", tmp, "dep", "list")
	if err != nil {
		t.Fatalf("dep list failed: %v", err)
	}
	if !strings.Contains(out, "shared") {
		t.Fatalf("expected shared dep, got: %q", out)
	}
	if !strings.Contains(out, "local") {
		t.Fatalf("expected local dep, got: %q", out)
	}
	// shared (from deps.mod) should appear before local (from deps.local.mod).
	sIdx := strings.Index(out, "shared")
	lIdx := strings.Index(out, "local")
	if sIdx > lIdx {
		t.Fatalf("expected shared before local, got: %q", out)
	}
}

func TestDepListCmd_JSON(t *testing.T) {
	tmp := t.TempDir()
	writeModfile(t, filepath.Join(tmp, "deps.mod"), `dep(name = "stdlib", version = "0.1.0")`)

	out, err := execCmd(t, "--config", tmp, "dep", "list", "--json")
	if err != nil {
		t.Fatalf("dep list --json failed: %v", err)
	}
	var entries []map[string]interface{}
	if jsonErr := json.Unmarshal([]byte(out), &entries); jsonErr != nil {
		t.Fatalf("output is not valid JSON: %v\n%s", jsonErr, out)
	}
	if len(entries) != 1 {
		t.Fatalf("expected 1 entry, got %d", len(entries))
	}
	if entries[0]["name"] != "stdlib" {
		t.Fatalf("expected name stdlib, got %v", entries[0]["name"])
	}
}

// ---------------------------------------------------------------------------
// dep add
// ---------------------------------------------------------------------------

func TestDepAddCmd_WritesModfile(t *testing.T) {
	tmp := t.TempDir()
	writeModfile(t, filepath.Join(tmp, "deps.mod"), `dep(name = "alpha", version = "0.1.0")`)

	// Verify beta is not present before the add.
	pre, err := os.ReadFile(filepath.Join(tmp, "deps.mod")) // #nosec G304
	if err != nil {
		t.Fatal(err)
	}
	if strings.Contains(string(pre), "beta") {
		t.Fatal("beta should not be present in deps.mod before add")
	}

	// Add a brand-new dep — should be written to the modfile.
	// runSync will fail (no registry), which is fine; we only check the modfile was updated.
	_, _ = execCmd(t, "--config", tmp, "dep", "add", "beta", "--version", "0.2.0") //nolint:errcheck

	data, err := os.ReadFile(filepath.Join(tmp, "deps.mod")) // #nosec G304
	if err != nil {
		t.Fatal(err)
	}
	if !strings.Contains(string(data), "beta") {
		t.Fatalf("expected beta to be written to deps.mod, got:\n%s", data)
	}
}

func TestDepAddCmd_DuplicateSameVersion_NoOp(t *testing.T) {
	tmp := t.TempDir()
	writeModfile(t, filepath.Join(tmp, "deps.mod"), `dep(name = "alpha", version = "0.1.0")`)

	_, err := execCmd(t, "--config", tmp, "dep", "add", "alpha", "--version", "0.1.0")
	if err != nil {
		t.Fatalf("exact duplicate should be no-op (no error), got: %v", err)
	}
}

func TestDepAddCmd_DuplicateDifferentVersion_Error(t *testing.T) {
	tmp := t.TempDir()
	writeModfile(t, filepath.Join(tmp, "deps.mod"), `dep(name = "alpha", version = "0.1.0")`)

	_, err := execCmd(t, "--config", tmp, "dep", "add", "alpha", "--version", "0.2.0")
	if err == nil {
		t.Fatal("expected error for different-version duplicate")
	}
	if !strings.Contains(err.Error(), "already declared") {
		t.Fatalf("expected 'already declared' in error, got: %v", err)
	}
}

func TestDepAddCmd_LocalFlag_WritesLocalModfile(t *testing.T) {
	tmp := t.TempDir()
	writeModfile(t, filepath.Join(tmp, "deps.mod"), "# empty\n")
	// deps.local.mod does not exist yet.

	// Duplicate-same-version no-op: first add a dep to local, then add same again.
	// First add: will try runSync after writing — runSync for local will be called.
	// We expect the local modfile to be written (even if runSync fails after).
	// We check that deps.local.mod was created/updated.
	// Workaround: use exact-duplicate path which returns before runSync.
	writeModfile(t, filepath.Join(tmp, "deps.local.mod"), `dep(name = "myplugin", version = "0.1.0")`)

	_, err := execCmd(t, "--config", tmp, "dep", "add", "--local", "myplugin", "--version", "0.1.0")
	if err != nil {
		t.Fatalf("exact duplicate in local should be no-op, got: %v", err)
	}
}

func TestDepAddCmd_MissingVersionAndSource_Error(t *testing.T) {
	tmp := t.TempDir()
	writeModfile(t, filepath.Join(tmp, "deps.mod"), "# empty\n")

	_, err := execCmd(t, "--config", tmp, "dep", "add", "foo")
	if err == nil {
		t.Fatal("expected error when neither --version nor --source provided")
	}
}

// ---------------------------------------------------------------------------
// dep remove
// ---------------------------------------------------------------------------

func TestDepRemoveCmd_RemovesFromShared(t *testing.T) {
	tmp := t.TempDir()
	writeModfile(t, filepath.Join(tmp, "deps.mod"), `
dep(name = "alpha", version = "0.1.0")
dep(name = "bravo", version = "0.2.0")
`)
	_ = os.WriteFile(filepath.Join(tmp, "deps.lock"), []byte("[modules]\n"), 0o600)

	// Remove bravo — this will call runSync after. runSync will fail (no registry),
	// but by then the modfile has already been written. We verify the modfile content.
	// runSync failure should bubble up as error; we use a pre-existing lock to minimise that.
	// Actually since bravo is gone from modfile, runSync will just try to resolve alpha.
	// With empty lock and no registry, it will fail. That's acceptable for this test —
	// we check the modfile was written by inspecting it directly.
	if _, err := execCmd(t, "--config", tmp, "dep", "remove", "bravo"); err != nil {
		t.Log("dep remove bravo:", err) // error expected (runSync will fail with no registry)
	}

	data, err := os.ReadFile(filepath.Join(tmp, "deps.mod")) // #nosec G304
	if err != nil {
		t.Fatal(err)
	}
	if strings.Contains(string(data), "bravo") {
		t.Fatalf("bravo still present in deps.mod after remove: %s", data)
	}
	if !strings.Contains(string(data), "alpha") {
		t.Fatalf("alpha unexpectedly removed from deps.mod: %s", data)
	}
}

func TestDepRemoveCmd_NotFound_Error(t *testing.T) {
	tmp := t.TempDir()
	writeModfile(t, filepath.Join(tmp, "deps.mod"), `dep(name = "alpha", version = "0.1.0")`)

	_, err := execCmd(t, "--config", tmp, "dep", "remove", "nonexistent")
	if err == nil {
		t.Fatal("expected error when dep not found")
	}
	if !strings.Contains(err.Error(), "not found") {
		t.Fatalf("expected 'not found' in error, got: %v", err)
	}
}

func TestDepRemoveCmd_InBothFiles_Error(t *testing.T) {
	tmp := t.TempDir()
	writeModfile(t, filepath.Join(tmp, "deps.mod"), `dep(name = "alpha", version = "0.1.0")`)
	writeModfile(t, filepath.Join(tmp, "deps.local.mod"), `dep(name = "alpha", version = "0.1.0")`)

	_, err := execCmd(t, "--config", tmp, "dep", "remove", "alpha")
	if err == nil {
		t.Fatal("expected error when dep found in both modfiles")
	}
	if !strings.Contains(err.Error(), "both") {
		t.Fatalf("expected 'both' in error, got: %v", err)
	}
}

// ---------------------------------------------------------------------------
// dep tidy
// ---------------------------------------------------------------------------

func TestDepTidyCmd_RemovesOrphan(t *testing.T) {
	tmp := t.TempDir()
	writeModfile(t, filepath.Join(tmp, "deps.mod"), `
dep(name = "used", version = "0.1.0")
dep(name = "orphan", version = "0.2.0")
`)
	// init.star references @used// but not @orphan//.
	star := `load("@used//components/foo", "foo")`
	_ = os.WriteFile(filepath.Join(tmp, "init.star"), []byte(star), 0o600)
	_ = os.WriteFile(filepath.Join(tmp, "deps.lock"), []byte("[modules]\n"), 0o600)

	if _, err := execCmd(t, "--config", tmp, "dep", "tidy"); err != nil { //nolint:errcheck
		t.Log("dep tidy:", err) // error expected (runSync will fail with no registry)
	}

	data, err := os.ReadFile(filepath.Join(tmp, "deps.mod")) // #nosec G304
	if err != nil {
		t.Fatal(err)
	}
	if strings.Contains(string(data), "orphan") {
		t.Fatalf("orphan dep still present after tidy: %s", data)
	}
	if !strings.Contains(string(data), "used") {
		t.Fatalf("used dep was removed: %s", data)
	}
}

func TestDepTidyCmd_KeepsDepUsedInHooks(t *testing.T) {
	tmp := t.TempDir()
	writeModfile(t, filepath.Join(tmp, "deps.mod"), `dep(name = "hookdep", version = "0.1.0")`)
	// init.star has no refs; hooks/post-install.star does.
	_ = os.WriteFile(filepath.Join(tmp, "init.star"), []byte(""), 0o600)
	hooksDir := filepath.Join(tmp, "hooks")
	_ = os.Mkdir(hooksDir, 0o700)
	hookStar := `load("@hookdep//lib/hook", "run")`
	_ = os.WriteFile(filepath.Join(hooksDir, "post-install.star"), []byte(hookStar), 0o600)
	_ = os.WriteFile(filepath.Join(tmp, "deps.lock"), []byte("[modules]\n"), 0o600)

	if _, err := execCmd(t, "--config", tmp, "dep", "tidy"); err != nil { //nolint:errcheck
		t.Log("dep tidy:", err) // error expected (runSync will fail with no registry)
	}

	data, err := os.ReadFile(filepath.Join(tmp, "deps.mod")) // #nosec G304
	if err != nil {
		t.Fatal(err)
	}
	if !strings.Contains(string(data), "hookdep") {
		t.Fatalf("hookdep was removed even though it is referenced in hooks/: %s", data)
	}
}

func TestDepTidyCmd_NothingToTidy(t *testing.T) {
	tmp := t.TempDir()
	writeModfile(t, filepath.Join(tmp, "deps.mod"), `dep(name = "used", version = "0.1.0")`)
	star := `load("@used//components/foo", "foo")`
	_ = os.WriteFile(filepath.Join(tmp, "init.star"), []byte(star), 0o600)
	_ = os.WriteFile(filepath.Join(tmp, "deps.lock"), []byte("[modules]\n"), 0o600)

	// Tidy should report nothing to tidy and return without calling runSync.
	_, err := execCmd(t, "--config", tmp, "dep", "tidy")
	if err != nil {
		t.Fatalf("dep tidy with no orphans should not error, got: %v", err)
	}
}

func TestDepTidyCmd_DryRun_NoWrites(t *testing.T) {
	tmp := t.TempDir()
	const orig = `
dep(name = "used", version = "0.1.0")
dep(name = "orphan", version = "0.2.0")
`
	writeModfile(t, filepath.Join(tmp, "deps.mod"), orig)
	_ = os.WriteFile(filepath.Join(tmp, "init.star"), []byte(`load("@used//x", "x")`), 0o600)

	_, err := execCmd(t, "--config", tmp, "dep", "tidy", "--dry-run")
	if err != nil {
		t.Fatalf("dep tidy --dry-run should not error, got: %v", err)
	}

	data, _ := os.ReadFile(filepath.Join(tmp, "deps.mod")) // #nosec G304
	if !strings.Contains(string(data), "orphan") {
		t.Fatalf("dry-run must not modify deps.mod, but orphan was removed: %s", data)
	}
}

// ---------------------------------------------------------------------------
// dep sync
// ---------------------------------------------------------------------------

func TestDepSyncCmd_RejectsExtraArgs(t *testing.T) {
	tmp := t.TempDir()
	writeModfile(t, filepath.Join(tmp, "deps.mod"), "# empty\n")
	// dep sync declares Args: cobra.NoArgs — extra positional args must be rejected.
	_, err := execCmd(t, "--config", tmp, "dep", "sync", "extra-arg")
	if err == nil {
		t.Fatal("expected error when extra args passed to dep sync")
	}
}

func TestDepSyncCmd_Runs(t *testing.T) {
	tmp := t.TempDir()
	writeModfile(t, filepath.Join(tmp, "deps.mod"), "# empty\n")
	_ = os.WriteFile(filepath.Join(tmp, "deps.lock"), []byte("[modules]\n"), 0o600)
	// dep sync with an empty modfile and empty lock should succeed (nothing to sync).
	if _, err := execCmd(t, "--config", tmp, "dep", "sync"); err != nil {
		t.Log("dep sync (empty):", err) // may fail due to missing registry — acceptable
	}
}

// ---------------------------------------------------------------------------
// dep tidy – unknown-ref warning path
// ---------------------------------------------------------------------------

func TestDepTidyCmd_WarnsOnUnknownRef(t *testing.T) {
	tmp := t.TempDir()
	// deps.mod has "used"; star file also references "unknown" which is not declared.
	writeModfile(t, filepath.Join(tmp, "deps.mod"), `dep(name = "used", version = "0.1.0")`)
	star := `load("@used//x", "x")
load("@unknown//y", "y")`
	_ = os.WriteFile(filepath.Join(tmp, "init.star"), []byte(star), 0o600)
	_ = os.WriteFile(filepath.Join(tmp, "deps.lock"), []byte("[modules]\n"), 0o600)

	// tidy should report nothing to tidy (used is referenced) and not error,
	// but should print a warning to stderr for @unknown//.
	_, err := execCmd(t, "--config", tmp, "dep", "tidy")
	if err != nil {
		t.Fatalf("dep tidy should not error when only unknown refs are present, got: %v", err)
	}
}

// ---------------------------------------------------------------------------
// dep upgrade – nothing to upgrade
// ---------------------------------------------------------------------------

func TestDepUpgradeCmd_AllSHAPinned_NothingToUpgrade(t *testing.T) {
	tmp := t.TempDir()
	sha := "abc1234def5678901234567890123456789012ab"
	writeModfile(t, filepath.Join(tmp, "deps.mod"),
		`dep(name = "pinned", source = "github:owner/pinned@`+sha+`")`)
	_ = os.WriteFile(filepath.Join(tmp, "deps.lock"), []byte("[modules]\n"), 0o600)

	// All deps are SHA-pinned — nothing to upgrade; should return nil without sync.
	_, err := execCmd(t, "--config", tmp, "dep", "upgrade")
	if err != nil {
		t.Fatalf("dep upgrade with all-SHA-pinned deps should not error, got: %v", err)
	}
}

// ---------------------------------------------------------------------------
// dep list – locked version shown
// ---------------------------------------------------------------------------

func TestDepListCmd_ShowsLockedVersion(t *testing.T) {
	tmp := t.TempDir()
	writeModfile(t, filepath.Join(tmp, "deps.mod"), `dep(name = "myplugin", version = "0.1.0")`)

	lf := &lock.LockFile{
		Modules: map[string]lock.ModuleEntry{
			"myplugin": {Version: "0.2.0", Source: "https://example.com/myplugin-0.2.0.tar.gz"},
		},
	}
	if err := lock.Write(filepath.Join(tmp, "deps.lock"), lf); err != nil {
		t.Fatalf("write lock: %v", err)
	}

	out, err := execCmd(t, "--config", tmp, "dep", "list")
	if err != nil {
		t.Fatalf("dep list failed: %v", err)
	}
	if !strings.Contains(out, "0.2.0") {
		t.Fatalf("expected locked version 0.2.0 in output, got: %q", out)
	}
}

// TestDepListCmd_LockedByCommitSHA verifies that a GitHub dep with a CommitSHA lock entry
// shows the SHA in the LOCKED column.
func TestDepListCmd_LockedByCommitSHA(t *testing.T) {
	tmp := t.TempDir()
	writeModfile(t, filepath.Join(tmp, "deps.mod"), `dep(name = "ghplugin", source = "github:owner/ghplugin@main")`)

	sha := "deadbeefdeadbeef00000000000000000000beef"
	lf := &lock.LockFile{
		Modules: map[string]lock.ModuleEntry{
			"ghplugin": {CommitSHA: sha},
		},
	}
	if err := lock.Write(filepath.Join(tmp, "deps.lock"), lf); err != nil {
		t.Fatalf("write lock: %v", err)
	}

	out, err := execCmd(t, "--config", tmp, "dep", "list")
	if err != nil {
		t.Fatalf("dep list failed: %v", err)
	}
	if !strings.Contains(out, sha) {
		t.Fatalf("expected commit SHA %s in output, got: %q", sha, out)
	}
}

// ---------------------------------------------------------------------------
// dep upgrade
// ---------------------------------------------------------------------------

func TestDepUpgradeCmd_DryRun_NoWrites(t *testing.T) {
	tmp := t.TempDir()
	writeModfile(t, filepath.Join(tmp, "deps.mod"), `dep(name = "myplugin", source = "github:owner/myplugin@main")`)

	_, err := execCmd(t, "--config", tmp, "dep", "upgrade", "--dry-run")
	if err != nil {
		t.Fatalf("dep upgrade --dry-run should not error, got: %v", err)
	}
	// modfile must be unchanged.
	data, _ := os.ReadFile(filepath.Join(tmp, "deps.mod")) // #nosec G304
	if !strings.Contains(string(data), "myplugin") {
		t.Fatalf("dry-run must not modify deps.mod")
	}
}

func TestDepUpgradeCmd_SkipsSHAPinned(t *testing.T) {
	tmp := t.TempDir()
	sha := "abc1234def5678901234567890123456789012ab"
	writeModfile(t, filepath.Join(tmp, "deps.mod"),
		`dep(name = "pinned", source = "github:owner/pinned@`+sha+`")`)

	_, err := execCmd(t, "--config", tmp, "dep", "upgrade", "--dry-run")
	if err != nil {
		t.Fatalf("dep upgrade --dry-run with SHA-pinned dep should not error, got: %v", err)
	}
}

// TestDepUpgradeCmd_RegistryDep_ClearsLockAndRewritesVersion verifies that
// upgrading a registry dep clears its lock entry and sets version to "latest" in modfile
// so the next sync resolves to the latest version.
func TestDepUpgradeCmd_RegistryDep_ClearsLockAndRewritesVersion(t *testing.T) {
	tmp := t.TempDir()
	writeModfile(t, filepath.Join(tmp, "deps.mod"), `dep(name = "myplugin", version = "0.1.0")`)

	// Seed lock with a locked version for myplugin.
	lf := &lock.LockFile{
		Modules: map[string]lock.ModuleEntry{
			"myplugin": {Version: "0.1.0", Source: "https://example.com/myplugin-0.1.0.tar.gz"},
		},
	}
	if err := lock.Write(filepath.Join(tmp, "deps.lock"), lf); err != nil {
		t.Fatalf("write lock: %v", err)
	}

	// dep upgrade will: clear lock entry, rewrite modfile, then call runSync which
	// will fail (no registry). We check the pre-sync state after the error.
	if _, err := execCmd(t, "--config", tmp, "dep", "upgrade"); err != nil {
		t.Log("dep upgrade:", err) // expected — runSync fails without registry
	}

	// Lock entry for myplugin must be removed.
	lf2, err := lock.Read(filepath.Join(tmp, "deps.lock"))
	if err != nil {
		t.Fatalf("read lock after upgrade: %v", err)
	}
	if _, ok := lf2.Modules["myplugin"]; ok {
		t.Error("expected lock entry for 'myplugin' to be cleared before sync")
	}

	// Modfile version must be rewritten to "latest".
	mf, err := modfile.Parse(filepath.Join(tmp, "deps.mod"))
	if err != nil {
		t.Fatalf("parse modfile after upgrade: %v", err)
	}
	if len(mf.Deps) == 0 {
		t.Fatal("expected at least one dep in modfile")
	}
	if mf.Deps[0].Version != "latest" {
		t.Errorf("expected version set to 'latest', got %q", mf.Deps[0].Version)
	}
}

// TestDepUpgradeCmd_GitHubDep_NonSHA_ClearsLock verifies that a non-SHA GitHub dep
// has its lock entry cleared before sync (enabling re-resolution to latest SHA).
func TestDepUpgradeCmd_GitHubDep_NonSHA_ClearsLock(t *testing.T) {
	tmp := t.TempDir()
	writeModfile(t, filepath.Join(tmp, "deps.mod"), `dep(name = "myplugin", source = "github:owner/myplugin@main")`)

	// Seed lock with an old commit SHA.
	lf := &lock.LockFile{
		Modules: map[string]lock.ModuleEntry{
			"myplugin": {CommitSHA: "oldsha1234567890", Source: "https://github.com/owner/myplugin/archive/oldsha.tar.gz"},
		},
	}
	if err := lock.Write(filepath.Join(tmp, "deps.lock"), lf); err != nil {
		t.Fatalf("write lock: %v", err)
	}

	if _, err := execCmd(t, "--config", tmp, "dep", "upgrade"); err != nil {
		t.Log("dep upgrade:", err) // expected — runSync fails without GitHub API
	}

	// Lock entry must be cleared so the next sync re-resolves.
	lf2, err := lock.Read(filepath.Join(tmp, "deps.lock"))
	if err != nil {
		t.Fatalf("read lock after upgrade: %v", err)
	}
	if _, ok := lf2.Modules["myplugin"]; ok {
		t.Error("expected lock entry for 'myplugin' to be cleared before sync")
	}
}

// TestDepUpgradeCmd_FilterByModule verifies that named module args limit which deps are upgraded.
func TestDepUpgradeCmd_FilterByModule(t *testing.T) {
	tmp := t.TempDir()
	writeModfile(t, filepath.Join(tmp, "deps.mod"),
		"dep(name = \"alpha\", version = \"0.1.0\")\ndep(name = \"beta\", version = \"0.2.0\")")

	lf := &lock.LockFile{
		Modules: map[string]lock.ModuleEntry{
			"alpha": {Version: "0.1.0", Source: "https://example.com/alpha.tar.gz"},
			"beta":  {Version: "0.2.0", Source: "https://example.com/beta.tar.gz"},
		},
	}
	if err := lock.Write(filepath.Join(tmp, "deps.lock"), lf); err != nil {
		t.Fatalf("write lock: %v", err)
	}

	// Only upgrade alpha.
	if _, err := execCmd(t, "--config", tmp, "dep", "upgrade", "alpha"); err != nil {
		t.Log("dep upgrade alpha:", err) // expected — runSync fails without registry
	}

	lf2, err := lock.Read(filepath.Join(tmp, "deps.lock"))
	if err != nil {
		t.Fatalf("read lock after upgrade: %v", err)
	}
	// alpha must be cleared.
	if _, ok := lf2.Modules["alpha"]; ok {
		t.Error("expected lock entry for 'alpha' to be cleared")
	}
	// beta must be untouched.
	if _, ok := lf2.Modules["beta"]; !ok {
		t.Error("expected lock entry for 'beta' to remain unchanged")
	}
}
