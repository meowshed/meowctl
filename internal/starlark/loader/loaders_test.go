package loader_test

import (
	"os"
	"path/filepath"
	"sync"
	"testing"

	gostarlark "go.starlark.net/starlark"
	"go.starlark.net/syntax"

	"github.com/meowshed/meowctl/internal/starlark/loader"
)

// writeStarFile writes content to a .star file in dir.
func writeStarFile(t *testing.T, dir, name, content string) {
	t.Helper()
	path := filepath.Join(dir, name)
	if err := os.WriteFile(path, []byte(content), 0o600); err != nil {
		t.Fatalf("writeStarFile: %v", err)
	}
}

// TestFileSystemLoader_Load_PathTraversal verifies that paths escaping the root are rejected.
func TestFileSystemLoader_Load_PathTraversal(t *testing.T) {
	l := loader.NewFileSystemLoader(t.TempDir(), &syntax.FileOptions{})
	thread := &gostarlark.Thread{Name: "test"}

	_, err := l.Load(thread, "self//../../../etc/passwd", gostarlark.StringDict{})
	if err == nil {
		t.Fatal("expected error for path traversal attempt")
	}
}

// TestFileSystemLoader_Load_HappyPath verifies that a self// URL resolves and evaluates.
func TestFileSystemLoader_Load_HappyPath(t *testing.T) {
	dir := t.TempDir()
	writeStarFile(t, dir, "lib.star", `x = 42`)

	l := loader.NewFileSystemLoader(dir, &syntax.FileOptions{})
	thread := &gostarlark.Thread{Name: "test"}

	globals, err := l.Load(thread, "self//lib.star", gostarlark.StringDict{})
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	v, ok := globals["x"]
	if !ok {
		t.Fatal("expected 'x' in globals")
	}
	n, ok2 := v.(gostarlark.Int).Int64()
	if !ok2 || n != 42 {
		t.Errorf("expected x=42, got %v", v)
	}
}

// TestFileSystemLoader_Load_UnsupportedScheme verifies that non-self// URLs are rejected.
func TestFileSystemLoader_Load_UnsupportedScheme(t *testing.T) {
	l := loader.NewFileSystemLoader(t.TempDir(), &syntax.FileOptions{})
	thread := &gostarlark.Thread{Name: "test"}

	_, err := l.Load(thread, "github://foo/bar", gostarlark.StringDict{})
	if err == nil {
		t.Fatal("expected error for unsupported scheme")
	}
}

// TestFileSystemLoader_Load_MissingFile verifies that a missing file returns an error.
func TestFileSystemLoader_Load_MissingFile(t *testing.T) {
	l := loader.NewFileSystemLoader(t.TempDir(), &syntax.FileOptions{})
	thread := &gostarlark.Thread{Name: "test"}

	_, err := l.Load(thread, "self//nonexistent.star", gostarlark.StringDict{})
	if err == nil {
		t.Fatal("expected error for missing file")
	}
}

// TestCompositeLoader_Dispatch_SelfScheme verifies that self// is dispatched to FileSystemLoader.
func TestCompositeLoader_Dispatch_SelfScheme(t *testing.T) {
	dir := t.TempDir()
	writeStarFile(t, dir, "mod.star", `answer = 1`)

	cl := loader.NewCompositeLoader(dir, &syntax.FileOptions{})
	thread := &gostarlark.Thread{Name: "test"}

	globals, err := cl.Load(thread, "self//mod.star", gostarlark.StringDict{})
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if _, ok := globals["answer"]; !ok {
		t.Error("expected 'answer' in globals")
	}
}

// TestCompositeLoader_Dispatch_GitHubScheme verifies that github:// returns not-implemented error.
func TestCompositeLoader_Dispatch_GitHubScheme(t *testing.T) {
	cl := loader.NewCompositeLoader(t.TempDir(), &syntax.FileOptions{})
	thread := &gostarlark.Thread{Name: "test"}

	_, err := cl.Load(thread, "github://org/repo@1.0.0//lib.star", gostarlark.StringDict{})
	if err == nil {
		t.Fatal("expected error for github:// stub")
	}
}

// TestCompositeLoader_Dispatch_RegistryScheme verifies that @name// returns not-implemented error.
func TestCompositeLoader_Dispatch_RegistryScheme(t *testing.T) {
	cl := loader.NewCompositeLoader(t.TempDir(), &syntax.FileOptions{})
	thread := &gostarlark.Thread{Name: "test"}

	_, err := cl.Load(thread, "@mymod//lib.star", gostarlark.StringDict{})
	if err == nil {
		t.Fatal("expected error for @name// stub")
	}
}

// TestCompositeLoader_Dispatch_UnknownScheme verifies that an unknown scheme returns an error.
func TestCompositeLoader_Dispatch_UnknownScheme(t *testing.T) {
	cl := loader.NewCompositeLoader(t.TempDir(), &syntax.FileOptions{})
	thread := &gostarlark.Thread{Name: "test"}

	_, err := cl.Load(thread, "http://example.com/lib.star", gostarlark.StringDict{})
	if err == nil {
		t.Fatal("expected error for unknown scheme")
	}
}

// TestCompositeLoader_Cache verifies that a module is evaluated exactly once.
func TestCompositeLoader_Cache(t *testing.T) {
	dir := t.TempDir()
	writeStarFile(t, dir, "once.star", `val = "cached"`)

	cl := loader.NewCompositeLoader(dir, &syntax.FileOptions{})
	thread := &gostarlark.Thread{Name: "test"}

	g1, err := cl.Load(thread, "self//once.star", gostarlark.StringDict{})
	if err != nil {
		t.Fatalf("first load: %v", err)
	}
	g2, err := cl.Load(thread, "self//once.star", gostarlark.StringDict{})
	if err != nil {
		t.Fatalf("second load: %v", err)
	}
	// Both loads must return the same dict instance (cached).
	if len(g1) != len(g2) {
		t.Errorf("cached dict length mismatch: %d vs %d", len(g1), len(g2))
	}
	for k, v1 := range g1 {
		v2, ok := g2[k]
		if !ok || v1 != v2 {
			t.Errorf("key %q: cached value mismatch", k)
		}
	}
}

// TestCompositeLoader_CacheConcurrent verifies there are no data races under concurrent load.
func TestCompositeLoader_CacheConcurrent(t *testing.T) {
	dir := t.TempDir()
	writeStarFile(t, dir, "concurrent.star", `n = 99`)

	cl := loader.NewCompositeLoader(dir, &syntax.FileOptions{})

	const goroutines = 10
	var wg sync.WaitGroup
	wg.Add(goroutines)
	for range goroutines {
		go func() {
			defer wg.Done()
			thread := &gostarlark.Thread{Name: "concurrent"}
			if _, err := cl.Load(thread, "self//concurrent.star", gostarlark.StringDict{}); err != nil {
				t.Errorf("concurrent load: %v", err)
			}
		}()
	}
	wg.Wait()
}
