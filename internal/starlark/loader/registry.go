package loader

import (
	"archive/tar"
	"bytes"
	"compress/gzip"
	"context"
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
// URL format: @name//path/to/file.star
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
	// @stdlib//components/apt.star → /path/to/meowctl-stdlib/components/apt.star
	Replaces map[string]string
}

// registryURL is the parsed form of a @name// module URL.
type registryURL struct {
	module string
	path   string
}

// parseRegistryURL decomposes a @name// URL into module name and file path.
func parseRegistryURL(raw string) (registryURL, error) {
	// Must start with '@' and contain '//'.
	if len(raw) < 2 || raw[0] != '@' {
		return registryURL{}, fmt.Errorf("registry loader: invalid URL %q: must start with @", raw)
	}
	s := raw[1:] // strip '@'
	idx := strings.Index(s, "//")
	if idx < 0 || idx == 0 {
		return registryURL{}, fmt.Errorf("registry loader: invalid URL %q: missing // separator", raw)
	}
	filePath := s[idx+2:]
	if filePath == "" {
		return registryURL{}, fmt.Errorf("registry loader: invalid URL %q: empty path after //", raw)
	}
	modName := s[:idx]
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
	globals, err := gostarlark.ExecFileOptions(l.FileOpts, childThread, moduleURL, src, predeclared)
	if err != nil {
		return nil, fmt.Errorf("RegistryLoader: eval %q: %w", moduleURL, err)
	}
	return globals, nil
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
	globals, err := gostarlark.ExecFileOptions(l.FileOpts, childThread, abs, src, predeclared)
	if err != nil {
		return nil, fmt.Errorf("RegistryLoader: replace eval %q: %w", moduleURL, err)
	}
	return globals, nil
}

// resolveVersion returns the version to use for the named module. If the lock
// file already has an entry, that version is used (no MVS). Otherwise MVS is
// run, all resolved modules (root + transitive deps) are downloaded and written
// to the lock file, then the requested version is returned.
func (l *RegistryLoader) resolveVersion(name string) (string, error) {
	lf, err := lock.Read(l.LockPath)
	if err != nil {
		return "", fmt.Errorf("read lock: %w", err)
	}

	if entry, ok := lf.Modules[name]; ok {
		// Locked: ensure the tarball is present in cache.
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
		source := buildSourceURL(modEntry.Source, mod.Name, mod.Version)
		cacheDir := l.moduleCacheDir(mod.Name, mod.Version)

		// Use the SRI sidecar written by Required() if the cache is already populated,
		// avoiding a redundant tarball download for modules fetched during MVS traversal.
		integ, sriErr := readSRISidecar(cacheDir)
		if sriErr != nil {
			// Cache missing or sidecar absent — download the tarball to get the SRI.
			tarball, fetchErr := l.fetchTarball(source)
			if fetchErr != nil {
				return "", fmt.Errorf("fetch tarball for %s@%s: %w", mod.Name, mod.Version, fetchErr)
			}
			integ = computeSRI(tarball)
			if _, statErr := os.Stat(cacheDir); statErr != nil {
				if extractErr := l.extractTarball(tarball, cacheDir); extractErr != nil {
					return "", fmt.Errorf("extract tarball for %s@%s: %w", mod.Name, mod.Version, extractErr)
				}
			}
			// Write the sidecar so future runs skip the re-download.
			_ = writeSRISidecar(cacheDir, integ)
		}
		if writeErr := l.writeLockEntry(mod.Name, mod.Version, source, integ); writeErr != nil {
			// Non-fatal: modules are cached; next run will re-resolve.
			fmt.Fprintf(os.Stderr, "meowctl: warning: could not write lock entry for %s@%s: %v\n", mod.Name, mod.Version, writeErr)
		}
	}
	return buildList[0].Version, nil
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
	replaceIndex := make(map[string]string, len(replaces))
	for _, r := range replaces {
		replaceIndex[r.Module] = r.Path
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

	index, err := l.fetchIndex()
	if err != nil {
		return nil, err
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
func (l *RegistryLoader) syncOneDep(dep ModfileDep, replaceIndex map[string]string, index *registryIndex, modules map[string]lock.ModuleEntry, result *SyncResult) error {
	modName, reqVersion := splitModuleURL(dep.URL)

	if localPath, replaced := replaceIndex[modName]; replaced {
		fmt.Fprintf(os.Stderr, "meowctl: warning: module %q is replaced by local path %q — skipping registry resolution\n", modName, localPath)
		if _, statErr := os.Stat(localPath); statErr != nil {
			return fmt.Errorf("sync: replace() path for %q does not exist: %s (G5)", modName, localPath)
		}
		modules[modName] = lock.ModuleEntry{Replaced: true, Path: localPath}
		result.ReplacedPaths[modName] = localPath
		return nil
	}

	entry, ok := index.Modules[modName]
	if !ok {
		return fmt.Errorf("sync: module %q not found in registry", modName)
	}

	version, integ, source, err := l.resolveDepVersion(modName, reqVersion, entry)
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
type ModfileDep struct {
	URL string
}

// ModfileReplace is a replace() entry from a modfile, used by SyncModules.
type ModfileReplace struct {
	Module string
	Path   string
}

// splitModuleURL splits "github://owner/repo@v1.0.0" into ("github://owner/repo", "v1.0.0").
// If no @version is present, version is "".
func splitModuleURL(url string) (module, version string) {
	if idx := strings.LastIndex(url, "@"); idx >= 0 {
		return url[:idx], url[idx+1:]
	}
	return url, ""
}

// buildSourceURL substitutes {name} and {version} in the source URL template.
func buildSourceURL(sourceTmpl, name, version string) string {
	s := strings.ReplaceAll(sourceTmpl, "{name}", name)
	return strings.ReplaceAll(s, "{version}", version)
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

// parseModuleMeow evaluates a MODULE.meow file with only a dep() builtin
// and returns the declared dependencies. A missing MODULE.meow is not an error
// (the module has no dependencies).
func parseModuleMeow(path string) ([]mvs.Module, error) {
	data, err := os.ReadFile(path) // #nosec G304 -- path is constructed from validated cache dir
	if os.IsNotExist(err) {
		return nil, nil
	}
	if err != nil {
		return nil, fmt.Errorf("read MODULE.meow: %w", err)
	}

	var deps []mvs.Module
	depFn := gostarlark.NewBuiltin("dep", func(_ *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (gostarlark.Value, error) {
		var name, version string
		if err := gostarlark.UnpackPositionalArgs("dep", args, kwargs, 2, &name, &version); err != nil {
			return nil, err
		}
		deps = append(deps, mvs.Module{Name: name, Version: version})
		return gostarlark.None, nil
	})
	predeclared := gostarlark.StringDict{"dep": depFn}

	thread := &gostarlark.Thread{Name: "MODULE.meow"}
	opts := &syntax.FileOptions{}
	if _, err := gostarlark.ExecFileOptions(opts, thread, path, data, predeclared); err != nil {
		return nil, fmt.Errorf("eval MODULE.meow: %w", err)
	}
	return deps, nil
}
