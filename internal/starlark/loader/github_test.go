package loader_test

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"strings"
	"testing"

	gostarlark "go.starlark.net/starlark"
	"go.starlark.net/syntax"

	"github.com/meowshed/meowctl/internal/starlark/loader"
)

// testServer creates an httptest.Server that serves the given URL path → body mapping.
// Requests for unknown paths return 404.
func testServer(t *testing.T, routes map[string][]byte) *httptest.Server {
	t.Helper()
	return httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if body, ok := routes[r.URL.Path]; ok {
			w.WriteHeader(http.StatusOK)
			_, _ = w.Write(body)
			return
		}
		http.NotFound(w, r)
	}))
}

// commitResponse builds the JSON body returned by the GitHub commits API.
func commitResponse(sha string) []byte {
	b, _ := json.Marshal(map[string]string{"sha": sha})
	return b
}

// newTestGitHubLoader builds a CompositeLoader wired to fake API and raw servers.
func newTestGitHubLoader(t *testing.T, apiServer, rawServer *httptest.Server, cacheDir, lockPath string) *loader.CompositeLoader {
	t.Helper()
	cl := loader.NewCompositeLoaderForTest(
		t.TempDir(),
		&syntax.FileOptions{},
		loader.CompositeLoaderOptions{
			CacheDir: cacheDir,
			LockPath: lockPath,
			Client:   &http.Client{},
		},
		apiServer.URL,
		rawServer.URL,
	)
	return cl
}

// TestGitHubLoader_HappyPath verifies that a github:// URL resolves, fetches, caches,
// and evaluates a Starlark file on the first call.
func TestGitHubLoader_HappyPath(t *testing.T) {
	const sha = "abc123def456abc123def456abc123def456abc123"
	content := []byte(`greeting = "hello"`)

	apiServer := testServer(t, map[string][]byte{
		"/repos/owner/repo/commits/main": commitResponse(sha),
	})
	defer apiServer.Close()

	rawServer := testServer(t, map[string][]byte{
		"/owner/repo/" + sha + "/lib.star": content,
	})
	defer rawServer.Close()

	cacheDir := t.TempDir()
	lockPath := filepath.Join(t.TempDir(), "meowctl.lock")

	cl := newTestGitHubLoader(t, apiServer, rawServer, cacheDir, lockPath)
	thread := &gostarlark.Thread{Name: "test"}

	globals, err := cl.Load(thread, "github://owner/repo@main//lib.star", gostarlark.StringDict{})
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if _, ok := globals["greeting"]; !ok {
		t.Error("expected 'greeting' in globals")
	}
}

// TestGitHubLoader_CacheHit verifies that a second load does not make any HTTP calls.
func TestGitHubLoader_CacheHit(t *testing.T) {
	const sha = "deadbeefdeadbeefdeadbeefdeadbeefdeadbeef"
	content := []byte(`cached = True`)

	callCount := 0
	apiServer := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		callCount++
		if r.URL.Path == "/repos/owner/repo/commits/v1.0.0" {
			w.WriteHeader(http.StatusOK)
			_, _ = w.Write(commitResponse(sha))
			return
		}
		http.NotFound(w, r)
	}))
	defer apiServer.Close()

	rawCallCount := 0
	rawServer := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		rawCallCount++
		if r.URL.Path == "/owner/repo/"+sha+"/mod.star" {
			w.WriteHeader(http.StatusOK)
			_, _ = w.Write(content)
			return
		}
		http.NotFound(w, r)
	}))
	defer rawServer.Close()

	cacheDir := t.TempDir()
	lockPath := filepath.Join(t.TempDir(), "meowctl.lock")

	// First load — populates cache and lock.
	cl := newTestGitHubLoader(t, apiServer, rawServer, cacheDir, lockPath)
	thread := &gostarlark.Thread{Name: "test"}
	if _, err := cl.Load(thread, "github://owner/repo@v1.0.0//mod.star", gostarlark.StringDict{}); err != nil {
		t.Fatalf("first load: %v", err)
	}

	prevAPI := callCount
	prevRaw := rawCallCount

	// Second load on a fresh CompositeLoader using the same cache + lock.
	cl2 := newTestGitHubLoader(t, apiServer, rawServer, cacheDir, lockPath)
	if _, err := cl2.Load(thread, "github://owner/repo@v1.0.0//mod.star", gostarlark.StringDict{}); err != nil {
		t.Fatalf("second load: %v", err)
	}

	// The in-memory CompositeLoader cache will serve subsequent calls for the same
	// CompositeLoader instance; a fresh loader should use the on-disk lock+cache.
	if callCount > prevAPI {
		t.Errorf("API called again on cache hit: total=%d prev=%d", callCount, prevAPI)
	}
	if rawCallCount > prevRaw {
		t.Errorf("raw fetch called again on cache hit: total=%d prev=%d", rawCallCount, prevRaw)
	}
}

