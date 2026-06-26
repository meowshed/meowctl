package loader_test

import (
	"archive/tar"
	"bytes"
	"compress/gzip"
	"crypto/sha512"
	"encoding/base64"
	"fmt"
	"io"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/BurntSushi/toml"
	gostarlark "go.starlark.net/starlark"
	"go.starlark.net/syntax"

	"github.com/meowshed/meowctl/internal/lock"
	"github.com/meowshed/meowctl/internal/starlark/loader"
)

// registrySRI returns the sha384 SRI string for data (mirrors computeSRI in registry.go).
func registrySRI(data []byte) string {
	sum := sha512.Sum384(data)
	return "sha384-" + base64.StdEncoding.EncodeToString(sum[:])
}

// buildTarGz builds an in-memory tar.gz archive from a map of path → content.
func buildTarGz(t *testing.T, files map[string][]byte) []byte {
	t.Helper()
	var buf bytes.Buffer
	gw := gzip.NewWriter(&buf)
	tw := tar.NewWriter(gw)
	for name, data := range files {
		hdr := &tar.Header{
			Name:     name,
			Typeflag: tar.TypeReg,
			Size:     int64(len(data)),
			Mode:     0o600,
		}
		if err := tw.WriteHeader(hdr); err != nil {
			t.Fatalf("buildTarGz WriteHeader %q: %v", name, err)
		}
		if _, err := tw.Write(data); err != nil {
			t.Fatalf("buildTarGz Write %q: %v", name, err)
		}
	}
	if err := tw.Close(); err != nil {
		t.Fatalf("buildTarGz tw.Close: %v", err)
	}
	if err := gw.Close(); err != nil {
		t.Fatalf("buildTarGz gw.Close: %v", err)
	}
	return buf.Bytes()
}

// registryIndexTOML serialises a simple index.toml from a map of module name →
// (versions, source template).
func registryIndexTOML(t *testing.T, modules map[string]map[string]interface{}) []byte {
	t.Helper()
	type entry struct {
		Versions []string `toml:"versions"`
		Source   string   `toml:"source"`
	}
	type index struct {
		Modules map[string]entry `toml:"modules"`
	}
	idx := index{Modules: map[string]entry{}}
	for name, m := range modules {
		idx.Modules[name] = entry{
			Versions: append([]string{}, m["versions"].([]string)...),
			Source:   m["source"].(string),
		}
	}
	var buf bytes.Buffer
	if err := toml.NewEncoder(&buf).Encode(idx); err != nil {
		t.Fatalf("registryIndexTOML: %v", err)
	}
	return buf.Bytes()
}

// newTestRegistryLoader wires a CompositeLoader to a fake registry index server
// and a fake tarball server.
func newTestRegistryLoader(
	t *testing.T,
	indexServer *httptest.Server,
	cacheDir, lockPath string,
) *loader.CompositeLoader {
	t.Helper()
	return loader.NewRegistryLoaderForTest(
		t.TempDir(),
		&syntax.FileOptions{},
		loader.CompositeLoaderOptions{
			CacheDir: cacheDir,
			LockPath: lockPath,
			Client:   &http.Client{},
		},
		"", // no GitHub override needed for registry tests
		"",
		indexServer.URL,
	)
}

// TestRegistryLoader_HappyPath verifies that @name// resolves, downloads,
// extracts, and evaluates a module file.
func TestRegistryLoader_HappyPath(t *testing.T) {
	modContent := []byte(`answer = 42`)
	tarball := buildTarGz(t, map[string][]byte{"lib.star": modContent})

	tarballServer := testServer(t, map[string][]byte{
		"/mymod-v1.0.0.tar.gz": tarball,
	})
	defer tarballServer.Close()

	indexBody := registryIndexTOML(t, map[string]map[string]interface{}{
		"mymod": {
			"versions": []string{"v1.0.0"},
			"source":   tarballServer.URL + "/mymod-{version}.tar.gz",
		},
	})
	indexServer := testServer(t, map[string][]byte{"/": indexBody})
	defer indexServer.Close()

	cacheDir := t.TempDir()
	lockPath := filepath.Join(t.TempDir(), "meowctl.lock")

	cl := newTestRegistryLoader(t, indexServer, cacheDir, lockPath)
	thread := &gostarlark.Thread{Name: "test"}

	globals, err := cl.Load(thread, "@mymod//lib.star", gostarlark.StringDict{})
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if _, ok := globals["answer"]; !ok {
		t.Error("expected 'answer' in globals")
	}
}

