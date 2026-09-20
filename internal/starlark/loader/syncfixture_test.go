package loader_test

import (
	"archive/tar"
	"bytes"
	"compress/gzip"
	"crypto/sha512"
	"encoding/base64"
	"fmt"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"testing"

	"github.com/meowshed/meowctl/internal/starlark/loader"
)

// TestWriteSyncFixture regenerates tests/compat/fixtures/deps.lock.synced,
// the lock v0.1.0 writes from a known manifest against a known registry.
//
// It is the oracle for the Rust sync: the two must produce the same bytes from
// the same inputs. Skipped unless MEOWCTL_WRITE_FIXTURE is set, so an ordinary
// test run does not rewrite a checked-in file.
//
// Regenerate with:
//
//	MEOWCTL_WRITE_FIXTURE=1 go test ./internal/starlark/loader -run TestWriteSyncFixture
func TestWriteSyncFixture(t *testing.T) {
	if os.Getenv("MEOWCTL_WRITE_FIXTURE") == "" {
		t.Skip("set MEOWCTL_WRITE_FIXTURE=1 to regenerate the fixture")
	}

	stdlib := buildTarball(t, map[string]string{
		"MODULE.meow":              "module(name = \"stdlib\", version = \"0.2.17\", compat = 2)\n",
		"components/zsh/init.star": "ZSH = 1\n",
	})
	helper := buildTarball(t, map[string]string{
		"MODULE.meow": "module(name = \"helper\", version = \"1.1.0\")\n",
		"init.star":   "HELPER = 1\n",
	})

	mux := http.NewServeMux()
	var base string
	mux.HandleFunc("/index.toml", func(w http.ResponseWriter, _ *http.Request) {
		_, _ = fmt.Fprintf(w, `compat = 1

[modules.stdlib]
versions = ["0.2.16", "0.2.17"]
source = "%s/t/{name}-{version_no_v}.tar.gz"

[modules.stdlib.integrity]
"0.2.17" = "%s"

[modules.helper]
versions = ["1.0.0", "1.1.0"]
source = "%s/t/{name}-{version_no_v}.tar.gz"

[modules.helper.integrity]
"1.1.0" = "%s"
`, base, sri(stdlib), base, sri(helper))
	})
	mux.HandleFunc("/t/stdlib-0.2.17.tar.gz", func(w http.ResponseWriter, _ *http.Request) {
		_, _ = w.Write(stdlib)
	})
	mux.HandleFunc("/t/helper-1.1.0.tar.gz", func(w http.ResponseWriter, _ *http.Request) {
		_, _ = w.Write(helper)
	})
	server := httptest.NewServer(mux)
	defer server.Close()
	base = server.URL

	dir := t.TempDir()
	lockPath := filepath.Join(dir, "deps.lock")
	l := &loader.RegistryLoader{
		RegistryURL: server.URL + "/index.toml",
		CacheDir:    filepath.Join(dir, "cache"),
		LockPath:    lockPath,
	}

	deps := []loader.ModfileDep{
		{Name: "stdlib", Version: "0.2.17"},
		{Name: "helper", Version: "1.1.0"},
		{Name: "forked", Version: "0.1.0"},
	}
	replaces := []loader.ModfileReplace{
		{Name: "forked", Path: dir},
	}
	if _, err := l.SyncModules(deps, replaces); err != nil {
		t.Fatalf("sync: %v", err)
	}

	written, err := os.ReadFile(lockPath) // #nosec G304 -- the path is this test's temporary directory
	if err != nil {
		t.Fatalf("read lock: %v", err)
	}
	// The server's port and the temporary directory change on every run, so
	// they are replaced by placeholders the Rust side substitutes back.
	text := bytes.ReplaceAll(written, []byte(server.URL), []byte("{registry}"))
	text = bytes.ReplaceAll(text, []byte(dir), []byte("{checkout}"))

	dir2 := filepath.Join("..", "..", "..", "tests", "compat", "fixtures")
	if err := os.MkdirAll(dir2, 0o750); err != nil {
		t.Fatalf("mkdir: %v", err)
	}
	out := filepath.Join(dir2, "deps.lock.synced")
	// #nosec G703 -- every path here is a constant in this file; nothing
	// reaching it came from outside the test.
	if err := os.WriteFile(out, text, 0o600); err != nil {
		t.Fatalf("write fixture: %v", err)
	}
	// The tarballs go beside the lock so the other implementation hashes the
	// same bytes. Without them the two agree on every field but the hashes,
	// which are the fields the lock exists to carry.
	for name, body := range map[string][]byte{
		"synced-stdlib-0.2.17.tar.gz": stdlib,
		"synced-helper-1.1.0.tar.gz":  helper,
	} {
		if err := os.WriteFile(filepath.Join(dir2, name), body, 0o600); err != nil {
			t.Fatalf("write %s: %v", name, err)
		}
	}
	t.Logf("wrote %s", out)
}

func buildTarball(t *testing.T, files map[string]string) []byte {
	t.Helper()
	var buf bytes.Buffer
	gz := gzip.NewWriter(&buf)
	tw := tar.NewWriter(gz)
	names := make([]string, 0, len(files))
	for name := range files {
		names = append(names, name)
	}
	// Deterministic order, so the tarball's hash is the same on every run.
	for i := 0; i < len(names); i++ {
		for j := i + 1; j < len(names); j++ {
			if names[j] < names[i] {
				names[i], names[j] = names[j], names[i]
			}
		}
	}
	for _, name := range names {
		body := files[name]
		if err := tw.WriteHeader(&tar.Header{
			Name:     name,
			Mode:     0o644,
			Size:     int64(len(body)),
			Typeflag: tar.TypeReg,
		}); err != nil {
			t.Fatalf("tar header: %v", err)
		}
		if _, err := tw.Write([]byte(body)); err != nil {
			t.Fatalf("tar body: %v", err)
		}
	}
	if err := tw.Close(); err != nil {
		t.Fatalf("tar close: %v", err)
	}
	if err := gz.Close(); err != nil {
		t.Fatalf("gzip close: %v", err)
	}
	return buf.Bytes()
}

func sri(data []byte) string {
	sum := sha512.Sum384(data)
	return "sha384-" + base64.StdEncoding.EncodeToString(sum[:])
}
