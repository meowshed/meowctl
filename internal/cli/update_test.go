package cli

import (
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"testing"

	"github.com/meowshed/meowctl/internal/state"
)

func TestDiffConfigDirs(t *testing.T) {
	tmp := t.TempDir()
	staging := filepath.Join(tmp, "staging")
	current := filepath.Join(tmp, "current")
	_ = os.MkdirAll(staging, 0o700)
	_ = os.MkdirAll(current, 0o700)

	// Staging files.
	writeFile(t, filepath.Join(staging, "init.star"), "stage init")
	writeFile(t, filepath.Join(staging, "new.star"), "new file")
	writeFile(t, filepath.Join(staging, "same.star"), "same")
	writeFile(t, filepath.Join(staging, "local.star"), "staging local")
	writeFile(t, filepath.Join(staging, "deps.local.mod"), "staging local mod")

	// Current files.
	writeFile(t, filepath.Join(current, "init.star"), "current init")
	writeFile(t, filepath.Join(current, "same.star"), "same")
	writeFile(t, filepath.Join(current, "old.star"), "old file")
	writeFile(t, filepath.Join(current, "local.star"), "current local")
	writeFile(t, filepath.Join(current, "deps.local.mod"), "current local mod")

	changes, err := diffConfigDirs(staging, current)
	if err != nil {
		t.Fatalf("diffConfigDirs: %v", err)
	}

	want := map[string]changeOp{
		"init.star": opModify,
		"new.star":  opAdd,
	}
	got := make(map[string]changeOp, len(changes))
	for _, c := range changes {
		got[c.path] = c.op
	}

	if len(got) != len(want) {
		t.Fatalf("got %d changes, want %d: got=%v", len(got), len(want), got)
	}
	for p, op := range want {
		if got[p] != op {
			t.Errorf("%s: got op %q, want %q", p, got[p], op)
		}
	}
	if _, ok := got["local.star"]; ok {
		t.Error("local.star should be skipped")
	}
	if _, ok := got["deps.local.mod"]; ok {
		t.Error("deps.local.mod should be skipped")
	}
	if _, ok := got["old.star"]; ok {
		t.Error("old.star should not appear (only staging files are compared)")
	}
}

func TestApplyChange(t *testing.T) {
	tmp := t.TempDir()
	staging := filepath.Join(tmp, "staging")
	current := filepath.Join(tmp, "current")
	_ = os.MkdirAll(staging, 0o700)
	_ = os.MkdirAll(current, 0o700)

	writeFile(t, filepath.Join(staging, "foo.star"), "updated")
	sub := filepath.Join(staging, "sub", "bar.star")
	_ = os.MkdirAll(filepath.Dir(sub), 0o700)
	writeFile(t, sub, "nested")

	c := change{path: "foo.star", op: opModify}
	if err := applyChange(c, staging, current); err != nil {
		t.Fatalf("applyChange foo: %v", err)
	}
	assertFileContent(t, filepath.Join(current, "foo.star"), "updated")

	c = change{path: filepath.Join("sub", "bar.star"), op: opAdd}
	if err := applyChange(c, staging, current); err != nil {
		t.Fatalf("applyChange sub/bar: %v", err)
	}
	assertFileContent(t, filepath.Join(current, "sub", "bar.star"), "nested")
}

func TestIsProtectedFile(t *testing.T) {
	tests := []struct {
		path string
		want bool
	}{
		{"local.star", true},
		{"deps.local.mod", true},
		{"deps.local.lock", true},
		{"init.star", false},
		{"deps.mod", false},
		{"sub/local.star", true},
		{"sub/deps.local.mod", true},
	}
	for _, tc := range tests {
		if got := isProtectedFile(tc.path); got != tc.want {
			t.Errorf("isProtectedFile(%q) = %v, want %v", tc.path, got, tc.want)
		}
	}
}

func TestFilesEqual(t *testing.T) {
	tmp := t.TempDir()
	a := filepath.Join(tmp, "a")
	b := filepath.Join(tmp, "b")
	c := filepath.Join(tmp, "c")

	writeFile(t, a, "hello")
	writeFile(t, b, "hello")
	writeFile(t, c, "world")

	if same, err := filesEqual(a, b); err != nil || !same {
		t.Errorf("filesEqual(a,b) = %v, %v; want true, nil", same, err)
	}
	if same, err := filesEqual(a, c); err != nil || same {
		t.Errorf("filesEqual(a,c) = %v, %v; want false, nil", same, err)
	}
}