// TestRegistryLoader_PreservesExecBit verifies that an executable file in a
// module tarball keeps its exec bit (normalized to 0755) after extraction to the
// module cache, while a non-executable file lands 0644.
func TestRegistryLoader_PreservesExecBit(t *testing.T) {
	// Build a tarball with mixed modes (buildTarGz forces 0600, so build inline).
	var buf bytes.Buffer
	gw := gzip.NewWriter(&buf)
	tw := tar.NewWriter(gw)
	for _, e := range []struct {
		name string
		mode int64
	}{
		{"lib.star", 0o644},
		{"scripts/run.sh", 0o755},
	} {
		body := []byte("answer = 42\n")
		if err := tw.WriteHeader(&tar.Header{Name: e.name, Typeflag: tar.TypeReg, Size: int64(len(body)), Mode: e.mode}); err != nil {
			t.Fatal(err)
		}
		if _, err := tw.Write(body); err != nil {
			t.Fatal(err)
		}
	}
	if err := tw.Close(); err != nil {
		t.Fatal(err)
	}
	if err := gw.Close(); err != nil {
		t.Fatal(err)
	}
	tarball := buf.Bytes()

	tarballServer := testServer(t, map[string][]byte{"/mymod-v1.0.0.tar.gz": tarball})
	defer tarballServer.Close()

	indexBody := registryIndexTOML(t, map[string]map[string]interface{}{
		"mymod": {
			"versions": []string{"v1.0.0"},
			"source":   tarballServer.URL + "/mymod-{version}.tar.gz",
		},
	})
	indexServer := testServer(t, map[string][]byte{"/": indexBody})
	defer indexServer.Close()

	cacheDir := t.TempDir()
	lockPath := filepath.Join(t.TempDir(), "meowctl.lock")
	cl := newTestRegistryLoader(t, indexServer, cacheDir, lockPath)
	thread := &gostarlark.Thread{Name: "test"}

	if _, err := cl.Load(thread, "@mymod//lib.star", gostarlark.StringDict{}); err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	base := filepath.Join(cacheDir, "modules", "mymod", "v1.0.0")
	cases := map[string]os.FileMode{
		"scripts/run.sh": 0o755,
		"lib.star":       0o644,
	}
	for rel, want := range cases {
		info, statErr := os.Stat(filepath.Join(base, filepath.FromSlash(rel)))
		if statErr != nil {
			t.Fatalf("stat %s: %v", rel, statErr)
		}
		if got := info.Mode().Perm(); got != want {
			t.Errorf("%s perm: got %o, want %o", rel, got, want)
		}
	}
}

// TestRegistryLoader_UnknownModule verifies that a module not listed in the
// registry index produces a clear error.
func TestRegistryLoader_UnknownModule(t *testing.T) {
	indexBody := registryIndexTOML(t, map[string]map[string]interface{}{})
	indexServer := testServer(t, map[string][]byte{"/": indexBody})
	defer indexServer.Close()

	cacheDir := t.TempDir()
	lockPath := filepath.Join(t.TempDir(), "meowctl.lock")

	cl := newTestRegistryLoader(t, indexServer, cacheDir, lockPath)
	thread := &gostarlark.Thread{Name: "test"}

	_, err := cl.Load(thread, "@nosuchmod//lib.star", gostarlark.StringDict{})
	if err == nil {
		t.Fatal("expected error for unknown module, got nil")
	}
}

// TestRegistryLoader_SRIMismatch verifies that a locked entry with a wrong SRI
// causes an error when the tarball is re-downloaded (cache absent + corrupt lock).
func TestRegistryLoader_SRIMismatch(t *testing.T) {
	modContent := []byte(`ok = True`)
	tarball := buildTarGz(t, map[string][]byte{"lib.star": modContent})
	wrongInteg := "sha384-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"

	tarballServer := testServer(t, map[string][]byte{
		"/mymod-v1.0.0.tar.gz": tarball,
	})
	defer tarballServer.Close()

	indexBody := registryIndexTOML(t, map[string]map[string]interface{}{
		"mymod": {
			"versions": []string{"v1.0.0"},
			"source":   tarballServer.URL + "/mymod-{version}.tar.gz",
		},
	})
	indexServer := testServer(t, map[string][]byte{"/": indexBody})
	defer indexServer.Close()

	cacheDir := t.TempDir()
	lockDir := t.TempDir()
	lockPath := filepath.Join(lockDir, "meowctl.lock")

	// Write a lock entry with the wrong integrity hash.
	lockContent := "[modules.mymod]\nversion = \"v1.0.0\"\nsource = \"" +
		tarballServer.URL + "/mymod-v1.0.0.tar.gz\"\nintegrity = \"" + wrongInteg + "\"\n"
	if err := os.WriteFile(lockPath, []byte(lockContent), 0o600); err != nil {
		t.Fatalf("write lock: %v", err)
	}

	cl := newTestRegistryLoader(t, indexServer, cacheDir, lockPath)
	thread := &gostarlark.Thread{Name: "test"}

	_, err := cl.Load(thread, "@mymod//lib.star", gostarlark.StringDict{})
	if err == nil {
		t.Fatal("expected SRI mismatch error, got nil")
	}
}

