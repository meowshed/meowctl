package cli

import (
	"archive/tar"
	"bytes"
	"compress/gzip"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"testing"
)

// makeTarball creates an in-memory .tar.gz with the given files under a top-level
// directory named "repo-main" (mimicking GitHub's tarball format).
func makeTarball(t *testing.T, files map[string]string) []byte {
	t.Helper()
	var buf bytes.Buffer
	gw := gzip.NewWriter(&buf)
	tw := tar.NewWriter(gw)

	// Write the top-level directory entry.
	_ = tw.WriteHeader(&tar.Header{
		Typeflag: tar.TypeDir,
		Name:     "repo-main/",
	})

	for name, content := range files {
		_ = tw.WriteHeader(&tar.Header{
			Typeflag: tar.TypeReg,
			Name:     "repo-main/" + name,
			Size:     int64(len(content)),
			Mode:     0o600,
		})
		_, _ = tw.Write([]byte(content))
	}
	_ = tw.Close()
	_ = gw.Close()
	return buf.Bytes()
}

// TestTarballURL verifies the GitHub tarball URL derivation.
func TestTarballURL(t *testing.T) {
	cases := []struct {
		repoURL string
		want    string
	}{
		{
			"https://github.com/meowshed/dotmeow",
			"https://github.com/meowshed/dotmeow/archive/refs/heads/main.tar.gz",
		},
		{
			"https://github.com/meowshed/dotmeow/", // trailing slash stripped
			"https://github.com/meowshed/dotmeow/archive/refs/heads/main.tar.gz",
		},
	}
	for _, tc := range cases {
		got := tarballURL(tc.repoURL)
		if got != tc.want {
			t.Errorf("tarballURL(%q): got %q, want %q", tc.repoURL, got, tc.want)
		}
	}
}

// TestContainsDotDot verifies path traversal detection.
func TestContainsDotDot(t *testing.T) {
	cases := []struct {
		path string
		want bool
	}{
		{"foo/bar", false},
		{"foo/../bar", true},
		{"..", true},
		{"foo/..bar", false}, // "..bar" is not ".."
		{"../etc/passwd", true},
	}
	for _, tc := range cases {
		got := containsDotDot(tc.path)
		if got != tc.want {
			t.Errorf("containsDotDot(%q): got %v, want %v", tc.path, got, tc.want)
		}
	}
}

// TestExtractTarball_Basic verifies that files are extracted and the top-level
// directory prefix is stripped.
func TestExtractTarball_Basic(t *testing.T) {
	tarball := makeTarball(t, map[string]string{
		"init.star":        "# init",
		"deps.mod":         "module my-dotfiles",
		"components/a.txt": "hello",
	})

	// Write tarball to a temp file.
	tmp, err := os.CreateTemp(t.TempDir(), "*.tar.gz")
	if err != nil {
		t.Fatal(err)
	}
	if _, err := tmp.Write(tarball); err != nil {
		t.Fatal(err)
	}
	_ = tmp.Close()

	destDir := t.TempDir()
	if err := extractTarball(tmp.Name(), destDir); err != nil {
		t.Fatalf("extractTarball: %v", err)
	}

	// Verify extracted files.
	for _, rel := range []string{"init.star", "deps.mod", "components/a.txt"} {
		path := filepath.Join(destDir, filepath.FromSlash(rel))
		if _, err := os.Lstat(path); err != nil {
			t.Errorf("expected extracted file %s: %v", rel, err)
		}
	}
}

// TestExtractTarball_PreservesExecBit verifies that the executable bit from tar
// headers survives extraction (normalized to 0644/0755), so distributed scripts
// remain runnable.
func TestExtractTarball_PreservesExecBit(t *testing.T) {
	var buf bytes.Buffer
	gw := gzip.NewWriter(&buf)
	tw := tar.NewWriter(gw)
	_ = tw.WriteHeader(&tar.Header{Typeflag: tar.TypeDir, Name: "repo-main/"})
	entries := []struct {
		name string
		mode int64
	}{
		{"scripts/run.sh", 0o755},
		{"README.md", 0o644},
	}
	for _, e := range entries {
		_ = tw.WriteHeader(&tar.Header{
			Typeflag: tar.TypeReg,
			Name:     "repo-main/" + e.name,
			Size:     int64(len("x")),
			Mode:     e.mode,
		})
		_, _ = tw.Write([]byte("x"))
	}
	_ = tw.Close()
	_ = gw.Close()

	tmp, err := os.CreateTemp(t.TempDir(), "*.tar.gz")
	if err != nil {
		t.Fatal(err)
	}
	if _, err := tmp.Write(buf.Bytes()); err != nil {
		t.Fatal(err)
	}
	_ = tmp.Close()

	destDir := t.TempDir()
	if err := extractTarball(tmp.Name(), destDir); err != nil {
		t.Fatalf("extractTarball: %v", err)
	}

	cases := map[string]os.FileMode{
		"scripts/run.sh": 0o755,
		"README.md":      0o644,
	}
	for rel, want := range cases {
		info, statErr := os.Stat(filepath.Join(destDir, filepath.FromSlash(rel)))
		if statErr != nil {
			t.Fatalf("stat %s: %v", rel, statErr)
		}
		if got := info.Mode().Perm(); got != want {
			t.Errorf("%s perm: got %o, want %o", rel, got, want)
		}
	}
}

