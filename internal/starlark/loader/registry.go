package loader

import (
	"archive/tar"
	"bytes"
	"compress/gzip"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net/http"
	"os"
	"path/filepath"
	"strings"
	"time"

	"github.com/BurntSushi/toml"
	gostarlark "go.starlark.net/starlark"
	"go.starlark.net/syntax"

	"github.com/meowshed/meowctl/internal/lock"
	"github.com/meowshed/meowctl/internal/mvs"
)

const (
	registryScheme     = "@"
	defaultRegistryURL = "https://raw.githubusercontent.com/meowshed/meowctl-registry/main/index.toml"
)

// indexEntry describes one module in the registry index.toml.
type indexEntry struct {
	// Versions is the list of available semver versions, in ascending order.
	Versions []string `toml:"versions"`
	// Source is a URL template for the tarball. {name} and {version} are replaced.
	Source string `toml:"source"`
	// Integrity maps version strings to their sha384 SRI hashes.
	// When present for a version, the loader verifies the downloaded tarball
	// against this hash before extracting. Missing entries are silently skipped
	// (backwards-compatible with modules that predate per-version integrity).
	Integrity map[string]string `toml:"integrity"`
}

// registryIndex is the top-level structure of index.toml.
type registryIndex struct {
	// Compat is the schema version of the index. Loaders MUST warn (not fail)
	// when Compat > 1. A missing compat field decodes as 0 (TOML zero-value),
	// which is treated identically to compat = 1 (silent). The supported compat
	// level is 1; update this comment and the warning text together if that changes.
	Compat  int                   `toml:"compat"`
	Modules map[string]indexEntry `toml:"modules"`
}

// RegistryLoader loads Starlark modules from the meowctl module registry.
// It handles the @name// URL scheme.
//
// URL format: @name//path/to/file[.star]
//
// The .star extension is optional; it is appended automatically if the path
// has no extension. @stdlib//components/apt and @stdlib//components/apt.star
// are equivalent.
//
// On first access, the registry index is fetched and MVS is run to select
// a version. The resolved version and tarball SRI are written to the lock
// file. Subsequent loads use the locked version and skip MVS.
type RegistryLoader struct {
	// RegistryURL is the URL of the registry index.toml.
	// Zero value uses the default meowshed registry.
	RegistryURL string
	// CacheDir is the root of the on-disk module cache (~/.cache/meowctl/modules).
	CacheDir string
	// LockPath is the path to the meowctl lock file.
	LockPath string
	// Client is the HTTP client used for all registry and tarball fetches.
	// Zero value creates a client with a 30-second timeout.
	Client *http.Client
	// FileOpts are the Starlark parse options propagated to child evaluations.
	FileOpts *syntax.FileOptions
	// Replaces maps registry module names to local filesystem paths.
	// When a module name is present here, Load serves files from the local
	// directory instead of fetching from the registry. The path within the
	// module URL (the part after //) is appended to the local root.
	// Example: {"stdlib": "/path/to/meowctl-stdlib"} makes
	// @stdlib//components/apt → /path/to/meowctl-stdlib/components/apt.star
	Replaces map[string]string
	// GitHubAPIBase overrides the GitHub API base URL (https://api.github.com).
	// Used in tests to point at a local httptest.Server.
	// Zero value uses the default production URL.
	GitHubAPIBase string
	// GitHubTarballBase overrides the GitHub tarball download base URL
	// (https://github.com). Used in tests to point at a local httptest.Server.
	// Zero value uses the default production URL.
	GitHubTarballBase string
}

// registryURL is the parsed form of a @name or @name// module URL.
type registryURL struct {
	module string
	path   string
}

// parseRegistryURL decomposes a @name or @name// URL into module name and file path.
// @name resolves to the module root init.star.
// @name//path resolves to path/init.star inside the module.
func parseRegistryURL(raw string) (registryURL, error) {
	if len(raw) < 2 || raw[0] != '@' {
		return registryURL{}, fmt.Errorf("registry loader: invalid URL %q: must start with @", raw)
	}
	s := raw[1:] // strip '@'
	idx := strings.Index(s, "//")
	if idx < 0 {
		// @name — root component, resolves to init.star at module root.
		return registryURL{module: s, path: "init.star"}, nil
	}
	if idx == 0 {
		return registryURL{}, fmt.Errorf("registry loader: invalid URL %q: missing module name before //", raw)
	}
	filePath := s[idx+2:]
	if filePath == "" {
		return registryURL{}, fmt.Errorf("registry loader: invalid URL %q: empty path after //", raw)
	}
	modName := s[:idx]
	// Normalise: path without extension resolves to path/init.star.
	if !strings.Contains(filepath.Base(filePath), ".") {
		filePath += "/init.star"
	}
	return registryURL{module: modName, path: filePath}, nil
}

// client returns the configured HTTP client or a default one.
func (l *RegistryLoader) client() *http.Client {
	if l.Client != nil {
		return l.Client
	}
	return &http.Client{Timeout: 30 * time.Second}
}