// TestRegistryLoader_CacheHit verifies that the tarball is not re-fetched when
// the module is already extracted in the cache.
func TestRegistryLoader_CacheHit(t *testing.T) {
	modContent := []byte(`hit = True`)
	tarball := buildTarGz(t, map[string][]byte{"lib.star": modContent})

	tarballCallCount := 0
	tarballServer := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		tarballCallCount++
		if r.URL.Path == "/mymod-v1.0.0.tar.gz" {
			w.WriteHeader(http.StatusOK)
			_, _ = w.Write(tarball)
			return
		}
		http.NotFound(w, r)
	}))
	defer tarballServer.Close()

	indexBody := registryIndexTOML(t, map[string]map[string]interface{}{
		"mymod": {
			"versions": []string{"v1.0.0"},
			"source":   tarballServer.URL + "/mymod-{version}.tar.gz",
		},
	})
	indexServer := testServer(t, map[string][]byte{"/": indexBody})
	defer indexServer.Close()

	cacheDir := t.TempDir()
	lockPath := filepath.Join(t.TempDir(), "meowctl.lock")

	// First load — downloads and extracts.
	cl := newTestRegistryLoader(t, indexServer, cacheDir, lockPath)
	thread := &gostarlark.Thread{Name: "test"}
	if _, err := cl.Load(thread, "@mymod//lib.star", gostarlark.StringDict{}); err != nil {
		t.Fatalf("first load: %v", err)
	}
	prevCount := tarballCallCount

	// Second load on a fresh CompositeLoader using the same cache + lock.
	cl2 := newTestRegistryLoader(t, indexServer, cacheDir, lockPath)
	if _, err := cl2.Load(thread, "@mymod//lib.star", gostarlark.StringDict{}); err != nil {
		t.Fatalf("second load: %v", err)
	}

	if tarballCallCount > prevCount {
		t.Errorf("tarball re-fetched on cache hit: before=%d after=%d", prevCount, tarballCallCount)
	}
}

// TestRegistryLoader_LockedVersion verifies that a module with a lock entry uses
// the locked version without running MVS.
func TestRegistryLoader_LockedVersion(t *testing.T) {
	modContent := []byte(`locked = True`)
	tarball := buildTarGz(t, map[string][]byte{"lib.star": modContent})
	integ := registrySRI(tarball)

	tarballServer := testServer(t, map[string][]byte{
		"/mymod-v1.2.0.tar.gz": tarball,
	})
	defer tarballServer.Close()

	// Index has v2.0.0 as latest, but lock pins v1.2.0.
	indexBody := registryIndexTOML(t, map[string]map[string]interface{}{
		"mymod": {
			"versions": []string{"v1.2.0", "v2.0.0"},
			"source":   tarballServer.URL + "/mymod-{version}.tar.gz",
		},
	})
	indexServer := testServer(t, map[string][]byte{"/": indexBody})
	defer indexServer.Close()

	cacheDir := t.TempDir()
	lockDir := t.TempDir()
	lockPath := filepath.Join(lockDir, "meowctl.lock")

	// Write lock pinning v1.2.0.
	lockContent := "[modules.mymod]\nversion = \"v1.2.0\"\nsource = \"" +
		tarballServer.URL + "/mymod-v1.2.0.tar.gz\"\nintegrity = \"" + integ + "\"\n"
	if err := os.WriteFile(lockPath, []byte(lockContent), 0o600); err != nil {
		t.Fatalf("write lock: %v", err)
	}

	cl := newTestRegistryLoader(t, indexServer, cacheDir, lockPath)
	thread := &gostarlark.Thread{Name: "test"}

	globals, err := cl.Load(thread, "@mymod//lib.star", gostarlark.StringDict{})
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if _, ok := globals["locked"]; !ok {
		t.Error("expected 'locked' in globals")
	}
}

