package modfile_test

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/meowshed/meowctl/internal/modfile"
)

func TestParseBytes_ModuleDepReplace(t *testing.T) {
	src := []byte(`
module(name = "test-dotfiles", version = "1.2.3")
dep(url = "github://owner/repo@v1.0.0")
replace(module = "github://owner/repo", path = "/local/repo")
`)
	mf, err := modfile.ParseBytes("test.mod", src)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if mf.Module == nil {
		t.Fatal("Module is nil")
	}
	if mf.Module.Name != "test-dotfiles" {
		t.Errorf("Name = %q, want %q", mf.Module.Name, "test-dotfiles")
	}
	if mf.Module.Version != "1.2.3" {
		t.Errorf("Version = %q, want %q", mf.Module.Version, "1.2.3")
	}
	if len(mf.Deps) != 1 {
		t.Fatalf("len(Deps) = %d, want 1", len(mf.Deps))
	}
	if mf.Deps[0].URL != "github://owner/repo@v1.0.0" {
		t.Errorf("Dep URL = %q, want %q", mf.Deps[0].URL, "github://owner/repo@v1.0.0")
	}
	if len(mf.Replace) != 1 {
		t.Fatalf("len(Replace) = %d, want 1", len(mf.Replace))
	}
	if mf.Replace[0].Module != "github://owner/repo" {
		t.Errorf("Replace.Module = %q", mf.Replace[0].Module)
	}
	if mf.Replace[0].Path != "/local/repo" {
		t.Errorf("Replace.Path = %q", mf.Replace[0].Path)
	}
}

func TestParseBytes_Empty(t *testing.T) {
	mf, err := modfile.ParseBytes("empty.mod", []byte(""))
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if mf.Module != nil {
		t.Error("expected Module to be nil")
	}
	if len(mf.Deps) != 0 {
		t.Errorf("expected no deps, got %d", len(mf.Deps))
	}
}

func TestWrite_RoundTrip(t *testing.T) {
	tmp := t.TempDir()
	path := filepath.Join(tmp, "meowctl.mod")

	mf := &modfile.ModFile{
		Module: &modfile.ModuleDecl{Name: "dotfiles", Version: "0.2.0"},
		Deps:   []modfile.DepDecl{{URL: "github://foo/bar@v2.0.0"}},
	}
	if err := modfile.Write(path, mf); err != nil {
		t.Fatalf("Write: %v", err)
	}

	parsed, err := modfile.Parse(path)
	if err != nil {
		t.Fatalf("Parse: %v", err)
	}
	if parsed.Module.Name != "dotfiles" {
		t.Errorf("Name = %q", parsed.Module.Name)
	}
	if len(parsed.Deps) != 1 || parsed.Deps[0].URL != "github://foo/bar@v2.0.0" {
		t.Errorf("Deps = %v", parsed.Deps)
	}
}

func TestParse_FileNotFound(t *testing.T) {
	_, err := modfile.Parse("/nonexistent/meowctl.mod")
	if err == nil {
		t.Fatal("expected error for missing file")
	}
}

func TestParseBytes_DuplicateModule(t *testing.T) {
	src := []byte(`
module(name = "a", version = "1.0.0")
module(name = "b", version = "2.0.0")
`)
	_, err := modfile.ParseBytes("dup.mod", src)
	if err == nil {
		t.Fatal("expected error for duplicate module declaration")
	}
}

func TestParseBytes_UnknownBuiltin(t *testing.T) {
	src := []byte(`component("shell")`)
	_, err := modfile.ParseBytes("unknown.mod", src)
	if err == nil {
		t.Fatal("expected error for unknown builtin in modfile")
	}
}

func TestWrite_FilePermissions(t *testing.T) {
	tmp := t.TempDir()
	path := filepath.Join(tmp, "meowctl.mod")
	mf := &modfile.ModFile{}
	if err := modfile.Write(path, mf); err != nil {
		t.Fatalf("Write: %v", err)
	}
	info, err := os.Stat(path)
	if err != nil {
		t.Fatalf("Stat: %v", err)
	}
	if info.Mode().Perm() != 0o600 {
		t.Errorf("permissions = %o, want 600", info.Mode().Perm())
	}
}