// registryIndexURL returns the registry index URL, falling back to the default.
func (l *RegistryLoader) registryIndexURL() string {
	if l.RegistryURL != "" {
		return l.RegistryURL
	}
	return defaultRegistryURL
}

// Load implements Loader.
func (l *RegistryLoader) Load(thread *gostarlark.Thread, moduleURL string, predeclared gostarlark.StringDict) (gostarlark.StringDict, error) {
	u, err := parseRegistryURL(moduleURL)
	if err != nil {
		return nil, err
	}

	// If a local replace is configured for this module, serve from the local path.
	if localRoot, ok := l.Replaces[u.module]; ok {
		return l.loadFromLocal(thread, moduleURL, localRoot, u.path, predeclared)
	}

	version, err := l.resolveVersion(u.module)
	if err != nil {
		return nil, fmt.Errorf("RegistryLoader: resolve %q: %w", moduleURL, err)
	}

	extractDir := l.moduleCacheDir(u.module, version)
	filePath := filepath.Join(extractDir, filepath.FromSlash(u.path))

	src, err := os.ReadFile(filePath) // #nosec G304 -- path constructed from lock-verified module name and version
	if err != nil {
		return nil, fmt.Errorf("RegistryLoader: read %q: %w", moduleURL, err)
	}

	childThread := &gostarlark.Thread{Name: moduleURL, Load: thread.Load}
	globals, err := gostarlark.ExecFileOptions(l.fileOpts(), childThread, moduleURL, src, predeclared)
	if err != nil {
		return nil, fmt.Errorf("RegistryLoader: eval %q: %w", moduleURL, err)
	}
	return globals, nil
}

// fileOpts returns the FileOptions to use for Starlark evaluation.
// Falls back to a zero-value FileOptions if the field is nil, so callers
// that construct RegistryLoader without setting FileOpts do not panic.
func (l *RegistryLoader) fileOpts() *syntax.FileOptions {
	if l.FileOpts != nil {
		return l.FileOpts
	}
	return &syntax.FileOptions{}
}

// loadFromLocal serves a @name//path load from a local filesystem root instead
// of the registry. Used when a replace override is active for the module.
func (l *RegistryLoader) loadFromLocal(thread *gostarlark.Thread, moduleURL, localRoot, filePath string, predeclared gostarlark.StringDict) (gostarlark.StringDict, error) {
	abs := filepath.Join(localRoot, filepath.FromSlash(filePath))

	// Reject path traversal.
	cleanRoot := filepath.Clean(localRoot) + string(os.PathSeparator)
	if !strings.HasPrefix(filepath.Clean(abs)+string(os.PathSeparator), cleanRoot) {
		return nil, fmt.Errorf("RegistryLoader: replace path %q escapes local root %q", filePath, localRoot)
	}

	src, err := os.ReadFile(abs) // #nosec G304 -- path validated against localRoot above
	if err != nil {
		return nil, fmt.Errorf("RegistryLoader: replace load %q: %w", moduleURL, err)
	}

	childThread := &gostarlark.Thread{Name: moduleURL, Load: thread.Load}
	globals, err := gostarlark.ExecFileOptions(l.fileOpts(), childThread, abs, src, predeclared)
	if err != nil {
		return nil, fmt.Errorf("RegistryLoader: replace eval %q: %w", moduleURL, err)
	}
	return globals, nil
}

// resolveVersion returns the cache-dir key for the named module. For registry
// deps this is the semver version; for GitHub source deps it is the commit SHA.
// If the lock file already has an entry, that value is used (no MVS). Otherwise
// MVS is run, all resolved modules are downloaded and written to the lock file.
func (l *RegistryLoader) resolveVersion(name string) (string, error) {
	lf, err := lock.Read(l.LockPath)
	if err != nil {
		return "", fmt.Errorf("read lock: %w", err)
	}

	if entry, ok := lf.Modules[name]; ok {
		// GitHub source dep: cache dir is keyed by commit SHA.
		if entry.CommitSHA != "" {
			extractDir := l.moduleCacheDir(name, entry.CommitSHA)
			if _, statErr := os.Stat(extractDir); statErr == nil {
				return entry.CommitSHA, nil
			}
			// Cache missing — re-download using locked source URL.
			if fetchErr := l.downloadAndExtract(name, entry.CommitSHA, entry.Source, entry.Integrity); fetchErr != nil {
				return "", fmt.Errorf("re-fetch locked github module %s@%s: %w", name, entry.CommitSHA, fetchErr)
			}
			return entry.CommitSHA, nil
		}

		// Registry dep: cache dir is keyed by version.
		extractDir := l.moduleCacheDir(name, entry.Version)
		if _, statErr := os.Stat(extractDir); statErr == nil {
			return entry.Version, nil
		}
		// Cache is missing; re-download using locked metadata.
		if fetchErr := l.downloadAndExtract(name, entry.Version, entry.Source, entry.Integrity); fetchErr != nil {
			return "", fmt.Errorf("re-fetch locked module %s@%s: %w", name, entry.Version, fetchErr)
		}
		return entry.Version, nil
	}

	// Not locked: fetch index, run MVS, download all resolved modules.
	index, err := l.fetchIndex()
	if err != nil {
		return "", err
	}

	entry, ok := index.Modules[name]
	if !ok {
		return "", fmt.Errorf("module %q not found in registry", name)
	}

	buildList, err := l.runMVS(name, entry, index)
	if err != nil {
		return "", err
	}

	// Download and lock every module in the build list. The root (index 0) is
	// processed first so the requested module is available even if a dep fails.
	for _, mod := range buildList {
		modEntry, ok := index.Modules[mod.Name]
		if !ok {
			return "", fmt.Errorf("resolved module %q not found in registry index", mod.Name)
		}
		if err := l.cacheAndLockModule(mod.Name, mod.Version, modEntry); err != nil {
			return "", err
		}
	}
	return buildList[0].Version, nil
}

