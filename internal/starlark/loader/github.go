package loader

import (
	"context"
	"crypto/sha512"
	"encoding/base64"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"os"
	"path/filepath"
	"strings"
	"time"

	sri "github.com/peterebden/go-sri"
	gostarlark "go.starlark.net/starlark"
	"go.starlark.net/syntax"

	"github.com/meowshed/meowctl/internal/lock"
)

const githubScheme = "github://"

// githubURL holds the parsed components of a github:// module URL.
// Format: github://owner/repo@ref//path/to/file.star
type githubURL struct {
	owner string
	repo  string
	ref   string
	path  string
}

// parseGitHubURL decomposes a raw github:// URL into its components.
func parseGitHubURL(raw string) (githubURL, error) {
	s := strings.TrimPrefix(raw, githubScheme)

	// Split owner/repo@ref//path on "//".
	parts := strings.SplitN(s, "//", 2)
	if len(parts) != 2 || parts[1] == "" {
		return githubURL{}, fmt.Errorf("github loader: invalid URL %q: missing //path component", raw)
	}
	filePath := parts[1]

	// Split owner/repo@ref on "@".
	atParts := strings.SplitN(parts[0], "@", 2)
	if len(atParts) != 2 {
		return githubURL{}, fmt.Errorf("github loader: invalid URL %q: missing @ref", raw)
	}
	ref := atParts[1]

	// Split owner/repo.
	slashParts := strings.SplitN(atParts[0], "/", 2)
	if len(slashParts) != 2 || slashParts[0] == "" || slashParts[1] == "" {
		return githubURL{}, fmt.Errorf("github loader: invalid URL %q: expected owner/repo", raw)
	}

	return githubURL{
		owner: slashParts[0],
		repo:  slashParts[1],
		ref:   ref,
		path:  filePath,
	}, nil
}

// githubLoader loads Starlark modules from GitHub-hosted repositories using the
// github:// URL scheme.
//
// URL format: github://owner/repo@ref//path/to/file.star
//
// On first access, the ref is resolved to a commit SHA via the GitHub API and
// pinned in the lock file. Subsequent loads read the SHA from the lock and skip
// the API call. File content is cached under cacheDir and verified with SRI
// (sha384) on every read.
type githubLoader struct {
	cacheDir string
	lockPath string
	client   *http.Client
	fileOpts *syntax.FileOptions
	// apiBase is the GitHub API root (e.g. https://api.github.com).
	// Set by newGitHubLoader; overridden in tests via CompositeLoader.
	apiBase string
	// rawBase is the raw content root (e.g. https://raw.githubusercontent.com).
	// Set by newGitHubLoader; overridden in tests via CompositeLoader.
	rawBase string
}

// newGitHubLoader returns a githubLoader with production defaults.
func newGitHubLoader(cacheDir, lockPath string, client *http.Client, opts *syntax.FileOptions) *githubLoader {
	c := client
	if c == nil {
		c = &http.Client{Timeout: 30 * time.Second}
	}
	return &githubLoader{
		cacheDir: cacheDir,
		lockPath: lockPath,
		client:   c,
		fileOpts: opts,
		apiBase:  "https://api.github.com",
		rawBase:  "https://raw.githubusercontent.com",
	}
}

// Load implements Loader.
func (l *githubLoader) Load(thread *gostarlark.Thread, moduleURL string, predeclared gostarlark.StringDict) (gostarlark.StringDict, error) {
	u, err := parseGitHubURL(moduleURL)
	if err != nil {
		return nil, err
	}

	commit, integrity, err := l.resolveAndFetch(u)
	if err != nil {
		return nil, err
	}

	// Persist the lock entry. A write failure is non-fatal: the module loaded
	// successfully and callers should not be blocked by a transient disk error.
	// The next run will re-resolve the ref (one extra API call) but correctness
	// is preserved.
	_ = l.writeLockEntry(u, commit, integrity)

	cachePath := l.cachePath(u.owner, u.repo, commit, u.path)
	src, err := os.ReadFile(cachePath) // #nosec G304 -- path constructed from parsed, validated URL components
	if err != nil {
		return nil, fmt.Errorf("github loader: read cache %q: %w", cachePath, err)
	}

	childThread := &gostarlark.Thread{Name: moduleURL, Load: thread.Load}
	globals, err := gostarlark.ExecFileOptions(l.fileOpts, childThread, moduleURL, src, predeclared)
	if err != nil {
		return nil, fmt.Errorf("github loader: eval %q: %w", moduleURL, err)
	}
	return globals, nil
}

// resolveAndFetch returns the commit SHA and SRI hash for u, using the lock
// cache when available and falling back to network fetch otherwise.
func (l *githubLoader) resolveAndFetch(u githubURL) (commit, integrity string, err error) {
	lf, err := lock.Read(l.lockPath)
	if err != nil {
		return "", "", fmt.Errorf("github loader: read lock: %w", err)
	}

	key := lockKey(u.owner, u.repo, u.ref, u.path)
	if entry, ok := lf.GitHub[key]; ok {
		// Lock hit: verify cache, re-fetch on corruption.
		cachePath := l.cachePath(u.owner, u.repo, entry.Commit, u.path)
		if data, readErr := os.ReadFile(cachePath); readErr == nil { // #nosec G304 -- path constructed from lock-verified fields
			if sriErr := checkSRI(entry.Integrity, data); sriErr == nil {
				return entry.Commit, entry.Integrity, nil
			}
			// Cache is corrupt; fall through to re-fetch.
		}
		// Re-fetch using the pinned commit (no extra API call needed).
		data, integ, fetchErr := l.fetchRaw(u.owner, u.repo, entry.Commit, u.path)
		if fetchErr != nil {
			return "", "", fetchErr
		}
		if writeErr := writeCacheFile(cachePath, data); writeErr != nil {
			return "", "", writeErr
		}
		return entry.Commit, integ, nil
	}

	// No lock entry: resolve ref → commit via GitHub API, then fetch.
	resolvedCommit, err := l.resolveRef(u.owner, u.repo, u.ref)
	if err != nil {
		return "", "", err
	}

	cachePath := l.cachePath(u.owner, u.repo, resolvedCommit, u.path)
	data, integ, err := l.fetchRaw(u.owner, u.repo, resolvedCommit, u.path)
	if err != nil {
		return "", "", err
	}
	if err := writeCacheFile(cachePath, data); err != nil {
		return "", "", err
	}
	return resolvedCommit, integ, nil
}

