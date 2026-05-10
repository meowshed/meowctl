package lock_test

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/meowshed/meowctl/internal/lock"
)

func TestRead_MissingFile(t *testing.T) {
	lf, err := lock.Read(filepath.Join(t.TempDir(), "nonexistent.lock"))
	if err != nil {
		t.Fatalf("expected nil error for missing file, got: %v", err)
	}
	if lf == nil {
		t.Fatal("expected non-nil LockFile for missing file")
	}
}

func TestWriteRead_RoundTrip(t *testing.T) {
	dir := t.TempDir()
	path := filepath.Join(dir, "meowctl.lock")

	original := &lock.LockFile{
		Meta: lock.LockMeta{
			GeneratedBy: "meowctl v0.1.0",
			UpdatedAt:   "2026-05-10T00:00:00Z",
		},
		Modules: map[string]lock.ModuleEntry{
			"github.com/meowshed/meowctl-stdlib": {
				Version:   "v1.2.3",
				Source:    "https://example.com/stdlib-v1.2.3.tar.gz",
				Integrity: "sha384-abc123",
				Files: map[string]string{
					"init.star": "sha384-def456",
				},
			},
		},
		GitHub: map[string]lock.GitHubEntry{
			"github://meowshed/extras@main//tool.star": {
				Commit:    "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef",
				Integrity: "sha384-ghijkl",
			},
		},
		Packages: map[string]lock.ManagerPackages{
			"homebrew": {
				"ripgrep": lock.PackageEntry{
					Requested: "latest",
					Installed: "14.1.0",
				},
				"jq": lock.PackageEntry{
					Requested: "1.7",
					Installed: "1.7.1",
					Note:      "pinned: compatibility",
				},
			},
		},
	}

	if err := lock.Write(path, original); err != nil {
		t.Fatalf("Write failed: %v", err)
	}

	got, err := lock.Read(path)
	if err != nil {
		t.Fatalf("Read failed: %v", err)
	}

	// Verify modules.
	mod, ok := got.Modules["github.com/meowshed/meowctl-stdlib"]
	if !ok {
		t.Fatal("expected stdlib module in lock file")
	}
	if mod.Version != "v1.2.3" {
		t.Errorf("module version: got %q, want %q", mod.Version, "v1.2.3")
	}
	if mod.Files["init.star"] != "sha384-def456" {
		t.Errorf("module file hash: got %q, want %q", mod.Files["init.star"], "sha384-def456")
	}

	// Verify GitHub entries.
	gh, ok := got.GitHub["github://meowshed/extras@main//tool.star"]
	if !ok {
		t.Fatal("expected github entry in lock file")
	}
	if gh.Commit != "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef" {
		t.Errorf("github commit: got %q", gh.Commit)
	}

	// Verify packages.
	rg, ok := got.Packages["homebrew"]["ripgrep"]
	if !ok {
		t.Fatal("expected ripgrep package entry")
	}
	if rg.Installed != "14.1.0" {
		t.Errorf("ripgrep installed: got %q, want %q", rg.Installed, "14.1.0")
	}
	jq := got.Packages["homebrew"]["jq"]
	if jq.Note != "pinned: compatibility" {
		t.Errorf("jq note: got %q", jq.Note)
	}

	// Verify meta.
	if got.Meta.GeneratedBy != "meowctl v0.1.0" {
		t.Errorf("meta generated-by: got %q", got.Meta.GeneratedBy)
	}
}

func TestWrite_Atomic(t *testing.T) {
	// Verify that Write creates parent directories and that the result is readable.
	dir := t.TempDir()
	path := filepath.Join(dir, "sub", "meowctl.lock")

	lf := &lock.LockFile{Meta: lock.LockMeta{GeneratedBy: "test"}}
	if err := lock.Write(path, lf); err != nil {
		t.Fatalf("Write with nested dir failed: %v", err)
	}

	if _, err := os.Stat(path); err != nil {
		t.Fatalf("lock file not found after Write: %v", err)
	}
}

func TestWrite_NoTempFileLeft(t *testing.T) {
	// After a successful Write, no temp files should remain in the directory.
	dir := t.TempDir()
	path := filepath.Join(dir, "meowctl.lock")

	lf := &lock.LockFile{Meta: lock.LockMeta{GeneratedBy: "test"}}
	if err := lock.Write(path, lf); err != nil {
		t.Fatalf("Write failed: %v", err)
	}

	entries, err := os.ReadDir(dir)
	if err != nil {
		t.Fatalf("ReadDir failed: %v", err)
	}
	for _, e := range entries {
		if e.Name() != "meowctl.lock" {
			t.Errorf("unexpected file left in dir: %s", e.Name())
		}
	}
}