// TestRegistryLoader_MVSDiamond verifies that MVS selects the maximum required
// version when two modules require different versions of a common dependency.
//
//	root → A@v1.0.0, B@v1.0.0
//	A@v1.0.0 → dep@v1.0.0
//	B@v1.0.0 → dep@v2.0.0
//
// Expected: dep@v2.0.0 selected.
func TestRegistryLoader_MVSDiamond(t *testing.T) {
	// dep@v1.0.0 tarball.
	dep1Content := []byte(`dep_version = "v1.0.0"`)
	dep1Tarball := buildTarGz(t, map[string][]byte{"dep.star": dep1Content})

	// dep@v2.0.0 tarball.
	dep2Content := []byte(`dep_version = "v2.0.0"`)
	dep2Tarball := buildTarGz(t, map[string][]byte{"dep.star": dep2Content})

	// mod-a@v1.0.0: depends on dep@v1.0.0.
	modAContent := []byte(`a_val = 1`)
	modATarball := buildTarGz(t, map[string][]byte{
		"a.star":      modAContent,
		"MODULE.meow": []byte(`dep("dep", "v1.0.0")`),
	})

	// mod-b@v1.0.0: depends on dep@v2.0.0.
	modBContent := []byte(`b_val = 2`)
	modBTarball := buildTarGz(t, map[string][]byte{
		"b.star":      modBContent,
		"MODULE.meow": []byte(`dep("dep", "v2.0.0")`),
	})

	// root module: depends on mod-a and mod-b.
	rootTarball := buildTarGz(t, map[string][]byte{
		"root.star":   []byte(`root_val = 0`),
		"MODULE.meow": []byte(`dep("mod-a", "v1.0.0")` + "\n" + `dep("mod-b", "v1.0.0")`),
	})

	tarballServer := testServer(t, map[string][]byte{
		"/root-v1.0.0.tar.gz":  rootTarball,
		"/mod-a-v1.0.0.tar.gz": modATarball,
		"/mod-b-v1.0.0.tar.gz": modBTarball,
		"/dep-v1.0.0.tar.gz":   dep1Tarball,
		"/dep-v2.0.0.tar.gz":   dep2Tarball,
	})
	defer tarballServer.Close()

	base := tarballServer.URL + "/{name}-{version}.tar.gz"
	indexBody := registryIndexTOML(t, map[string]map[string]interface{}{
		"root":  {"versions": []string{"v1.0.0"}, "source": base},
		"mod-a": {"versions": []string{"v1.0.0"}, "source": base},
		"mod-b": {"versions": []string{"v1.0.0"}, "source": base},
		"dep":   {"versions": []string{"v1.0.0", "v2.0.0"}, "source": base},
	})
	indexServer := testServer(t, map[string][]byte{"/": indexBody})
	defer indexServer.Close()

	cacheDir := t.TempDir()
	lockPath := filepath.Join(t.TempDir(), "meowctl.lock")

	cl := newTestRegistryLoader(t, indexServer, cacheDir, lockPath)
	thread := &gostarlark.Thread{Name: "test"}

	// Load root — MVS must resolve dep@v2.0.0 and write it to the lock.
	globals, err := cl.Load(thread, "@root//root.star", gostarlark.StringDict{})
	if err != nil {
		t.Fatalf("load root: %v", err)
	}
	if _, ok := globals["root_val"]; !ok {
		t.Error("expected 'root_val' in globals")
	}

	// Assert the lock file records dep at v2.0.0 (proving MVS selected the max version).
	lockData, err := os.ReadFile(lockPath) // #nosec G304 -- lockPath is a t.TempDir() path
	if err != nil {
		t.Fatalf("read lock: %v", err)
	}
	lockStr := string(lockData)
	if !strings.Contains(lockStr, `[modules.dep]`) {
		t.Error("lock file does not contain [modules.dep] entry")
	}
	if !strings.Contains(lockStr, `version = "v2.0.0"`) {
		t.Errorf("expected lock to pin dep at v2.0.0, got:\n%s", lockStr)
	}

	// Load @dep//dep.star on a fresh loader (uses locked v2.0.0, not re-resolving).
	cl2 := newTestRegistryLoader(t, indexServer, cacheDir, lockPath)
	depGlobals, err := cl2.Load(thread, "@dep//dep.star", gostarlark.StringDict{})
	if err != nil {
		t.Fatalf("load dep: %v", err)
	}
	dv, ok := depGlobals["dep_version"]
	if !ok {
		t.Fatal("expected 'dep_version' in dep globals")
	}
	if dv.String() != `"v2.0.0"` {
		t.Errorf("expected dep_version = v2.0.0, got %v", dv)
	}
}

// TestRegistryLoader_CompatWarn verifies that a registry index with compat > 1
// causes a warning to be printed to stderr, but does NOT fail the load.
func TestRegistryLoader_CompatWarn(t *testing.T) {
	modContent := []byte(`compat_val = "ok"`)
	tarball := buildTarGz(t, map[string][]byte{"compat.star": modContent})

	tarballServer := testServer(t, map[string][]byte{
		"/compat-v1.0.0.tar.gz": tarball,
	})
	defer tarballServer.Close()

	// Build a raw TOML index with compat = 2.
	indexBody := []byte(fmt.Sprintf(
		"compat = 2\n\n[modules.compat]\nversions = [\"v1.0.0\"]\nsource = \"%s/compat-{version}.tar.gz\"\n",
		tarballServer.URL,
	))

	indexServer := testServer(t, map[string][]byte{"/": indexBody})
	defer indexServer.Close()

	// Redirect os.Stderr to a pipe so we can capture the warning.
	origStderr := os.Stderr
	r, w, err := os.Pipe()
	if err != nil {
		t.Fatalf("os.Pipe: %v", err)
	}
	os.Stderr = w

	cacheDir := t.TempDir()
	lockPath := filepath.Join(t.TempDir(), "meowctl.lock")
	cl := newTestRegistryLoader(t, indexServer, cacheDir, lockPath)
	thread := &gostarlark.Thread{Name: "test"}

	_, loadErr := cl.Load(thread, "@compat//compat.star", gostarlark.StringDict{})

	// Restore stderr before any assertions.
	if err := w.Close(); err != nil {
		t.Fatalf("close write pipe: %v", err)
	}
	os.Stderr = origStderr
	var stderrBuf bytes.Buffer
	_, _ = io.Copy(&stderrBuf, r)
	if err := r.Close(); err != nil {
		t.Fatalf("close read pipe: %v", err)
	}

	if loadErr != nil {
		t.Fatalf("Load should succeed even with compat=2, got: %v", loadErr)
	}
	if !strings.Contains(stderrBuf.String(), "compat=2") {
		t.Errorf("expected stderr warning containing 'compat=2', got: %q", stderrBuf.String())
	}
}