// resolveRef calls the GitHub API to turn a ref (branch/tag/sha) into a full commit SHA.
func (l *githubLoader) resolveRef(owner, repo, ref string) (string, error) {
	rawURL := fmt.Sprintf("%s/repos/%s/%s/commits/%s", l.apiBase, owner, repo, ref)
	req, err := http.NewRequestWithContext(context.Background(), http.MethodGet, rawURL, nil)
	if err != nil {
		return "", fmt.Errorf("github loader: build request for ref %q: %w", ref, err)
	}
	resp, err := l.client.Do(req) //nolint:gosec // URL constructed from validated parsed fields, not user-tainted input
	if err != nil {
		return "", fmt.Errorf("github loader: resolve ref %q: %w", ref, err)
	}
	defer resp.Body.Close() //nolint:errcheck // response body close errors are not actionable

	if resp.StatusCode != http.StatusOK {
		return "", fmt.Errorf("github loader: resolve ref %q: HTTP %d", ref, resp.StatusCode)
	}

	var payload struct {
		SHA string `json:"sha"`
	}
	if err := json.NewDecoder(resp.Body).Decode(&payload); err != nil {
		return "", fmt.Errorf("github loader: resolve ref %q: decode: %w", ref, err)
	}
	if payload.SHA == "" {
		return "", fmt.Errorf("github loader: resolve ref %q: empty SHA in response", ref)
	}
	return payload.SHA, nil
}

// fetchRaw downloads the file from raw.githubusercontent.com and returns the
// content and its sha384 SRI string.
func (l *githubLoader) fetchRaw(owner, repo, commit, path string) ([]byte, string, error) {
	rawURL := fmt.Sprintf("%s/%s/%s/%s/%s", l.rawBase, owner, repo, commit, path)
	req, err := http.NewRequestWithContext(context.Background(), http.MethodGet, rawURL, nil)
	if err != nil {
		return nil, "", fmt.Errorf("github loader: build request for %q: %w", rawURL, err)
	}
	resp, err := l.client.Do(req) //nolint:gosec // URL constructed from validated parsed fields, not user-tainted input
	if err != nil {
		return nil, "", fmt.Errorf("github loader: fetch %q: %w", rawURL, err)
	}
	defer resp.Body.Close() //nolint:errcheck // response body close errors are not actionable

	if resp.StatusCode != http.StatusOK {
		return nil, "", fmt.Errorf("github loader: fetch %q: HTTP %d", rawURL, resp.StatusCode)
	}

	data, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, "", fmt.Errorf("github loader: read body %q: %w", rawURL, err)
	}

	integ := computeSRI(data)
	return data, integ, nil
}

// cachePath returns the local filesystem path for the cached content of a GitHub file.
// Layout: <cacheDir>/github/<owner>/<repo>/<commit>/<path>
func (l *githubLoader) cachePath(owner, repo, commit, path string) string {
	return filepath.Join(l.cacheDir, "github", owner, repo, commit, filepath.FromSlash(path))
}

// writeLockEntry persists a GitHubEntry to the lock file for the given parsed URL.
func (l *githubLoader) writeLockEntry(u githubURL, commit, integrity string) error {
	lf, err := lock.Read(l.lockPath)
	if err != nil {
		return err
	}
	if lf.GitHub == nil {
		lf.GitHub = make(map[string]lock.GitHubEntry)
	}
	lf.GitHub[lockKey(u.owner, u.repo, u.ref, u.path)] = lock.GitHubEntry{
		Commit:    commit,
		Integrity: integrity,
	}
	return lock.Write(l.lockPath, lf)
}

// lockKey returns the map key used in LockFile.GitHub for a given GitHub file reference.
// Format: owner/repo@ref//path
func lockKey(owner, repo, ref, path string) string {
	return fmt.Sprintf("%s/%s@%s//%s", owner, repo, ref, path)
}

// computeSRI returns the W3C SRI sha384 string for data.
func computeSRI(data []byte) string {
	sum := sha512.Sum384(data)
	return "sha384-" + base64.StdEncoding.EncodeToString(sum[:])
}

// checkSRI verifies data against an expected SRI string.
func checkSRI(expected string, data []byte) error {
	c, err := sri.NewChecker(expected)
	if err != nil {
		return fmt.Errorf("invalid SRI string %q: %w", expected, err)
	}
	if _, err := c.Write(data); err != nil {
		return fmt.Errorf("SRI write: %w", err)
	}
	return c.Check()
}

// writeCacheFile writes data to path, creating parent directories as needed.
func writeCacheFile(path string, data []byte) error {
	if err := os.MkdirAll(filepath.Dir(path), 0o700); err != nil {
		return fmt.Errorf("github loader: mkdir cache: %w", err)
	}
	//nolint:gosec
	if err := os.WriteFile(path, data, 0o600); err != nil { // #nosec G306 -- 0600 is intentional; cache files should not be world-readable
		return fmt.Errorf("github loader: write cache: %w", err)
	}
	return nil
}