// TestGitHubLoader_SRIMismatch verifies that a corrupted cache entry triggers a re-fetch,
// and that a re-fetch failure (404) surfaces as an error rather than silently loading stale data.
func TestGitHubLoader_SRIMismatch(t *testing.T) {
	const sha = "cafebabecafebabecafebabecafebabecafebabe0"
	goodContent := []byte(`x = 1`)

	apiServer := testServer(t, map[string][]byte{
		"/repos/owner/repo/commits/main": commitResponse(sha),
	})
	defer apiServer.Close()

	rawServer := testServer(t, map[string][]byte{
		"/owner/repo/" + sha + "/x.star": goodContent,
	})
	defer rawServer.Close()

	cacheDir := t.TempDir()
	lockPath := filepath.Join(t.TempDir(), "meowctl.lock")

	// First load: populate cache and lock correctly.
	cl := newTestGitHubLoader(t, apiServer, rawServer, cacheDir, lockPath)
	thread := &gostarlark.Thread{Name: "test"}
	if _, err := cl.Load(thread, "github://owner/repo@main//x.star", gostarlark.StringDict{}); err != nil {
		t.Fatalf("first load: %v", err)
	}

	// Corrupt the cache file so the next load's SRI check fails.
	cachePath := filepath.Join(cacheDir, "github", "owner", "repo", sha, "x.star")
	if err := os.WriteFile(cachePath, []byte("corrupted content"), 0o600); err != nil {
		t.Fatalf("corrupt cache: %v", err)
	}

	// Point the re-fetch at a server that returns 404 so the re-fetch path errors out.
	// This confirms the loader does not silently serve the corrupted cached bytes.
	emptyServer := testServer(t, map[string][]byte{})
	defer emptyServer.Close()

	cl2 := newTestGitHubLoader(t, apiServer, emptyServer, cacheDir, lockPath)
	_, err := cl2.Load(thread, "github://owner/repo@main//x.star", gostarlark.StringDict{})
	if err == nil {
		t.Fatal("expected error when cache is corrupt and re-fetch returns 404, got nil")
	}
}

// TestGitHubLoader_LockWriteFailureNonFatal verifies that a lock-write failure does not
// prevent the module from loading successfully.
func TestGitHubLoader_LockWriteFailureNonFatal(t *testing.T) {
	const sha = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
	content := []byte(`ok = True`)

	apiServer := testServer(t, map[string][]byte{
		"/repos/owner/repo/commits/main": commitResponse(sha),
	})
	defer apiServer.Close()

	rawServer := testServer(t, map[string][]byte{
		"/owner/repo/" + sha + "/ok.star": content,
	})
	defer rawServer.Close()

	// Make the lock's parent directory read-only so os.CreateTemp (used by lock.Write)
	// fails. MkdirAll would succeed on a missing dir, so we need an existing but
	// unwritable directory instead.
	lockDir := t.TempDir()
	if err := os.Chmod(lockDir, 0o500); err != nil { // #nosec G302 -- 0o500 is intentional: read-only dir to force lock write failure
		t.Fatalf("chmod lock dir: %v", err)
	}
	// Restore permissions so t.TempDir cleanup can remove the directory.
	t.Cleanup(func() { _ = os.Chmod(lockDir, 0o700) }) // #nosec G302 -- restoring traversable permissions for cleanup
	lockPath := filepath.Join(lockDir, "meowctl.lock")

	cl := newTestGitHubLoader(t, apiServer, rawServer, t.TempDir(), lockPath)
	thread := &gostarlark.Thread{Name: "test"}

	globals, err := cl.Load(thread, "github://owner/repo@main//ok.star", gostarlark.StringDict{})
	if err != nil {
		t.Fatalf("expected successful load despite lock-write failure, got: %v", err)
	}
	if _, ok := globals["ok"]; !ok {
		t.Error("expected 'ok' in globals")
	}
}

