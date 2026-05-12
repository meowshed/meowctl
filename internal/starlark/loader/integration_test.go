package loader_test

import (
	"encoding/json"
	"net/http"
	"path/filepath"
	"testing"

	gostarlark "go.starlark.net/starlark"
	"go.starlark.net/syntax"

	"github.com/meowshed/meowctl/internal/starlark/loader"
)

// TestIntegration_AllSchemes evaluates a Starlark file that loads from all four
// schemes (self//, user://, github://, @name//) against fake httptest servers.
// It asserts that globals from each loaded module are accessible.
func TestIntegration_AllSchemes(t *testing.T) {
	const sha = "ffffffffffffffffffffffffffffffffffffffff"

	// self// module: resolved by FileSystemLoader from dotfiles root.
	selfDir := t.TempDir()
	writeStarFile(t, selfDir, "self_lib.star", `self_val = "from_self"`)

	// user:// module: resolved by FileSystemLoader from config dir.
	userDir := t.TempDir()
	writeStarFile(t, userDir, "user_lib.star", `user_val = "from_user"`)

	// github:// module: served by fake API + raw servers.
	githubContent := []byte(`github_val = "from_github"`)
	apiServer := testServer(t, map[string][]byte{
		"/repos/org/repo/commits/main": func() []byte {
			b, _ := json.Marshal(map[string]string{"sha": sha})
			return b
		}(),
	})
	defer apiServer.Close()

	rawServer := testServer(t, map[string][]byte{
		"/org/repo/" + sha + "/mod.star": githubContent,
	})
	defer rawServer.Close()

	// @name// module: served by fake registry index + tarball server.
	regContent := []byte(`registry_val = "from_registry"`)
	tarball := buildTarGz(t, map[string][]byte{"reg_lib.star": regContent})

	tarballServer := testServer(t, map[string][]byte{
		"/regmod-v1.0.0.tar.gz": tarball,
	})
	defer tarballServer.Close()

	indexBody := registryIndexTOML(t, map[string]map[string]interface{}{
		"regmod": {
			"versions": []string{"v1.0.0"},
			"source":   tarballServer.URL + "/regmod-{version}.tar.gz",
		},
	})
	indexServer := testServer(t, map[string][]byte{"/": indexBody})
	defer indexServer.Close()

	// Main Starlark script loads from all four schemes.
	mainScript := []byte(`
load("self//self_lib.star", "self_val")
load("user://user_lib.star", "user_val")
load("github://org/repo@main//mod.star", "github_val")
load("@regmod//reg_lib.star", "registry_val")

combined = self_val + "|" + user_val + "|" + github_val + "|" + registry_val
`)

	cacheDir := t.TempDir()
	lockPath := filepath.Join(t.TempDir(), "meowctl.lock")

	cl := loader.NewRegistryLoaderForTest(
		selfDir,
		&syntax.FileOptions{},
		loader.CompositeLoaderOptions{
			UserRoot: userDir,
			CacheDir: cacheDir,
			LockPath: lockPath,
			Client:   &http.Client{},
		},
		apiServer.URL,
		rawServer.URL,
		indexServer.URL,
	)

	thread := &gostarlark.Thread{
		Name: "integration",
		Load: func(t *gostarlark.Thread, module string) (gostarlark.StringDict, error) {
			return cl.Load(t, module, gostarlark.StringDict{})
		},
	}

	globals, err := gostarlark.ExecFileOptions(
		&syntax.FileOptions{},
		thread,
		"main.star",
		mainScript,
		gostarlark.StringDict{},
	)
	if err != nil {
		t.Fatalf("ExecFileOptions: %v", err)
	}

	combined, ok := globals["combined"]
	if !ok {
		t.Fatal("expected 'combined' in globals")
	}
	const want = `"from_self|from_user|from_github|from_registry"`
	if combined.String() != want {
		t.Errorf("combined = %v, want %v", combined.String(), want)
	}
}

// TestIntegration_RegistryScheme_NotImplemented_Replaced verifies that @name//
// no longer returns the old not-implemented stub error — it should succeed or
// fail with a meaningful error (not the stub sentinel).
func TestIntegration_RegistryScheme_NotImplemented_Replaced(t *testing.T) {
	indexBody := registryIndexTOML(t, map[string]map[string]interface{}{
		"mymod": {
			"versions": []string{"v1.0.0"},
			"source":   "http://127.0.0.1:0/mymod-{version}.tar.gz", // unreachable — just checking error type
		},
	})
	indexServer := testServer(t, map[string][]byte{"/": indexBody})
	defer indexServer.Close()

	cl := loader.NewRegistryLoaderForTest(
		t.TempDir(),
		&syntax.FileOptions{},
		loader.CompositeLoaderOptions{
			CacheDir: t.TempDir(),
			LockPath: filepath.Join(t.TempDir(), "meowctl.lock"),
			Client:   &http.Client{},
		},
		"", "",
		indexServer.URL,
	)
	thread := &gostarlark.Thread{Name: "test"}
	_, err := cl.Load(thread, "@mymod//lib.star", gostarlark.StringDict{})
	// We expect an error (tarball server is unreachable), but it must NOT be the
	// old "not implemented" stub error.
	if err == nil {
		t.Fatal("expected error for unreachable tarball, got nil")
	}
	const stub = "RegistryLoader: not implemented"
	if err.Error() == stub {
		t.Errorf("got old stub error %q — RegistryLoader is not fully implemented", stub)
	}
}

// TestIntegration_httptest_NoRealNetwork documents that all servers used in
// integration tests are httptest servers — no real network calls are made.
// This function does nothing at runtime; its presence records the intent.
func TestIntegration_httptest_NoRealNetwork(_ *testing.T) {}