// cacheAndLockModule downloads (if necessary), integrity-checks, extracts, and
// locks a single module version. It is called for every entry in the MVS build
// list during resolveVersion.
func (l *RegistryLoader) cacheAndLockModule(name, version string, entry indexEntry) error {
	source := buildSourceURL(entry.Source, name, version)
	cacheDir := l.moduleCacheDir(name, version)

	// Use the SRI sidecar written by Required() if the cache is already populated,
	// avoiding a redundant tarball download for modules fetched during MVS traversal.
	integ, sriErr := readSRISidecar(cacheDir)
	if sriErr != nil {
		// Cache missing or sidecar absent — download the tarball to get the SRI.
		tarball, fetchErr := l.fetchTarball(source)
		if fetchErr != nil {
			return fmt.Errorf("fetch tarball for %s@%s: %w", name, version, fetchErr)
		}
		// If the index carries a per-version integrity hash, verify before extracting.
		if verifyErr := checkIndexIntegrity(entry, version, tarball); verifyErr != nil {
			return fmt.Errorf("resolve: SRI mismatch for %s@%s (index integrity): %w", name, version, verifyErr)
		}
		integ = computeSRI(tarball)
		if _, statErr := os.Stat(cacheDir); statErr != nil {
			if extractErr := l.extractTarball(tarball, cacheDir); extractErr != nil {
				return fmt.Errorf("extract tarball for %s@%s: %w", name, version, extractErr)
			}
		}
		// Write the sidecar so future runs skip the re-download.
		_ = writeSRISidecar(cacheDir, integ)
	}
	if writeErr := l.writeLockEntry(name, version, source, integ); writeErr != nil {
		// Non-fatal: modules are cached; next run will re-resolve.
		fmt.Fprintf(os.Stderr, "meowctl: warning: could not write lock entry for %s@%s: %v\n", name, version, writeErr)
	}
	return nil
}

// runMVS resolves the full build list for name using Minimal Version Selection.
// It returns the complete list of resolved modules (root first).
func (l *RegistryLoader) runMVS(name string, entry indexEntry, index *registryIndex) ([]mvs.Module, error) {
	if len(entry.Versions) == 0 {
		return nil, fmt.Errorf("module %q has no versions in registry", name)
	}
	latest := entry.Versions[len(entry.Versions)-1]
	root := mvs.Module{Name: name, Version: latest}
	reqs := &registryReqs{loader: l, index: index}
	list, err := mvs.BuildList(root, reqs)
	if err != nil {
		return nil, fmt.Errorf("MVS for %q: %w", name, err)
	}
	return list, nil
}

// fetchIndex downloads and parses the registry index.toml.
func (l *RegistryLoader) fetchIndex() (*registryIndex, error) {
	body, err := l.httpGet(l.registryIndexURL())
	if err != nil {
		return nil, fmt.Errorf("fetch registry index: %w", err)
	}
	var idx registryIndex
	if _, err := toml.Decode(string(body), &idx); err != nil {
		return nil, fmt.Errorf("parse registry index: %w", err)
	}
	if idx.Compat > 1 {
		fmt.Fprintf(os.Stderr, "meowctl: warning: registry index requires compat=%d; this version of meowctl supports compat=1. Some features may be unavailable — upgrade meowctl if modules fail to load.\n", idx.Compat)
	}
	return &idx, nil
}

// fetchTarball downloads a tarball and returns its raw bytes.
func (l *RegistryLoader) fetchTarball(url string) ([]byte, error) {
	data, err := l.httpGet(url)
	if err != nil {
		return nil, fmt.Errorf("fetch tarball %q: %w", url, err)
	}
	return data, nil
}

// downloadAndExtract fetches a tarball, verifies its SRI, and extracts it.
func (l *RegistryLoader) downloadAndExtract(name, version, source, expectedInteg string) error {
	tarball, err := l.fetchTarball(source)
	if err != nil {
		return err
	}
	if err := checkSRI(expectedInteg, tarball); err != nil {
		return fmt.Errorf("SRI mismatch for %s@%s: %w", name, version, err)
	}
	return l.extractTarball(tarball, l.moduleCacheDir(name, version))
}