// TestRegistryLoader_CompatNoWarn verifies that a registry index with no compat
// field (TOML zero-value → Compat == 0) does NOT emit any warning to stderr.
// This is the case a loader sees when fetching a legacy or minimal index.toml
// that omits the compat key entirely.
func TestRegistryLoader_CompatNoWarn(t *testing.T) {
	modContent := []byte(`quiet_val = "quiet"`)
	tarball := buildTarGz(t, map[string][]byte{"quiet.star": modContent})

	tarballServer := testServer(t, map[string][]byte{
		"/quiet-v1.0.0.tar.gz": tarball,
	})
	defer tarballServer.Close()

	// Build a raw TOML index with NO compat field — Go decodes missing int as 0,
	// which is ≤ 1 → silent (per ADR-004 zero-value contract).
	indexBody := []byte(fmt.Sprintf(
		"[modules.quiet]\nversions = [\"v1.0.0\"]\nsource = \"%s/quiet-{version}.tar.gz\"\n",
		tarballServer.URL,
	))

	indexServer := testServer(t, map[string][]byte{"/": indexBody})
	defer indexServer.Close()

	origStderr := os.Stderr
	r, w, err := os.Pipe()
	if err != nil {
		t.Fatalf("os.Pipe: %v", err)
	}
	os.Stderr = w

	cacheDir := t.TempDir()
	lockPath := filepath.Join(t.TempDir(), "meowctl.lock")
	cl := newTestRegistryLoader(t, indexServer, cacheDir, lockPath)
	thread := &gostarlark.Thread{Name: "test"}

	_, loadErr := cl.Load(thread, "@quiet//quiet.star", gostarlark.StringDict{})

	if err := w.Close(); err != nil {
		t.Fatalf("close write pipe: %v", err)
	}
	os.Stderr = origStderr
	var stderrBuf bytes.Buffer
	_, _ = io.Copy(&stderrBuf, r)
	if err := r.Close(); err != nil {
		t.Fatalf("close read pipe: %v", err)
	}

	if loadErr != nil {
		t.Fatalf("Load should succeed with no compat field, got: %v", loadErr)
	}
	if strings.Contains(stderrBuf.String(), "compat") {
		t.Errorf("expected no compat warning for absent compat field, got: %q", stderrBuf.String())
	}
}

// TestRegistryLoader_NoExtensionNormalised verifies that a URL path without a
// file extension resolves to init.star inside the named directory.
// @mymod//lib resolves to lib/init.star.
func TestRegistryLoader_NoExtensionNormalised(t *testing.T) {
	modContent := []byte(`norm = True`)
	tarball := buildTarGz(t, map[string][]byte{"lib/init.star": modContent})

	tarballServer := testServer(t, map[string][]byte{
		"/mymod-v1.0.0.tar.gz": tarball,
	})
	defer tarballServer.Close()

	indexBody := registryIndexTOML(t, map[string]map[string]interface{}{
		"mymod": {
			"versions": []string{"v1.0.0"},
			"source":   tarballServer.URL + "/mymod-{version}.tar.gz",
		},
	})
	indexServer := testServer(t, map[string][]byte{"/": indexBody})
	defer indexServer.Close()

	cacheDir := t.TempDir()
	lockPath := filepath.Join(t.TempDir(), "meowctl.lock")
	cl := newTestRegistryLoader(t, indexServer, cacheDir, lockPath)
	thread := &gostarlark.Thread{Name: "test"}

	// Load without .star extension.
	globals, err := cl.Load(thread, "@mymod//lib", gostarlark.StringDict{})
	if err != nil {
		t.Fatalf("Load without .star extension: %v", err)
	}
	if _, ok := globals["norm"]; !ok {
		t.Error("expected 'norm' in globals")
	}
}

// --- parseGitHubSource tests ---

