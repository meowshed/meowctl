package modfile_test

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/meowshed/meowctl/internal/modfile"
)

func TestParseBytes_RegistryDep(t *testing.T) {
	src := []byte(`
module(name = "test-dotfiles", version = "1.2.3")
dep(name = "stdlib", version = "v0.1.1")
replace(name = "stdlib", path = "/local/stdlib")
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
	if mf.Deps[0].Name != "stdlib" {
		t.Errorf("Dep Name = %q, want %q", mf.Deps[0].Name, "stdlib")
	}
	if mf.Deps[0].Version != "v0.1.1" {
		t.Errorf("Dep Version = %q, want %q", mf.Deps[0].Version, "v0.1.1")
	}
	if mf.Deps[0].Source != "" {
		t.Errorf("Dep Source = %q, want empty", mf.Deps[0].Source)
	}
	if len(mf.Replace) != 1 {
		t.Fatalf("len(Replace) = %d, want 1", len(mf.Replace))
	}
	if mf.Replace[0].Name != "stdlib" {
		t.Errorf("Replace.Name = %q, want %q", mf.Replace[0].Name, "stdlib")
	}
	if mf.Replace[0].Path != "/local/stdlib" {
		t.Errorf("Replace.Path = %q, want %q", mf.Replace[0].Path, "/local/stdlib")
	}
	if mf.Replace[0].Source != "" {
		t.Errorf("Replace.Source = %q, want empty", mf.Replace[0].Source)
	}
}

func TestParseBytes_GitHubDep(t *testing.T) {
	src := []byte(`
module(name = "test-dotfiles", version = "1.0.0")
dep(name = "myplugin", source = "github:owner/repo@v1.2.3")
`)
	mf, err := modfile.ParseBytes("test.mod", src)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if len(mf.Deps) != 1 {
		t.Fatalf("len(Deps) = %d, want 1", len(mf.Deps))
	}
	d := mf.Deps[0]
	if d.Name != "myplugin" {
		t.Errorf("Name = %q, want %q", d.Name, "myplugin")
	}
	if d.Source != "github:owner/repo@v1.2.3" {
		t.Errorf("Source = %q, want %q", d.Source, "github:owner/repo@v1.2.3")
	}
	if d.Version != "" {
		t.Errorf("Version = %q, want empty for github dep", d.Version)
	}
}

func TestParseBytes_ReplaceWithSource(t *testing.T) {
	src := []byte(`
dep(name = "stdlib", version = "v0.1.0")
replace(name = "stdlib", source = "github:myfork/meowctl-stdlib@v1.2.4")
`)
	mf, err := modfile.ParseBytes("test.mod", src)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if len(mf.Replace) != 1 {
		t.Fatalf("len(Replace) = %d, want 1", len(mf.Replace))
	}
	r := mf.Replace[0]
	if r.Name != "stdlib" {
		t.Errorf("Name = %q, want %q", r.Name, "stdlib")
	}
	if r.Source != "github:myfork/meowctl-stdlib@v1.2.4" {
		t.Errorf("Source = %q, want %q", r.Source, "github:myfork/meowctl-stdlib@v1.2.4")
	}
	if r.Path != "" {
		t.Errorf("Path = %q, want empty", r.Path)
	}
}

func TestParseBytes_DepMissingVersionAndSource(t *testing.T) {
	src := []byte(`dep(name = "stdlib")`)
	_, err := modfile.ParseBytes("test.mod", src)
	if err == nil {
		t.Fatal("expected error when both version and source are absent")
	}
}

func TestParseBytes_DepBothVersionAndSource(t *testing.T) {
	src := []byte(`dep(name = "stdlib", version = "v1.0.0", source = "github:owner/repo@v1.0.0")`)
	_, err := modfile.ParseBytes("test.mod", src)
	if err == nil {
		t.Fatal("expected error when both version and source are set")
	}
}

func TestParseBytes_ReplaceMissingPathAndSource(t *testing.T) {
	src := []byte(`replace(name = "stdlib")`)
	_, err := modfile.ParseBytes("test.mod", src)
	if err == nil {
		t.Fatal("expected error when both path and source are absent")
	}
}

func TestParseBytes_ReplaceBothPathAndSource(t *testing.T) {
	src := []byte(`replace(name = "stdlib", path = "/local", source = "github:owner/repo@v1.0.0")`)
	_, err := modfile.ParseBytes("test.mod", src)
	if err == nil {
		t.Fatal("expected error when both path and source are set")
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

func TestWrite_RegistryDepRoundTrip(t *testing.T) {
	tmp := t.TempDir()
	path := filepath.Join(tmp, "deps.mod")

	mf := &modfile.ModFile{
		Module: &modfile.ModuleDecl{Name: "dotfiles", Version: "0.2.0"},
		Deps:   []modfile.DepDecl{{Name: "stdlib", Version: "v2.0.0"}},
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
	if len(parsed.Deps) != 1 || parsed.Deps[0].Name != "stdlib" || parsed.Deps[0].Version != "v2.0.0" {
		t.Errorf("Deps = %v", parsed.Deps)
	}
}

func TestWrite_GitHubDepRoundTrip(t *testing.T) {
	tmp := t.TempDir()
	path := filepath.Join(tmp, "deps.mod")

	mf := &modfile.ModFile{
		Deps: []modfile.DepDecl{{Name: "myplugin", Source: "github:owner/repo@v1.2.3"}},
	}
	if err := modfile.Write(path, mf); err != nil {
		t.Fatalf("Write: %v", err)
	}

	parsed, err := modfile.Parse(path)
	if err != nil {
		t.Fatalf("Parse: %v", err)
	}
	if len(parsed.Deps) != 1 {
		t.Fatalf("len(Deps) = %d, want 1", len(parsed.Deps))
	}
	if parsed.Deps[0].Source != "github:owner/repo@v1.2.3" {
		t.Errorf("Source = %q", parsed.Deps[0].Source)
	}
	if parsed.Deps[0].Version != "" {
		t.Errorf("Version = %q, want empty", parsed.Deps[0].Version)
	}
}

func TestWrite_ReplaceSourceRoundTrip(t *testing.T) {
	tmp := t.TempDir()
	path := filepath.Join(tmp, "deps.mod")

	mf := &modfile.ModFile{
		Deps:    []modfile.DepDecl{{Name: "stdlib", Version: "v1.0.0"}},
		Replace: []modfile.ReplaceDecl{{Name: "stdlib", Source: "github:myfork/stdlib@v1.2.4"}},
	}
	if err := modfile.Write(path, mf); err != nil {
		t.Fatalf("Write: %v", err)
	}

	parsed, err := modfile.Parse(path)
	if err != nil {
		t.Fatalf("Parse: %v", err)
	}
	if len(parsed.Replace) != 1 {
		t.Fatalf("len(Replace) = %d, want 1", len(parsed.Replace))
	}
	r := parsed.Replace[0]
	if r.Name != "stdlib" || r.Source != "github:myfork/stdlib@v1.2.4" || r.Path != "" {
		t.Errorf("Replace = %+v", r)
	}
}

func TestParse_FileNotFound(t *testing.T) {
	_, err := modfile.Parse("/nonexistent/deps.mod")
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
	path := filepath.Join(tmp, "deps.mod")
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