// httpGet performs a GET request and returns the response body.
func (l *RegistryLoader) httpGet(url string) ([]byte, error) {
	req, err := http.NewRequestWithContext(context.Background(), http.MethodGet, url, nil)
	if err != nil {
		return nil, fmt.Errorf("build request %q: %w", url, err)
	}
	resp, err := l.client().Do(req) //nolint:gosec // URL is from registry config or index, not raw user input
	if err != nil {
		return nil, fmt.Errorf("GET %q: %w", url, err)
	}
	defer resp.Body.Close() //nolint:errcheck // response body close errors are not actionable

	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("GET %q: HTTP %d", url, resp.StatusCode)
	}
	data, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, fmt.Errorf("read body %q: %w", url, err)
	}
	return data, nil
}

// extractTarball extracts a gzip-compressed tar archive to destDir.
// It rejects entries with absolute paths or path traversal components to
// prevent zip-slip attacks.
func (l *RegistryLoader) extractTarball(data []byte, destDir string) error {
	if err := os.MkdirAll(destDir, 0o700); err != nil {
		return fmt.Errorf("mkdir %q: %w", destDir, err)
	}

	gr, err := gzip.NewReader(bytes.NewReader(data))
	if err != nil {
		return fmt.Errorf("create gzip reader: %w", err)
	}
	defer gr.Close() //nolint:errcheck // gzip reader close errors are not actionable

	tr := tar.NewReader(gr)
	for {
		hdr, err := tr.Next()
		if errors.Is(err, io.EOF) {
			break
		}
		if err != nil {
			return fmt.Errorf("read tar entry: %w", err)
		}
		if err := extractTarEntry(hdr, tr, destDir); err != nil {
			return err
		}
	}
	return nil
}

// extractTarEntry writes a single tar entry to destDir after validating that
// the path does not escape the destination directory (zip-slip protection).
func extractTarEntry(hdr *tar.Header, tr *tar.Reader, destDir string) error {
	clean := filepath.Clean(hdr.Name)
	if filepath.IsAbs(clean) || strings.HasPrefix(clean, "..") {
		return fmt.Errorf("unsafe tar entry path %q", hdr.Name)
	}

	target := filepath.Join(destDir, clean)
	// Paranoid double-check: ensure target is still inside destDir.
	if !strings.HasPrefix(target+string(filepath.Separator), destDir+string(filepath.Separator)) {
		return fmt.Errorf("unsafe tar entry path %q escapes destination", hdr.Name)
	}

	switch hdr.Typeflag {
	case tar.TypeDir:
		if err := os.MkdirAll(target, 0o700); err != nil {
			return fmt.Errorf("mkdir %q: %w", target, err)
		}
	case tar.TypeReg:
		if err := os.MkdirAll(filepath.Dir(target), 0o700); err != nil {
			return fmt.Errorf("mkdir parent %q: %w", target, err)
		}
		if err := writeTarFile(target, tr); err != nil {
			return err
		}
	default:
		// Skip symlinks, hard links, and other special entries.
	}
	return nil
}

// writeTarFile creates or truncates target and copies the current tar entry into it.
func writeTarFile(target string, tr *tar.Reader) error {
	f, err := os.OpenFile(target, os.O_CREATE|os.O_WRONLY|os.O_TRUNC, 0o600) // #nosec G304 -- target validated against destDir in extractTarEntry
	if err != nil {
		return fmt.Errorf("create file %q: %w", target, err)
	}
	if _, err := io.Copy(f, tr); err != nil { //nolint:gosec // size bound by HTTP client; no zip bomb risk in this context
		_ = f.Close() // #nosec G104 -- best effort on error path; original error takes precedence
		return fmt.Errorf("write file %q: %w", target, err)
	}
	if err := f.Close(); err != nil {
		return fmt.Errorf("close file %q: %w", target, err)
	}
	return nil
}

// moduleCacheDir returns the local cache directory for a specific module version.
// Layout: <CacheDir>/<name>/<version>/
func (l *RegistryLoader) moduleCacheDir(name, version string) string {
	return filepath.Join(l.CacheDir, name, version)
}

// sriSidecarPath returns the path of the SRI sidecar file for a cache directory.
// The sidecar stores the tarball SRI so resolveVersion can skip a re-download
// for modules already extracted by Required() during MVS traversal.
func sriSidecarPath(cacheDir string) string {
	return filepath.Join(cacheDir, ".sri")
}

// writeSRISidecar atomically writes the SRI string to the sidecar file.
// Errors are non-fatal: the sidecar is an optimisation, not a correctness requirement.
func writeSRISidecar(cacheDir, integ string) error {
	path := sriSidecarPath(cacheDir)
	return os.WriteFile(path, []byte(integ), 0o600)
}

// readSRISidecar reads the SRI string from the sidecar file.
// Returns an error if the sidecar does not exist or cannot be read.
func readSRISidecar(cacheDir string) (string, error) {
	data, err := os.ReadFile(sriSidecarPath(cacheDir)) // #nosec G304 -- path constructed from validated cache dir
	if err != nil {
		return "", err
	}
	return string(data), nil
}

