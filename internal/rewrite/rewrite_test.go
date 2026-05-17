package rewrite_test

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/meowshed/meowctl/internal/rewrite"
)

func writeFile(t *testing.T, dir, name, content string) string {
	t.Helper()
	path := filepath.Join(dir, name)
	if err := os.WriteFile(path, []byte(content), 0o600); err != nil {
		t.Fatalf("writeFile: %v", err)
	}
	return path
}

func readFile(t *testing.T, path string) string {
	t.Helper()
	data, err := os.ReadFile(path) // #nosec G304 -- test helper; path is from t.TempDir()
	if err != nil {
		t.Fatalf("readFile: %v", err)
	}
	return string(data)
}

func TestSetDepVersion_Basic(t *testing.T) {
	tmp := t.TempDir()
	path := writeFile(t, tmp, "meowctl.mod", `module(name = "dotfiles", version = "0.1.0")
dep(name = "owner-repo", version = "v1.0.0")
`)
	if err := rewrite.SetDepVersion(path, "owner-repo", "v1.2.3"); err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	got := readFile(t, path)
	if !contains(got, `version = "v1.2.3"`) {
		t.Errorf("expected updated version in:\n%s", got)
	}
	if contains(got, `version = "v1.0.0"`) {
		t.Errorf("old version still present in:\n%s", got)
	}
}

func TestSetDepVersion_NotFound(t *testing.T) {
	tmp := t.TempDir()
	path := writeFile(t, tmp, "meowctl.mod", `dep(name = "other-repo", version = "v1.0.0")
`)
	err := rewrite.SetDepVersion(path, "owner-repo", "v2.0.0")
	if err == nil {
		t.Fatal("expected error when module not found")
	}
}

func TestSetDepVersion_FileNotFound(t *testing.T) {
	err := rewrite.SetDepVersion("/nonexistent/meowctl.mod", "foo", "v1.0.0")
	if err == nil {
		t.Fatal("expected error for missing file")
	}
}

func contains(s, substr string) bool {
	return len(s) >= len(substr) && (s == substr || len(s) > 0 && containsStr(s, substr))
}

func containsStr(s, substr string) bool {
	for i := 0; i <= len(s)-len(substr); i++ {
		if s[i:i+len(substr)] == substr {
			return true
		}
	}
	return false
}
