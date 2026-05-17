package cli

import (
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/BurntSushi/toml"
	"github.com/meowshed/meowctl/internal/lock"
	starlarkpkg "github.com/meowshed/meowctl/internal/starlark"
)

func TestReadPkgsLock_MissingFile(t *testing.T) {
	tmp := t.TempDir()
	lf, err := readPkgsLock(tmp, configPkgsLockFile)
	if err != nil {
		t.Fatalf("expected nil error for missing file, got: %v", err)
	}
	if lf.Packages != nil {
		t.Fatalf("expected nil Packages for empty lock, got: %v", lf.Packages)
	}
}

func TestWritePkgsLockFile_RoundTrip(t *testing.T) {
	tmp := t.TempDir()
	lf := lock.LockFile{
		Packages: map[string]lock.ManagerPackages{
			"brew": {"git": {Requested: "2.40.0", Installed: "2.40.0"}},
		},
	}
	if err := writePkgsLockFile(tmp, configPkgsLockFile, lf); err != nil {
		t.Fatalf("write: %v", err)
	}

	var got lock.LockFile
	if _, err := toml.DecodeFile(filepath.Join(tmp, configPkgsLockFile), &got); err != nil { // #nosec G304
		t.Fatalf("decode: %v", err)
	}
	if got.Packages["brew"]["git"].Requested != "2.40.0" {
		t.Errorf("got Requested=%q, want 2.40.0", got.Packages["brew"]["git"].Requested)
	}
	if got.Packages["brew"]["git"].Installed != "2.40.0" {
		t.Errorf("got Installed=%q, want 2.40.0", got.Packages["brew"]["git"].Installed)
	}
}

func TestReadPkgsLock_BackwardsCompat_NoPackages(t *testing.T) {
	// A lock file written without [packages] should parse cleanly with nil Packages.
	tmp := t.TempDir()
	path := filepath.Join(tmp, configPkgsLockFile)
	if err := os.WriteFile(path, []byte("[meta]\ngenerated-by = \"test\"\n"), 0o600); err != nil {
		t.Fatalf("write: %v", err)
	}
	lf, err := readPkgsLock(tmp, configPkgsLockFile)
	if err != nil {
		t.Fatalf("expected no error, got: %v", err)
	}
	if len(lf.Packages) != 0 {
		t.Errorf("expected empty Packages, got: %v", lf.Packages)
	}
}

func TestAppendPkgsLock_SharedComponent(t *testing.T) {
	tmp := t.TempDir()
	pins := map[string][]starlarkpkg.PkgDecl{
		"mycomp": {{Manager: "brew", Name: "ripgrep", Version: "13.0.0"}},
	}
	localSet := map[string]bool{} // mycomp is shared

	if err := appendPkgsLock(tmp, pins, localSet); err != nil {
		t.Fatalf("appendPkgsLock: %v", err)
	}

	// pkgs.lock should have the entry
	lf, err := readPkgsLock(tmp, configPkgsLockFile)
	if err != nil {
		t.Fatalf("read pkgs.lock: %v", err)
	}
	entry, ok := lf.Packages["brew"]["ripgrep"]
	if !ok {
		t.Fatal("expected brew/ripgrep in pkgs.lock")
	}
	if entry.Requested != "13.0.0" {
		t.Errorf("Requested=%q, want 13.0.0", entry.Requested)
	}

	// pkgs.local.lock should not have any entry
	localLF, err := readPkgsLock(tmp, configPkgsLocalLockFile)
	if err != nil {
		t.Fatalf("read pkgs.local.lock: %v", err)
	}
	if len(localLF.Packages) != 0 {
		t.Errorf("expected empty pkgs.local.lock, got: %v", localLF.Packages)
	}
}

func TestAppendPkgsLock_LocalComponent(t *testing.T) {
	tmp := t.TempDir()
	pins := map[string][]starlarkpkg.PkgDecl{
		"localcomp": {{Manager: "mise", Name: "node", Version: "20.0.0"}},
	}
	localSet := map[string]bool{"localcomp": true}

	if err := appendPkgsLock(tmp, pins, localSet); err != nil {
		t.Fatalf("appendPkgsLock: %v", err)
	}

	// pkgs.local.lock should have the entry
	localLF, err := readPkgsLock(tmp, configPkgsLocalLockFile)
	if err != nil {
		t.Fatalf("read pkgs.local.lock: %v", err)
	}
	entry, ok := localLF.Packages["mise"]["node"]
	if !ok {
		t.Fatal("expected mise/node in pkgs.local.lock")
	}
	if entry.Installed != "20.0.0" {
		t.Errorf("Installed=%q, want 20.0.0", entry.Installed)
	}

	// pkgs.lock should be empty
	sharedLF, err := readPkgsLock(tmp, configPkgsLockFile)
	if err != nil {
		t.Fatalf("read pkgs.lock: %v", err)
	}
	if len(sharedLF.Packages) != 0 {
		t.Errorf("expected empty pkgs.lock, got: %v", sharedLF.Packages)
	}
}