// TestParseGitHubSource_Valid verifies happy-path decomposition of well-formed strings.
func TestParseGitHubSource_Valid(t *testing.T) {
	cases := []struct {
		input     string
		wantOwner string
		wantRepo  string
		wantRef   string
	}{
		{"github:owner/repo@v1.2.3", "owner", "repo", "v1.2.3"},
		{"github:my-org/my-repo@main", "my-org", "my-repo", "main"},
		{"github:foo/bar@abc1234def5678", "foo", "bar", "abc1234def5678"},
	}
	for _, tc := range cases {
		owner, repo, ref, err := loader.ParseGitHubSourceForTest(tc.input)
		if err != nil {
			t.Errorf("ParseGitHubSource(%q): unexpected error: %v", tc.input, err)
			continue
		}
		if owner != tc.wantOwner || repo != tc.wantRepo || ref != tc.wantRef {
			t.Errorf("ParseGitHubSource(%q) = (%q, %q, %q), want (%q, %q, %q)",
				tc.input, owner, repo, ref, tc.wantOwner, tc.wantRepo, tc.wantRef)
		}
	}
}

// TestParseGitHubSource_Invalid verifies that malformed strings produce errors.
func TestParseGitHubSource_Invalid(t *testing.T) {
	cases := []string{
		"gitlab:owner/repo@v1.0.0", // wrong scheme
		"github:owner/repo",        // missing @ref
		"github:owner/repo@",       // empty ref
		"github:onlyowner@v1.0.0",  // missing repo (no slash)
		"github:@v1.0.0",           // empty owner
		"github:/repo@v1.0.0",      // empty owner with slash
	}
	for _, s := range cases {
		_, _, _, err := loader.ParseGitHubSourceForTest(s)
		if err == nil {
			t.Errorf("ParseGitHubSource(%q): expected error, got nil", s)
		}
	}
}

// --- SyncModules GitHub source dep tests ---

// TestSyncModules_GitHubSourceDep verifies that a dep with source="github:owner/repo@ref"
// resolves the commit SHA via the GitHub API, downloads the tarball, and writes
// the lock entry with CommitSHA set.
func TestSyncModules_GitHubSourceDep(t *testing.T) {
	const fakeCommit = "aabbccdd1122334455667788990011223344556677"

	modContent := []byte(`gh_val = "synced"`)
	tarball := buildTarGz(t, map[string][]byte{"lib.star": modContent})

	// Fake tarball server — served at /<owner>/<repo>/archive/<sha>.tar.gz
	tarballServer := testServer(t, map[string][]byte{
		"/owner/myrepo/archive/" + fakeCommit + ".tar.gz": tarball,
	})
	defer tarballServer.Close()

	// Fake GitHub API server — returns commit SHA JSON
	commitJSON := []byte(`{"sha":"` + fakeCommit + `"}`)
	apiServer := testServer(t, map[string][]byte{
		"/repos/owner/myrepo/commits/v1.0.0": commitJSON,
	})
	defer apiServer.Close()

	cacheDir := t.TempDir()
	lockPath := filepath.Join(t.TempDir(), "meowctl.lock")

	rl := &loader.RegistryLoader{
		CacheDir:          filepath.Join(cacheDir, "modules"),
		LockPath:          lockPath,
		GitHubAPIBase:     apiServer.URL,
		GitHubTarballBase: tarballServer.URL,
	}

	deps := []loader.ModfileDep{
		{Name: "mypkg", Source: "github:owner/myrepo@v1.0.0"},
	}
	result, err := rl.SyncModules(deps, nil)
	if err != nil {
		t.Fatalf("SyncModules: unexpected error: %v", err)
	}
	sha, ok := result.Resolved["mypkg"]
	if !ok {
		t.Fatal("expected 'mypkg' in Resolved")
	}
	if sha != fakeCommit {
		t.Errorf("expected commit SHA %q, got %q", fakeCommit, sha)
	}

	// Lock file must have CommitSHA set.
	lf, err := lock.Read(lockPath)
	if err != nil {
		t.Fatalf("read lock: %v", err)
	}
	entry, ok := lf.Modules["mypkg"]
	if !ok {
		t.Fatal("expected 'mypkg' in lock file")
	}
	if entry.CommitSHA != fakeCommit {
		t.Errorf("lock CommitSHA: want %q, got %q", fakeCommit, entry.CommitSHA)
	}
}

// TestSyncModules_GitHubSourceDep_CommitResolutionFailure verifies that a bad
// GitHub API response (non-200) causes SyncModules to return an error.
func TestSyncModules_GitHubSourceDep_CommitResolutionFailure(t *testing.T) {
	// API server always returns 404.
	apiServer := testServer(t, map[string][]byte{})
	defer apiServer.Close()

	cacheDir := t.TempDir()
	lockPath := filepath.Join(t.TempDir(), "meowctl.lock")

	rl := &loader.RegistryLoader{
		CacheDir:      filepath.Join(cacheDir, "modules"),
		LockPath:      lockPath,
		GitHubAPIBase: apiServer.URL,
	}

	deps := []loader.ModfileDep{
		{Name: "mypkg", Source: "github:owner/repo@v1.0.0"},
	}
	_, err := rl.SyncModules(deps, nil)
	if err == nil {
		t.Fatal("expected error when GitHub API returns 404, got nil")
	}
	if !strings.Contains(err.Error(), "resolve commit") {
		t.Errorf("expected 'resolve commit' in error, got: %v", err)
	}
}