// TestExtractTarball_PathTraversal verifies that a tarball with ".." entries is rejected.
func TestExtractTarball_PathTraversal(t *testing.T) {
	var buf bytes.Buffer
	gw := gzip.NewWriter(&buf)
	tw := tar.NewWriter(gw)

	_ = tw.WriteHeader(&tar.Header{
		Typeflag: tar.TypeDir,
		Name:     "repo-main/",
	})
	// Malicious entry: try to escape via ../
	content := "evil"
	_ = tw.WriteHeader(&tar.Header{
		Typeflag: tar.TypeReg,
		Name:     "repo-main/../../evil.txt",
		Size:     int64(len(content)),
	})
	_, _ = tw.Write([]byte(content))
	_ = tw.Close()
	_ = gw.Close()

	tmp, err := os.CreateTemp(t.TempDir(), "*.tar.gz")
	if err != nil {
		t.Fatal(err)
	}
	if _, err := tmp.Write(buf.Bytes()); err != nil {
		t.Fatal(err)
	}
	_ = tmp.Close()

	destDir := t.TempDir()
	if err := extractTarball(tmp.Name(), destDir); err == nil {
		t.Fatal("expected error for path traversal, got nil")
	}
}

// TestDownloadTarball_ServesContent verifies downloadTarball fetches and saves the body.
func TestDownloadTarball_ServesContent(t *testing.T) {
	payload := makeTarball(t, map[string]string{"init.star": "# hi"})

	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
		w.Header().Set("Content-Type", "application/gzip")
		_, _ = w.Write(payload)
	}))
	defer srv.Close()

	tmpPath, err := downloadTarball(srv.URL + "/archive/refs/heads/main.tar.gz")
	if err != nil {
		t.Fatalf("downloadTarball: %v", err)
	}
	defer func() { _ = os.Remove(tmpPath) }()

	data, err := os.ReadFile(tmpPath) // #nosec G304 -- tmpPath is our own temp file
	if err != nil {
		t.Fatal(err)
	}
	if !bytes.Equal(data, payload) {
		t.Fatalf("downloaded content does not match payload")
	}
}

// TestDownloadTarball_Non200 verifies that a non-200 response is an error.
func TestDownloadTarball_Non200(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
		w.WriteHeader(http.StatusNotFound)
	}))
	defer srv.Close()

	if _, err := downloadTarball(srv.URL + "/nope"); err == nil {
		t.Fatal("expected error for 404, got nil")
	}
}

// TestRunBootstrap_AlreadyExists verifies that bootstrap fails if init.star exists without --force.
func TestRunBootstrap_AlreadyExists(t *testing.T) {
	configDir := t.TempDir()
	entryPath := filepath.Join(configDir, configEntryFile)
	if err := os.WriteFile(entryPath, []byte("# existing"), 0o600); err != nil {
		t.Fatal(err)
	}

	// We use a no-op server; the guard should fire before any download.
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
		w.WriteHeader(http.StatusOK)
	}))
	defer srv.Close()

	err := runBootstrap(configDir, srv.URL, false)
	if err == nil {
		t.Fatal("expected error when init.star exists without --force")
	}
}

// TestRunBootstrap_NoInitStar verifies that bootstrap cleans up and errors if
// the tarball does not contain init.star.
func TestRunBootstrap_NoInitStar(t *testing.T) {
	// Tarball with no init.star.
	tarball := makeTarball(t, map[string]string{
		"deps.mod": "module test",
	})

	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
		_, _ = w.Write(tarball)
	}))
	defer srv.Close()

	configDir := t.TempDir()
	err := runBootstrap(configDir, srv.URL, false)
	if err == nil {
		t.Fatal("expected error when tarball contains no init.star")
	}
}