// checkIndexIntegrity verifies tarball against the index's per-version SRI hash
// for the given entry and version. It is a no-op when no hash is recorded for
// the version (backwards-compatible with modules that predate per-version integrity).
func checkIndexIntegrity(entry indexEntry, version string, tarball []byte) error {
	expected, ok := entry.Integrity[version]
	if !ok || expected == "" {
		return nil
	}
	return checkSRI(expected, tarball)
}

// writeLockEntry persists a ModuleEntry to the lock file.
func (l *RegistryLoader) writeLockEntry(name, version, source, integrity string) error {
	lf, err := lock.Read(l.LockPath)
	if err != nil {
		return err
	}
	if lf.Modules == nil {
		lf.Modules = make(map[string]lock.ModuleEntry)
	}
	lf.Modules[name] = lock.ModuleEntry{
		Version:   version,
		Source:    source,
		Integrity: integrity,
	}
	return lock.Write(l.LockPath, lf)
}

// SyncResult is returned by SyncModules.
type SyncResult struct {
	// Resolved maps module name to the version that was resolved and locked.
	// Replaced modules are not present in Resolved; they appear in ReplacedPaths instead.
	Resolved map[string]string
	// ReplacedPaths maps replaced module names to their local paths.
	ReplacedPaths map[string]string
}

// SyncModules resolves all deps in the modfile, downloads changed tarballs,
// and updates the lock file. It returns the resolved versions for diffing.
// Replace directives are validated: a non-existent local path is a hard error.
// Modules with active replace() are recorded in the lock with Replaced=true.
func (l *RegistryLoader) SyncModules(deps []ModfileDep, replaces []ModfileReplace) (*SyncResult, error) {
	// Build replace index for fast lookup.
	replaceIndex := make(map[string]ModfileReplace, len(replaces))
	for _, r := range replaces {
		replaceIndex[r.Name] = r
	}

	lf, err := lock.Read(l.LockPath)
	if err != nil {
		return nil, fmt.Errorf("sync: read lock: %w", err)
	}
	if lf.Modules == nil {
		lf.Modules = make(map[string]lock.ModuleEntry)
	}

	result := &SyncResult{
		Resolved:      make(map[string]string),
		ReplacedPaths: make(map[string]string),
	}

	// Only fetch the registry index when there are registry deps that need it.
	var index *registryIndex
	needsRegistry := false
	for _, dep := range deps {
		if dep.Source == "" {
			r, hasReplace := replaceIndex[dep.Name]
			if !hasReplace || r.Source != "" {
				// registry dep without a local-path replace needs the index
				needsRegistry = true
				break
			}
		}
	}
	if needsRegistry {
		index, err = l.fetchIndex()
		if err != nil {
			return nil, err
		}
	}

	for _, dep := range deps {
		if err := l.syncOneDep(dep, replaceIndex, index, lf.Modules, result); err != nil {
			return nil, err
		}
	}

	if writeErr := lock.Write(l.LockPath, lf); writeErr != nil {
		return nil, fmt.Errorf("sync: write lock: %w", writeErr)
	}
	return result, nil
}

// syncOneDep resolves a single dep entry, updating modules and result in-place.
func (l *RegistryLoader) syncOneDep(dep ModfileDep, replaceIndex map[string]ModfileReplace, index *registryIndex, modules map[string]lock.ModuleEntry, result *SyncResult) error {
	modName := dep.Name

	// Check for a replace directive first — it overrides the dep source.
	if r, replaced := replaceIndex[modName]; replaced {
		if r.Path != "" {
			// Local path override.
			fmt.Fprintf(os.Stderr, "meowctl: warning: module %q is replaced by local path %q — skipping registry resolution\n", modName, r.Path)
			if _, statErr := os.Stat(r.Path); statErr != nil {
				return fmt.Errorf("sync: replace() path for %q does not exist: %s", modName, r.Path)
			}
			modules[modName] = lock.ModuleEntry{Replaced: true, Path: r.Path}
			result.ReplacedPaths[modName] = r.Path
			return nil
		}
		if r.Source != "" {
			// Remote source override — treat as a GitHub source dep using the replace source.
			fmt.Fprintf(os.Stderr, "meowctl: warning: module %q is replaced by remote source %q\n", modName, r.Source)
			return l.syncGitHubDep(modName, r.Source, modules, result)
		}
	}

	// GitHub source dep (no replace active).
	if dep.Source != "" {
		return l.syncGitHubDep(modName, dep.Source, modules, result)
	}

	// Registry dep.
	if index == nil {
		return fmt.Errorf("sync: registry index not loaded for module %q", modName)
	}
	entry, ok := index.Modules[modName]
	if !ok {
		return fmt.Errorf("sync: module %q not found in registry", modName)
	}

	version, integ, source, err := l.resolveDepVersion(modName, dep.Version, entry)
	if err != nil {
		return err
	}

	modules[modName] = lock.ModuleEntry{
		Version:   version,
		Source:    source,
		Integrity: integ,
	}
	result.Resolved[modName] = version
	return nil
}