// TestSyncModules_GitHubSourceDep_InvalidSource verifies that a dep with a
// malformed source string causes SyncModules to return an error.
func TestSyncModules_GitHubSourceDep_InvalidSource(t *testing.T) {
	cacheDir := t.TempDir()
	lockPath := filepath.Join(t.TempDir(), "meowctl.lock")

	rl := &loader.RegistryLoader{
		CacheDir: filepath.Join(cacheDir, "modules"),
		LockPath: lockPath,
	}

	deps := []loader.ModfileDep{
		{Name: "mypkg", Source: "badscheme:owner/repo@v1.0.0"},
	}
	_, err := rl.SyncModules(deps, nil)
	if err == nil {
		t.Fatal("expected error for invalid source, got nil")
	}
}

// TestSyncModules_ReplaceWithSource verifies that a replace() with source=
// overrides the dep source and is treated as a GitHub dep.
func TestSyncModules_ReplaceWithSource(t *testing.T) {
	// API server always returns 404 — we just want to verify the GitHub path is taken.
	apiServer := testServer(t, map[string][]byte{})
	defer apiServer.Close()

	cacheDir := t.TempDir()
	lockPath := filepath.Join(t.TempDir(), "meowctl.lock")

	rl := &loader.RegistryLoader{
		CacheDir:      filepath.Join(cacheDir, "modules"),
		LockPath:      lockPath,
		GitHubAPIBase: apiServer.URL,
	}

	deps := []loader.ModfileDep{
		{Name: "mypkg", Version: "v1.0.0"}, // registry dep…
	}
	replaces := []loader.ModfileReplace{
		{Name: "mypkg", Source: "github:fork/repo@main"}, // …overridden by source replace
	}
	_, err := rl.SyncModules(deps, replaces)
	// Expect an error (404 from fake API), but it must be the GitHub-path error
	// (resolve commit), NOT a "not found in registry" error.
	if err == nil {
		t.Fatal("expected error when GitHub API returns 404, got nil")
	}
	if strings.Contains(err.Error(), "not found in registry") {
		t.Errorf("source-replace should bypass registry, got registry error: %v", err)
	}
	if !strings.Contains(err.Error(), "resolve commit") {
		t.Errorf("expected 'resolve commit' error via GitHub path, got: %v", err)
	}
}

// TestRegistryLoader_ReplaceLocal verifies that a Replaces entry serves files
// from the local filesystem root using the init.star convention.
func TestRegistryLoader_ReplaceLocal(t *testing.T) {
	localRoot := t.TempDir()
	aptDir := filepath.Join(localRoot, "components", "apt")
	if err := os.MkdirAll(aptDir, 0o750); err != nil {
		t.Fatalf("mkdir: %v", err)
	}
	if err := os.WriteFile(filepath.Join(aptDir, "init.star"), []byte(`pm_name = "apt"`), 0o600); err != nil {
		t.Fatalf("write: %v", err)
	}

	cacheDir := t.TempDir()
	lockPath := filepath.Join(t.TempDir(), "meowctl.lock")
	cl := loader.NewRegistryLoaderForTest(
		t.TempDir(),
		&syntax.FileOptions{},
		loader.CompositeLoaderOptions{
			CacheDir: cacheDir,
			LockPath: lockPath,
			Client:   &http.Client{},
			Replaces: map[string]string{"stdlib": localRoot},
		},
		"", "", "",
	)
	thread := &gostarlark.Thread{Name: "test"}

	// With explicit init.star extension.
	globals, err := cl.Load(thread, "@stdlib//components/apt/init.star", gostarlark.StringDict{})
	if err != nil {
		t.Fatalf("Load with init.star: %v", err)
	}
	if _, ok := globals["pm_name"]; !ok {
		t.Error("expected 'pm_name' in globals (with init.star)")
	}

	// Without extension — resolves to init.star inside the directory.
	globals2, err := cl.Load(thread, "@stdlib//components/apt", gostarlark.StringDict{})
	if err != nil {
		t.Fatalf("Load without extension: %v", err)
	}
	if _, ok := globals2["pm_name"]; !ok {
		t.Error("expected 'pm_name' in globals (without extension)")
	}
}

