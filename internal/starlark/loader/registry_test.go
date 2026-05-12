package loader_test

import (
	"archive/tar"
	"bytes"
	"compress/gzip"
	"crypto/sha512"
	"encoding/base64"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/BurntSushi/toml"
	gostarlark "go.starlark.net/starlark"
	"go.starlark.net/syntax"

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