func TestAppendPkgsLock_MixedComponents(t *testing.T) {
	tmp := t.TempDir()
	pins := map[string][]starlarkpkg.PkgDecl{
		"sharedcomp": {{Manager: "brew", Name: "jq", Version: "1.7"}},
		"localcomp":  {{Manager: "brew", Name: "bat", Version: "0.24.0"}},
	}
	localSet := map[string]bool{"localcomp": true}

	if err := appendPkgsLock(tmp, pins, localSet); err != nil {
		t.Fatalf("appendPkgsLock: %v", err)
	}

	sharedLF, _ := readPkgsLock(tmp, configPkgsLockFile)
	localLF, _ := readPkgsLock(tmp, configPkgsLocalLockFile)

	if _, ok := sharedLF.Packages["brew"]["jq"]; !ok {
		t.Error("expected jq in pkgs.lock")
	}
	if _, ok := sharedLF.Packages["brew"]["bat"]; ok {
		t.Error("bat should NOT be in pkgs.lock (it is local)")
	}
	if _, ok := localLF.Packages["brew"]["bat"]; !ok {
		t.Error("expected bat in pkgs.local.lock")
	}
	if _, ok := localLF.Packages["brew"]["jq"]; ok {
		t.Error("jq should NOT be in pkgs.local.lock (it is shared)")
	}
}

func TestAppendPkgsLock_UpdatesExistingEntry(t *testing.T) {
	tmp := t.TempDir()

	// Write initial lock with old version
	initial := lock.LockFile{
		Packages: map[string]lock.ManagerPackages{
			"brew": {"git": {Requested: "2.39.0", Installed: "2.39.0"}},
		},
	}
	if err := writePkgsLockFile(tmp, configPkgsLockFile, initial); err != nil {
		t.Fatalf("write initial: %v", err)
	}

	// Now append updated version
	pins := map[string][]starlarkpkg.PkgDecl{
		"gitcomp": {{Manager: "brew", Name: "git", Version: "2.40.0"}},
	}
	if err := appendPkgsLock(tmp, pins, map[string]bool{}); err != nil {
		t.Fatalf("appendPkgsLock: %v", err)
	}

	lf, _ := readPkgsLock(tmp, configPkgsLockFile)
	if lf.Packages["brew"]["git"].Requested != "2.40.0" {
		t.Errorf("expected updated Requested=2.40.0, got %q", lf.Packages["brew"]["git"].Requested)
	}
}

// ---------------------------------------------------------------------------
// checkModuleDeclared
// ---------------------------------------------------------------------------

func TestCheckModuleDeclared_BareNameAlwaysOK(t *testing.T) {
	tmp := t.TempDir()
	// Bare names (no "@") need no module declaration.
	if err := checkModuleDeclared(tmp, "mycomp"); err != nil {
		t.Fatalf("bare component name should not require module declaration, got: %v", err)
	}
}

func TestCheckModuleDeclared_DeclaredInShared(t *testing.T) {
	tmp := t.TempDir()
	writeModfileRaw(t, filepath.Join(tmp, configModFile), `dep(name = "myplugin", version = "0.1.0")`)

	if err := checkModuleDeclared(tmp, "@myplugin//comp/foo"); err != nil {
		t.Fatalf("module declared in deps.mod should pass, got: %v", err)
	}
}

func TestCheckModuleDeclared_DeclaredInLocal(t *testing.T) {
	tmp := t.TempDir()
	writeModfileRaw(t, filepath.Join(tmp, configLocalModFile), `dep(name = "myplugin", version = "0.1.0")`)

	if err := checkModuleDeclared(tmp, "@myplugin//comp/foo"); err != nil {
		t.Fatalf("module declared in deps.local.mod should pass, got: %v", err)
	}
}

func TestCheckModuleDeclared_UndeclaredModule_Error(t *testing.T) {
	tmp := t.TempDir()
	writeModfileRaw(t, filepath.Join(tmp, configModFile), `dep(name = "other", version = "0.1.0")`)

	err := checkModuleDeclared(tmp, "@myplugin//comp/foo")
	if err == nil {
		t.Fatal("expected error for undeclared module, got nil")
	}
	if !strings.Contains(err.Error(), "myplugin") {
		t.Errorf("expected module name in error, got: %v", err)
	}
}

// writeModfileRaw writes raw Starlark content to path, creating parent dirs as needed.
func writeModfileRaw(t *testing.T, path, content string) {
	t.Helper()
	if err := os.MkdirAll(filepath.Dir(path), 0o700); err != nil {
		t.Fatalf("writeModfileRaw: mkdir: %v", err)
	}
	if err := os.WriteFile(path, []byte(content+"\n"), 0o600); err != nil { // #nosec G306
		t.Fatalf("writeModfileRaw: write: %v", err)
	}
}
