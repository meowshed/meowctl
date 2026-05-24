package cli

import (
	"archive/tar"
	"compress/gzip"
	"context"
	"errors"
	"fmt"
	"io"
	"net/http"
	"os"
	"path/filepath"
	"strings"

	"github.com/meowshed/meowctl/internal/state"
)

// bootstrapMaxTarballBytes caps the tarball download at 256 MiB to avoid
// unbounded memory/disk usage from a malicious or runaway server response.
const bootstrapMaxTarballBytes = 256 << 20

// runBootstrap implements "meowctl init <repo-url>".
// It downloads a dotfiles repo as a tarball, extracts it to configDir,
// then runs dep sync and apply.
func runBootstrap(configDir, repoURL string, force bool) error {
	entryPath := filepath.Join(configDir, configEntryFile)

	// Guard: init.star already exists.
	if _, err := os.Lstat(entryPath); err == nil {
		if !force {
			return exitErrorf(ExitConfig,
				"config already initialized at %s\n  delete it or run 'meowctl init --force' to overwrite",
				entryPath)
		}
		// --force: remove the entire configDir and re-create it below.
		if err := os.RemoveAll(configDir); err != nil {
			return fmt.Errorf("init: remove existing config: %w", err)
		}
	}

	if err := os.MkdirAll(configDir, 0o700); err != nil {
		return fmt.Errorf("init: create config dir: %w", err)
	}

	// Download tarball.
	url := tarballURL(repoURL)
	fmt.Printf("meowctl: downloading %s\n", url)
	tmpPath, err := downloadTarball(url)
	if err != nil {
		return err
	}
	defer func() { _ = os.Remove(tmpPath) }()

	// Extract tarball to configDir.
	fmt.Printf("meowctl: extracting to %s\n", configDir)
	if err := extractTarball(tmpPath, configDir); err != nil {
		return err
	}

	// Verify init.star is present after extraction.
	if _, err := os.Lstat(entryPath); err != nil {
		_ = os.RemoveAll(configDir)
		return fmt.Errorf("init: init.star not found after extraction — is this a meowctl dotfiles repo?")
	}

	// Run dep sync (resolve deps.mod → deps.lock); skip if deps.lock already present.
	lockPath := filepath.Join(configDir, configLockFile)
	if _, err := os.Lstat(lockPath); errors.Is(err, os.ErrNotExist) {
		fmt.Println("meowctl: running dep sync")
		if err := runSync(configDir); err != nil {
			return fmt.Errorf("init: dep sync: %w", err)
		}
	} else {
		fmt.Println("meowctl: deps.lock present — skipping dep sync")
	}

	// Run apply.
	fmt.Println("meowctl: running apply")
	if err := runApply(runConfig{ConfigDir: configDir}, nil); err != nil {
		return err
	}

	// Persist repo URL for later meowctl update.
	mgr := state.NewManager(filepath.Join(configDir, configStateFile))
	s, _ := mgr.Load()
	s.RepoURL = repoURL
	_ = mgr.Save(s) // Non-fatal: update works without stored URL.
	return nil
}

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
		return "", fmt.Errorf("init: build request for %s: %w", url, err)
	}

	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		return "", fmt.Errorf("init: download %s: %w", url, err)
	}
	defer func() { _ = resp.Body.Close() }()

	if resp.StatusCode != http.StatusOK {
		return "", fmt.Errorf("init: download %s: server returned %s", url, resp.Status)
	}

	tmp, err := os.CreateTemp("", "meowctl-init-*.tar.gz")
	if err != nil {
		return "", fmt.Errorf("init: create temp file: %w", err)
	}

	limited := &io.LimitedReader{R: resp.Body, N: bootstrapMaxTarballBytes + 1}
	n, err := io.Copy(tmp, limited)
	if err != nil {
		_ = tmp.Close()
		_ = os.Remove(tmp.Name())
		return "", fmt.Errorf("init: write temp file: %w", err)
	}
	if err := tmp.Close(); err != nil {
		_ = os.Remove(tmp.Name())
		return "", fmt.Errorf("init: close temp file: %w", err)
	}
	if n > bootstrapMaxTarballBytes {
		_ = os.Remove(tmp.Name())
		return "", fmt.Errorf("init: tarball exceeds %d bytes limit", bootstrapMaxTarballBytes)
	}
	return tmp.Name(), nil
}

// extractTarball extracts a .tar.gz file to destDir, stripping the single
// top-level directory that GitHub/GitLab tarballs wrap everything in.
// All extracted paths are validated to stay within destDir (no path traversal).
func extractTarball(tarPath, destDir string) error {
	f, err := os.Open(tarPath) // #nosec G304 -- path comes from our own temp file
	if err != nil {
		return fmt.Errorf("init: open tarball: %w", err)
	}
	defer func() { _ = f.Close() }()

	gz, err := gzip.NewReader(f)
	if err != nil {
		return fmt.Errorf("init: decompress tarball: %w", err)
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
			return fmt.Errorf("init: read tarball: %w", err)
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
		return "", false, fmt.Errorf("init: unsafe path in tarball: %s", name)
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
		return "", fmt.Errorf("init: path escapes destination: %s", originalName)
	}
	return destPath, nil
}

// writeEntry writes a single tar entry (directory or regular file) to destPath.
func writeEntry(hdr *tar.Header, destPath string, r io.Reader) error {
	switch hdr.Typeflag {
	case tar.TypeDir:
		if err := os.MkdirAll(destPath, 0o700); err != nil {
			return fmt.Errorf("init: create directory %s: %w", destPath, err)
		}
	case tar.TypeReg:
		if err := os.MkdirAll(filepath.Dir(destPath), 0o700); err != nil {
			return fmt.Errorf("init: create parent for %s: %w", destPath, err)
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
		return fmt.Errorf("init: create %s: %w", path, err)
	}
	if _, err := io.Copy(out, r); err != nil { // #nosec G110 -- tarball size capped by downloadTarball
		_ = out.Close()
		return fmt.Errorf("init: write %s: %w", path, err)
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