// syncGitHubDep resolves a GitHub source dep: parses the source string, resolves
// the ref to a commit SHA, downloads and extracts the tarball, and writes the lock entry.
// It also reads MODULE.meow from the extracted tarball to resolve transitive deps.
func (l *RegistryLoader) syncGitHubDep(modName, sourceStr string, modules map[string]lock.ModuleEntry, result *SyncResult) error {
	return l.syncGitHubDepVisited(modName, sourceStr, modules, result, make(map[string]struct{}))
}

// syncGitHubDepVisited is the internal implementation of syncGitHubDep that
// tracks visited module names to prevent infinite recursion on circular MODULE.meow deps.
func (l *RegistryLoader) syncGitHubDepVisited(modName, sourceStr string, modules map[string]lock.ModuleEntry, result *SyncResult, visited map[string]struct{}) error {
	if _, seen := visited[modName]; seen {
		return nil
	}
	visited[modName] = struct{}{}

	gs, err := parseGitHubSource(sourceStr)
	if err != nil {
		return fmt.Errorf("sync: dep %q: %w", modName, err)
	}

	commit, err := l.resolveGitHubCommit(gs.owner, gs.repo, gs.ref)
	if err != nil {
		return fmt.Errorf("sync: dep %q: resolve commit: %w", modName, err)
	}

	tarballURL := githubTarballURLWithBase(l.GitHubTarballBase, gs.owner, gs.repo, commit)
	cacheDir := l.moduleCacheDir(modName, commit)

	integ, sriErr := readSRISidecar(cacheDir)
	if sriErr != nil {
		tarball, fetchErr := l.fetchTarball(tarballURL)
		if fetchErr != nil {
			return fmt.Errorf("sync: dep %q: fetch tarball: %w", modName, fetchErr)
		}
		integ = computeSRI(tarball)
		if _, statErr := os.Stat(cacheDir); statErr != nil {
			if extractErr := l.extractTarball(tarball, cacheDir); extractErr != nil {
				return fmt.Errorf("sync: dep %q: extract tarball: %w", modName, extractErr)
			}
		}
		_ = writeSRISidecar(cacheDir, integ)
	}

	modules[modName] = lock.ModuleEntry{
		Source:    tarballURL,
		Integrity: integ,
		CommitSHA: commit,
	}
	result.Resolved[modName] = commit

	// Walk MODULE.meow in the extracted tarball to resolve transitive GitHub deps.
	transitiveDeps, parseErr := parseModuleMeowGitHub(filepath.Join(cacheDir, "MODULE.meow"))
	if parseErr != nil {
		// Non-fatal: a malformed MODULE.meow in a transitive dep should warn, not fail.
		fmt.Fprintf(os.Stderr, "meowctl: warning: MODULE.meow in %s@%s: %v\n", modName, commit, parseErr)
		return nil
	}
	for _, td := range transitiveDeps {
		if _, alreadyResolved := modules[td.name]; alreadyResolved {
			continue
		}
		if syncErr := l.syncGitHubDepVisited(td.name, td.source, modules, result, visited); syncErr != nil {
			return syncErr
		}
	}
	return nil
}

// resolveDepVersion picks, fetches, and caches the tarball for a single dep.
// Returns the resolved version, SRI, and source URL.
func (l *RegistryLoader) resolveDepVersion(modName, reqVersion string, entry indexEntry) (version, integ, source string, err error) {
	version = reqVersion
	if version == "" || version == "latest" {
		if len(entry.Versions) == 0 {
			return "", "", "", fmt.Errorf("sync: module %q has no versions in registry", modName)
		}
		version = entry.Versions[len(entry.Versions)-1]
	}

	source = buildSourceURL(entry.Source, modName, version)
	cacheDir := l.moduleCacheDir(modName, version)

	integ, sriErr := readSRISidecar(cacheDir)
	if sriErr != nil {
		tarball, fetchErr := l.fetchTarball(source)
		if fetchErr != nil {
			return "", "", "", fmt.Errorf("sync: fetch %s@%s: %w", modName, version, fetchErr)
		}
		// If the index carries a per-version integrity hash, verify before extracting.
		if verifyErr := checkIndexIntegrity(entry, version, tarball); verifyErr != nil {
			return "", "", "", fmt.Errorf("sync: SRI mismatch for %s@%s (index integrity): %w", modName, version, verifyErr)
		}
		integ = computeSRI(tarball)
		if _, statErr := os.Stat(cacheDir); statErr != nil {
			if extractErr := l.extractTarball(tarball, cacheDir); extractErr != nil {
				return "", "", "", fmt.Errorf("sync: extract %s@%s: %w", modName, version, extractErr)
			}
		}
		_ = writeSRISidecar(cacheDir, integ)
	}
	return version, integ, source, nil
}

