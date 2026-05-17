package cli

import (
	"archive/tar"
	"compress/gzip"
	"context"
	"fmt"
	"io"
	"net/http"
	"os"
	"path/filepath"
	"strings"
)

// bootstrapMaxTarballBytes caps the tarball download at 256 MiB to avoid
// unbounded memory/disk usage from a malicious or runaway server response.
const bootstrapMaxTarballBytes = 256 << 20

// tarballURL derives the tarball download URL from a repo URL.
// Convention: <repo-url>/archive/refs/heads/main.tar.gz (GitHub / GitLab).
func tarballURL(repoURL string) string {
	repoURL = strings.TrimSuffix(repoURL, "/")
	return repoURL + "/archive/refs/heads/main.tar.gz"
}

// downloadTarball downloads the tarball at url to a temp file and returns the
// path. The caller is responsible for removing the temp file when done.
func downloadTarball(url string) (string, error) {
	req, err := http.NewRequestWithContext(context.Background(), http.MethodGet, url, nil) // #nosec G107 -- URL is user-supplied
	if err != nil {
		return "", fmt.Errorf("bootstrap: build request for %s: %w", url, err)
	}

	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		return "", fmt.Errorf("bootstrap: download %s: %w", url, err)
	}
	defer func() { _ = resp.Body.Close() }()

	if resp.StatusCode != http.StatusOK {
		return "", fmt.Errorf("bootstrap: download %s: server returned %s", url, resp.Status)
	}

	tmp, err := os.CreateTemp("", "meowctl-bootstrap-*.tar.gz")
	if err != nil {
		return "", fmt.Errorf("bootstrap: create temp file: %w", err)
	}

	limited := &io.LimitedReader{R: resp.Body, N: bootstrapMaxTarballBytes + 1}
	n, err := io.Copy(tmp, limited)
	if err != nil {
		_ = tmp.Close()
		_ = os.Remove(tmp.Name())
		return "", fmt.Errorf("bootstrap: write temp file: %w", err)
	}
	if err := tmp.Close(); err != nil {
		_ = os.Remove(tmp.Name())
		return "", fmt.Errorf("bootstrap: close temp file: %w", err)
	}
	if n > bootstrapMaxTarballBytes {
		_ = os.Remove(tmp.Name())
		return "", fmt.Errorf("bootstrap: tarball exceeds %d bytes limit", bootstrapMaxTarballBytes)
	}
	return tmp.Name(), nil
}

// extractTarball extracts a .tar.gz file to destDir, stripping the single
// top-level directory that GitHub/GitLab tarballs wrap everything in.
// All extracted paths are validated to stay within destDir (no path traversal).
func extractTarball(tarPath, destDir string) error {
	f, err := os.Open(tarPath) // #nosec G304 -- path comes from our own temp file
	if err != nil {
		return fmt.Errorf("bootstrap: open tarball: %w", err)
	}
	defer func() { _ = f.Close() }()

	gz, err := gzip.NewReader(f)
	if err != nil {
		return fmt.Errorf("bootstrap: decompress tarball: %w", err)
	}
	defer func() { _ = gz.Close() }()

	return extractTarEntries(tar.NewReader(gz), destDir)
}

// extractTarEntries iterates over tar entries and writes them to destDir.
// It strips the single top-level directory prefix present in GitHub/GitLab tarballs.
func extractTarEntries(tr *tar.Reader, destDir string) error {
	topDir := ""

	for {
		hdr, err := tr.Next()
		if err == io.EOF {
			break
		}
		if err != nil {
			return fmt.Errorf("bootstrap: read tarball: %w", err)
		}

		relPath, skip, err := stripTopDir(hdr.Name, &topDir)
		if err != nil {
			return err
		}
		if skip {
			continue
		}

		destPath, err := safeDestPath(relPath, destDir, hdr.Name)
		if err != nil {
			return err
		}

		if err := writeEntry(hdr, destPath, tr); err != nil {
			return err
		}
	}
	return nil
}

// stripTopDir strips the top-level directory prefix from a tar entry name.
// Returns the relative path, a skip flag (true for the top dir entry itself), and any error.
func stripTopDir(name string, topDir *string) (relPath string, skip bool, err error) {
	parts := strings.SplitN(filepath.ToSlash(name), "/", 2)
	if *topDir == "" {
		*topDir = parts[0]
	}
	if len(parts) < 2 || parts[1] == "" {
		return "", true, nil
	}
	if containsDotDot(parts[1]) {
		return "", false, fmt.Errorf("bootstrap: unsafe path in tarball: %s", name)
	}
	return parts[1], false, nil
}

// safeDestPath converts relPath to an absolute path inside destDir and
// validates it does not escape destDir.
func safeDestPath(relPath, destDir, originalName string) (string, error) {
	destPath := filepath.Join(destDir, filepath.FromSlash(relPath))
	if !strings.HasPrefix(
		filepath.Clean(destPath)+string(os.PathSeparator),
		filepath.Clean(destDir)+string(os.PathSeparator),
	) {
		return "", fmt.Errorf("bootstrap: path escapes destination: %s", originalName)
	}
	return destPath, nil
}

// writeEntry writes a single tar entry (directory or regular file) to destPath.
func writeEntry(hdr *tar.Header, destPath string, r io.Reader) error {
	switch hdr.Typeflag {
	case tar.TypeDir:
		if err := os.MkdirAll(destPath, 0o700); err != nil {
			return fmt.Errorf("bootstrap: create directory %s: %w", destPath, err)
		}
	case tar.TypeReg:
		if err := os.MkdirAll(filepath.Dir(destPath), 0o700); err != nil {
			return fmt.Errorf("bootstrap: create parent for %s: %w", destPath, err)
		}
		if err := writeExtractedFile(destPath, r); err != nil {
			return err
		}
	default:
		// Skip symlinks, hard links, devices, etc.
	}
	return nil
}

// writeExtractedFile writes the current tar entry to path.
func writeExtractedFile(path string, r io.Reader) error {
	out, err := os.OpenFile(path, os.O_CREATE|os.O_WRONLY|os.O_TRUNC, 0o600) // #nosec G304
	if err != nil {
		return fmt.Errorf("bootstrap: create %s: %w", path, err)
	}
	if _, err := io.Copy(out, r); err != nil { // #nosec G110 -- tarball size capped by downloadTarball
		_ = out.Close()
		return fmt.Errorf("bootstrap: write %s: %w", path, err)
	}
	return out.Close()
}

// containsDotDot reports whether p contains a ".." path component.
func containsDotDot(p string) bool {
	for _, part := range strings.Split(filepath.ToSlash(p), "/") {
		if part == ".." {
			return true
		}
	}
	return false
}