// TestSyncModules_GitHubDep_TransitiveDeps verifies that a GitHub dep whose
// MODULE.meow declares another source dep causes that transitive dep to be
// resolved and written to the lock file.
func TestSyncModules_GitHubDep_TransitiveDeps(t *testing.T) {
	const commitA = "aaaa000000000000000000000000000000000000"
	const commitB = "bbbb000000000000000000000000000000000000"

	// dep-b tarball — no MODULE.meow (leaf).
	tarballB := buildTarGz(t, map[string][]byte{"lib.star": []byte(`b_val = 2`)})

	// dep-a tarball — MODULE.meow declares dep-b as a transitive source dep.
	moduleMeowA := []byte(`dep(name="dep-b", source="github:owner/dep-b@main")` + "\n")
	tarballA := buildTarGz(t, map[string][]byte{
		"lib.star":    []byte(`a_val = 1`),
		"MODULE.meow": moduleMeowA,
	})

	// Tarball server serves both tarballs.
	tarballServer := testServer(t, map[string][]byte{
		"/owner/dep-a/archive/" + commitA + ".tar.gz": tarballA,
		"/owner/dep-b/archive/" + commitB + ".tar.gz": tarballB,
	})
	defer tarballServer.Close()

	// API server resolves both refs.
	apiServer := testServer(t, map[string][]byte{
		"/repos/owner/dep-a/commits/main": []byte(`{"sha":"` + commitA + `"}`),
		"/repos/owner/dep-b/commits/main": []byte(`{"sha":"` + commitB + `"}`),
	})
	defer apiServer.Close()

	cacheDir := t.TempDir()
	lockPath := filepath.Join(t.TempDir(), "meowctl.lock")

	rl := &loader.RegistryLoader{
		CacheDir:          filepath.Join(cacheDir, "modules"),
		LockPath:          lockPath,
		GitHubAPIBase:     apiServer.URL,
		GitHubTarballBase: tarballServer.URL,
	}

	deps := []loader.ModfileDep{
		{Name: "dep-a", Source: "github:owner/dep-a@main"},
	}
	result, err := rl.SyncModules(deps, nil)
	if err != nil {
		t.Fatalf("SyncModules: unexpected error: %v", err)
	}

	// Both dep-a and transitive dep-b must appear in Resolved.
	if sha := result.Resolved["dep-a"]; sha != commitA {
		t.Errorf("dep-a resolved SHA: want %q, got %q", commitA, sha)
	}
	if sha := result.Resolved["dep-b"]; sha != commitB {
		t.Errorf("dep-b transitive SHA: want %q, got %q", commitB, sha)
	}

	// Lock file must contain both entries.
	lf, err := lock.Read(lockPath)
	if err != nil {
		t.Fatalf("read lock: %v", err)
	}
	if lf.Modules["dep-a"].CommitSHA != commitA {
		t.Errorf("lock dep-a CommitSHA: want %q, got %q", commitA, lf.Modules["dep-a"].CommitSHA)
	}
	if lf.Modules["dep-b"].CommitSHA != commitB {
		t.Errorf("lock dep-b CommitSHA: want %q, got %q", commitB, lf.Modules["dep-b"].CommitSHA)
	}
}

// TestSyncModules_GitHubDep_CircularDeps verifies that a circular MODULE.meow
// dependency (A → B → A) does not cause infinite recursion and completes without error.
func TestSyncModules_GitHubDep_CircularDeps(t *testing.T) {
	const commitA = "aaaa111111111111111111111111111111111111"
	const commitB = "bbbb111111111111111111111111111111111111"

	// dep-a MODULE.meow → dep-b; dep-b MODULE.meow → dep-a (circular).
	tarballA := buildTarGz(t, map[string][]byte{
		"lib.star":    []byte(`a_val = 1`),
		"MODULE.meow": []byte(`dep(name="dep-b", source="github:owner/dep-b@main")` + "\n"),
	})
	tarballB := buildTarGz(t, map[string][]byte{
		"lib.star":    []byte(`b_val = 2`),
		"MODULE.meow": []byte(`dep(name="dep-a", source="github:owner/dep-a@main")` + "\n"),
	})

	tarballServer := testServer(t, map[string][]byte{
		"/owner/dep-a/archive/" + commitA + ".tar.gz": tarballA,
		"/owner/dep-b/archive/" + commitB + ".tar.gz": tarballB,
	})
	defer tarballServer.Close()

	apiServer := testServer(t, map[string][]byte{
		"/repos/owner/dep-a/commits/main": []byte(`{"sha":"` + commitA + `"}`),
		"/repos/owner/dep-b/commits/main": []byte(`{"sha":"` + commitB + `"}`),
	})
	defer apiServer.Close()

	cacheDir := t.TempDir()
	lockPath := filepath.Join(t.TempDir(), "meowctl.lock")

	rl := &loader.RegistryLoader{
		CacheDir:          filepath.Join(cacheDir, "modules"),
		LockPath:          lockPath,
		GitHubAPIBase:     apiServer.URL,
		GitHubTarballBase: tarballServer.URL,
	}

	// SyncModules must terminate and not error due to circular deps.
	result, err := rl.SyncModules([]loader.ModfileDep{
		{Name: "dep-a", Source: "github:owner/dep-a@main"},
	}, nil)
	if err != nil {
		t.Fatalf("SyncModules with circular deps: unexpected error: %v", err)
	}
	// Both modules resolved despite the cycle.
	if result.Resolved["dep-a"] == "" {
		t.Error("expected dep-a in Resolved")
	}
	if result.Resolved["dep-b"] == "" {
		t.Error("expected dep-b in Resolved (transitive via circular reference)")
	}
}