// LatestVersion returns the latest available version for modName from the registry.
func (l *RegistryLoader) LatestVersion(modName string) (string, error) {
	index, err := l.fetchIndex()
	if err != nil {
		return "", err
	}
	entry, ok := index.Modules[modName]
	if !ok {
		return "", fmt.Errorf("module %q not found in registry", modName)
	}
	if len(entry.Versions) == 0 {
		return "", fmt.Errorf("module %q has no versions in registry", modName)
	}
	return entry.Versions[len(entry.Versions)-1], nil
}

// ModfileDep is a dep() entry from a modfile, used by SyncModules.
// Exactly one of Version or Source must be set.
type ModfileDep struct {
	// Name is the logical module name used in @name// load URLs (e.g. "stdlib").
	Name string
	// Version is the required registry version (e.g. "0.1.1"). Set for registry deps.
	Version string
	// Source is the GitHub source reference (e.g. "github:owner/repo@v1.2.3"). Set for GitHub deps.
	Source string
}

// ModfileReplace is a replace() entry from a modfile, used by SyncModules.
// Exactly one of Path or Source must be set.
type ModfileReplace struct {
	// Name is the logical module name to replace (matches a dep Name).
	Name string
	// Path is the local filesystem path for a local override.
	Path string
	// Source is the GitHub source reference for a remote override (e.g. "github:fork/stdlib@v1.2.4").
	Source string
}

// buildSourceURL substitutes {name} and {version} in the source URL template.
func buildSourceURL(sourceTmpl, name, version string) string {
	s := strings.ReplaceAll(sourceTmpl, "{name}", name)
	return strings.ReplaceAll(s, "{version}", version)
}

// githubSource holds the parsed components of a "github:owner/repo@ref" source string.
type githubSource struct {
	owner string
	repo  string
	ref   string
}

// parseGitHubSource decomposes "github:owner/repo@ref" into its parts.
// The @ref component is required — bare refs are rejected to avoid ambiguity.
func parseGitHubSource(s string) (githubSource, error) {
	rest, ok := strings.CutPrefix(s, "github:")
	if !ok {
		return githubSource{}, fmt.Errorf("parseGitHubSource: not a github: source: %q", s)
	}
	ownerRepo, ref, found := strings.Cut(rest, "@")
	if !found || ref == "" {
		return githubSource{}, fmt.Errorf("parseGitHubSource: missing @ref in %q (use github:owner/repo@tag-or-branch)", s)
	}
	owner, repo, found2 := strings.Cut(ownerRepo, "/")
	if !found2 || owner == "" || repo == "" {
		return githubSource{}, fmt.Errorf("parseGitHubSource: expected owner/repo in %q", s)
	}
	return githubSource{owner: owner, repo: repo, ref: ref}, nil
}

// githubTarballURLWithBase returns the GitHub archive URL using the provided base.
// If base is empty it falls back to https://github.com.
func githubTarballURLWithBase(base, owner, repo, ref string) string {
	if base == "" {
		base = "https://github.com"
	}
	return fmt.Sprintf("%s/%s/%s/archive/%s.tar.gz", base, owner, repo, ref)
}

// resolveGitHubCommit resolves a ref (tag/branch/SHA) to a full commit SHA
// using the GitHub commits API.
func (l *RegistryLoader) resolveGitHubCommit(owner, repo, ref string) (string, error) {
	apiBase := l.GitHubAPIBase
	if apiBase == "" {
		apiBase = "https://api.github.com"
	}
	apiURL := fmt.Sprintf("%s/repos/%s/%s/commits/%s", apiBase, owner, repo, ref)
	req, err := http.NewRequestWithContext(context.Background(), http.MethodGet, apiURL, nil)
	if err != nil {
		return "", fmt.Errorf("resolveGitHubCommit: build request: %w", err)
	}
	resp, err := l.client().Do(req) //nolint:gosec
	if err != nil {
		return "", fmt.Errorf("resolveGitHubCommit %s/%s@%s: %w", owner, repo, ref, err)
	}
	defer resp.Body.Close() //nolint:errcheck
	if resp.StatusCode != http.StatusOK {
		return "", fmt.Errorf("resolveGitHubCommit %s/%s@%s: HTTP %d", owner, repo, ref, resp.StatusCode)
	}
	var payload struct {
		SHA string `json:"sha"`
	}
	if err := json.NewDecoder(resp.Body).Decode(&payload); err != nil {
		return "", fmt.Errorf("resolveGitHubCommit %s/%s@%s: decode: %w", owner, repo, ref, err)
	}
	if payload.SHA == "" {
		return "", fmt.Errorf("resolveGitHubCommit %s/%s@%s: empty SHA in response", owner, repo, ref)
	}
	return payload.SHA, nil
}

// registryReqs implements mvs.Reqs for the registry loader.
// It fetches each module's MODULE.meow from its tarball to discover dependencies.
type registryReqs struct {
	loader *RegistryLoader
	index  *registryIndex
}