// TestGitHubLoader_404 verifies that a 404 from the raw server produces a clear error.
func TestGitHubLoader_404(t *testing.T) {
	const sha = "0000000000000000000000000000000000000000"

	apiServer := testServer(t, map[string][]byte{
		"/repos/owner/repo/commits/main": commitResponse(sha),
	})
	defer apiServer.Close()

	// Raw server returns 404 for all paths.
	rawServer := testServer(t, map[string][]byte{})
	defer rawServer.Close()

	cacheDir := t.TempDir()
	lockPath := filepath.Join(t.TempDir(), "meowctl.lock")

	cl := newTestGitHubLoader(t, apiServer, rawServer, cacheDir, lockPath)
	thread := &gostarlark.Thread{Name: "test"}

	_, err := cl.Load(thread, "github://owner/repo@main//missing.star", gostarlark.StringDict{})
	if err == nil {
		t.Fatal("expected error for 404")
	}
}

// TestGitHubLoader_RefPinned verifies that the lock file records the commit SHA after first use.
func TestGitHubLoader_RefPinned(t *testing.T) {
	const sha = "1111111111111111111111111111111111111111"
	content := []byte(`pinned = True`)

	apiServer := testServer(t, map[string][]byte{
		"/repos/org/repo/commits/feature": commitResponse(sha),
	})
	defer apiServer.Close()

	rawServer := testServer(t, map[string][]byte{
		"/org/repo/" + sha + "/pin.star": content,
	})
	defer rawServer.Close()

	lockDir := t.TempDir()
	lockPath := filepath.Join(lockDir, "meowctl.lock")

	cl := newTestGitHubLoader(t, apiServer, rawServer, t.TempDir(), lockPath)
	thread := &gostarlark.Thread{Name: "test"}

	if _, err := cl.Load(thread, "github://org/repo@feature//pin.star", gostarlark.StringDict{}); err != nil {
		t.Fatalf("load: %v", err)
	}

	// Verify lock file was written with the resolved SHA.
	lockData, err := os.ReadFile(lockPath) // #nosec G304 -- lockPath is a t.TempDir() path, not user-tainted input
	if err != nil {
		t.Fatalf("read lock: %v", err)
	}
	if !strings.Contains(string(lockData), sha) {
		t.Errorf("lock file does not contain commit SHA %q:\n%s", sha, lockData)
	}
}

// TestGitHubLoader_UserScheme verifies that user:// resolves relative to the config dir.
func TestGitHubLoader_UserScheme(t *testing.T) {
	configDir := t.TempDir()
	writeStarFile(t, configDir, "helper.star", `result = 7`)

	cl := loader.NewCompositeLoader(
		t.TempDir(),
		&syntax.FileOptions{},
		loader.CompositeLoaderOptions{UserRoot: configDir},
	)
	thread := &gostarlark.Thread{Name: "test"}

	globals, err := cl.Load(thread, "user://helper.star", gostarlark.StringDict{})
	if err != nil {
		t.Fatalf("user:// load: %v", err)
	}
	if _, ok := globals["result"]; !ok {
		t.Error("expected 'result' in globals")
	}
}
