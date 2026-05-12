//go:build integration

package loader_test

// Live integration smoke test — requires real network access to GitHub.
// Run with: go test -tags integration ./internal/starlark/loader/...
//
// This test is intentionally excluded from normal CI. It validates that the
// live registry index and hello release asset are reachable and that
// loading @hello//hello.star returns greeting = "hello".

import (
	"net/http"
	"os"
	"path/filepath"
	"testing"

	gostarlark "go.starlark.net/starlark"
	"go.starlark.net/syntax"

	"github.com/meowshed/meowctl/internal/starlark/loader"
)

// TestLiveRegistry_HelloGreeting loads @hello//hello.star from the
// live meowctl-registry index and asserts greeting = "hello".
func TestLiveRegistry_HelloGreeting(t *testing.T) {
	cacheDir := t.TempDir()
	lockPath := filepath.Join(t.TempDir(), "meowctl.lock")

	// NewRegistryLoaderForTest with empty overrides → uses production defaults.
	// os.TempDir() is passed as the dotfiles root (used by self// scheme only —
	// not exercised here, so any readable directory works).
	cl := loader.NewRegistryLoaderForTest(
		os.TempDir(),
		&syntax.FileOptions{},
		loader.CompositeLoaderOptions{
			CacheDir: cacheDir,
			LockPath: lockPath,
			Client:   &http.Client{},
		},
		"", // no GitHub API override
		"", // no GitHub raw override
		"", // no registry override → uses DefaultRegistryURL
	)

	thread := &gostarlark.Thread{Name: "live-integration"}
	globals, err := cl.Load(thread, "@hello//hello.star", gostarlark.StringDict{})
	if err != nil {
		t.Fatalf("Load @hello//hello.star: %v", err)
	}

	v, ok := globals["greeting"]
	if !ok {
		t.Fatal("expected 'greeting' in globals")
	}
	const want = `"hello"`
	if v.String() != want {
		t.Errorf("greeting = %v, want %v", v.String(), want)
	}
}