// Required implements mvs.Reqs. It fetches the module tarball (if not cached),
// parses MODULE.meow for dep() calls, and returns the listed dependencies.
func (r *registryReqs) Required(m mvs.Module) ([]mvs.Module, error) {
	entry, ok := r.index.Modules[m.Name]
	if !ok {
		return nil, fmt.Errorf("module %q not found in registry index", m.Name)
	}

	extractDir := r.loader.moduleCacheDir(m.Name, m.Version)
	_, statErr := os.Stat(extractDir)
	if statErr != nil {
		// Not cached yet: download and extract to read MODULE.meow.
		source := buildSourceURL(entry.Source, m.Name, m.Version)
		tarball, err := r.loader.fetchTarball(source)
		if err != nil {
			return nil, fmt.Errorf("fetch %s@%s: %w", m.Name, m.Version, err)
		}
		if err := r.loader.extractTarball(tarball, extractDir); err != nil {
			return nil, fmt.Errorf("extract %s@%s: %w", m.Name, m.Version, err)
		}
		// Write the SRI sidecar so resolveVersion can skip re-downloading this tarball.
		_ = writeSRISidecar(extractDir, computeSRI(tarball))
	}

	return parseModuleMeow(filepath.Join(extractDir, "MODULE.meow"))
}

// Max implements mvs.Reqs.
func (r *registryReqs) Max(v1, v2 string) string {
	return mvs.Max(v1, v2)
}

// githubDepEntry holds a transitive GitHub dep parsed from MODULE.meow.
type githubDepEntry struct {
	name   string
	source string
}

// parseModuleMeowGitHub evaluates a MODULE.meow file inside a GitHub dep tarball
// and returns any dep() declarations that carry a source= kwarg (i.e. transitive
// GitHub deps). Registry deps (version= only) are ignored here — they follow the
// normal MVS path and must be in the shared modfile.
// A missing MODULE.meow is not an error.
func parseModuleMeowGitHub(path string) ([]githubDepEntry, error) {
	data, err := os.ReadFile(path) // #nosec G304 -- path constructed from validated cache dir
	if errors.Is(err, os.ErrNotExist) {
		return nil, nil
	}
	if err != nil {
		return nil, fmt.Errorf("read MODULE.meow: %w", err)
	}

	var deps []githubDepEntry
	depFn := gostarlark.NewBuiltin("dep", func(_ *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (gostarlark.Value, error) {
		var name gostarlark.String
		var source gostarlark.String
		// version is allowed but optional here — we only care about source= deps.
		var version gostarlark.String
		if err := gostarlark.UnpackArgs("dep", args, kwargs, "name", &name, "version?", &version, "source?", &source); err != nil {
			return nil, err
		}
		if string(source) != "" {
			deps = append(deps, githubDepEntry{name: string(name), source: string(source)})
		}
		return gostarlark.None, nil
	})
	moduleFn := gostarlark.NewBuiltin("module", func(_ *gostarlark.Thread, _ *gostarlark.Builtin, _ gostarlark.Tuple, _ []gostarlark.Tuple) (gostarlark.Value, error) {
		return gostarlark.None, nil
	})
	predeclared := gostarlark.StringDict{"dep": depFn, "module": moduleFn}

	thread := &gostarlark.Thread{Name: "MODULE.meow"}
	opts := &syntax.FileOptions{}
	if _, err := gostarlark.ExecFileOptions(opts, thread, path, data, predeclared); err != nil {
		return nil, fmt.Errorf("eval MODULE.meow: %w", err)
	}
	return deps, nil
}

// parseModuleMeow evaluates a MODULE.meow file with only a dep() builtin
// and returns the declared dependencies. A missing MODULE.meow is not an error
// (the module has no dependencies).
func parseModuleMeow(path string) ([]mvs.Module, error) {
	data, err := os.ReadFile(path) // #nosec G304 -- path is constructed from validated cache dir
	if errors.Is(err, os.ErrNotExist) {
		return nil, nil
	}
	if err != nil {
		return nil, fmt.Errorf("read MODULE.meow: %w", err)
	}

	var deps []mvs.Module
	depFn := gostarlark.NewBuiltin("dep", func(_ *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (gostarlark.Value, error) {
		var name, version gostarlark.String
		if err := gostarlark.UnpackArgs("dep", args, kwargs, "name", &name, "version", &version); err != nil {
			return nil, err
		}
		deps = append(deps, mvs.Module{Name: string(name), Version: string(version)})
		return gostarlark.None, nil
	})
	// module() declares the module's own identity. We ignore its arguments —
	// only dep() declarations affect MVS resolution.
	moduleFn := gostarlark.NewBuiltin("module", func(_ *gostarlark.Thread, _ *gostarlark.Builtin, _ gostarlark.Tuple, _ []gostarlark.Tuple) (gostarlark.Value, error) {
		return gostarlark.None, nil
	})
	predeclared := gostarlark.StringDict{"dep": depFn, "module": moduleFn}

	thread := &gostarlark.Thread{Name: "MODULE.meow"}
	opts := &syntax.FileOptions{}
	if _, err := gostarlark.ExecFileOptions(opts, thread, path, data, predeclared); err != nil {
		return nil, fmt.Errorf("eval MODULE.meow: %w", err)
	}
	return deps, nil
}