// TestRunUpdate_Success verifies the full update flow: tarball download, diff,
// apply, and protected-file preservation.
func TestRunUpdate_Success(t *testing.T) {
	// Build a tarball with updated files.
	tarball := makeTarball(t, map[string]string{
		"init.star":        "component(\"dummy\")\n",
		"deps.mod":         "module(name = \"test\", version = \"0.2.0\")\n",
		"components/a.txt": "updated a",
	})

	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
		_, _ = w.Write(tarball)
	}))
	defer srv.Close()

	configDir := t.TempDir()

	// Seed local config.
	_ = os.MkdirAll(filepath.Join(configDir, "components"), 0o700)
	writeFile(t, filepath.Join(configDir, "init.star"), "component(\"dummy\")\n")
	writeFile(t, filepath.Join(configDir, "deps.mod"), "module(name = \"test\", version = \"0.1.0\")\n")
	writeFile(t, filepath.Join(configDir, "components", "a.txt"), "original a")
	writeFile(t, filepath.Join(configDir, "local.star"), "# machine local\n")

	// Seed state.toml with repo URL.
	mgr := state.NewManager(filepath.Join(configDir, configStateFile))
	_ = mgr.Save(state.Sentinel{SchemaVersion: 1, RepoURL: srv.URL})

	cfg := runConfig{ConfigDir: configDir}
	if err := runUpdate(cfg, true); err != nil {
		t.Fatalf("runUpdate: %v", err)
	}

	// Tracked files updated.
	assertFileContent(t, filepath.Join(configDir, "init.star"), "component(\"dummy\")\n")
	assertFileContent(t, filepath.Join(configDir, "deps.mod"), "module(name = \"test\", version = \"0.2.0\")\n")
	assertFileContent(t, filepath.Join(configDir, "components", "a.txt"), "updated a")

	// Protected file untouched.
	assertFileContent(t, filepath.Join(configDir, "local.star"), "# machine local\n")

	// Staging dir cleaned up.
	if _, err := os.Lstat(filepath.Join(configDir, ".update-staging")); !os.IsNotExist(err) {
		t.Error("staging dir should be removed after update")
	}
}

// TestRunUpdate_NoChanges verifies that update is a no-op when upstream is
// identical to the local config.
func TestRunUpdate_NoChanges(t *testing.T) {
	tarball := makeTarball(t, map[string]string{
		"init.star": "same",
	})

	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
		_, _ = w.Write(tarball)
	}))
	defer srv.Close()

	configDir := t.TempDir()
	writeFile(t, filepath.Join(configDir, "init.star"), "same")

	mgr := state.NewManager(filepath.Join(configDir, configStateFile))
	_ = mgr.Save(state.Sentinel{SchemaVersion: 1, RepoURL: srv.URL})

	cfg := runConfig{ConfigDir: configDir}
	if err := runUpdate(cfg, true); err != nil {
		t.Fatalf("runUpdate: %v", err)
	}
}

// TestRunUpdate_DryRun verifies that --dry-run previews changes without
// writing them.
func TestRunUpdate_DryRun(t *testing.T) {
	tarball := makeTarball(t, map[string]string{
		"init.star": "new content",
	})

	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
		_, _ = w.Write(tarball)
	}))
	defer srv.Close()

	configDir := t.TempDir()
	writeFile(t, filepath.Join(configDir, "init.star"), "old content")

	mgr := state.NewManager(filepath.Join(configDir, configStateFile))
	_ = mgr.Save(state.Sentinel{SchemaVersion: 1, RepoURL: srv.URL})

	cfg := runConfig{ConfigDir: configDir, DryRun: true}
	if err := runUpdate(cfg, true); err != nil {
		t.Fatalf("runUpdate: %v", err)
	}

	// File should NOT be modified.
	assertFileContent(t, filepath.Join(configDir, "init.star"), "old content")
}

// TestRunUpdate_MissingRepoURL verifies that update fails gracefully when
// state.toml does not contain a repo_url.
func TestRunUpdate_MissingRepoURL(t *testing.T) {
	configDir := t.TempDir()
	mgr := state.NewManager(filepath.Join(configDir, configStateFile))
	_ = mgr.Save(state.Sentinel{SchemaVersion: 1})

	cfg := runConfig{ConfigDir: configDir}
	err := runUpdate(cfg, true)
	if err == nil {
		t.Fatal("expected error when repo_url is missing")
	}
}

func writeFile(t *testing.T, path, content string) {
	t.Helper()
	if err := os.WriteFile(path, []byte(content), 0o600); err != nil {
		t.Fatalf("writeFile(%q): %v", path, err)
	}
}

func assertFileContent(t *testing.T, path, want string) {
	t.Helper()
	got, err := os.ReadFile(path) // #nosec G304
	if err != nil {
		t.Fatalf("read %q: %v", path, err)
	}
	if string(got) != want {
		t.Errorf("%q content = %q, want %q", path, got, want)
	}
}
