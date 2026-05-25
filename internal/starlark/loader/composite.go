package loader

import (
	"fmt"
	"net/http"
	"path/filepath"
	"strings"
	"sync"
	"time"

	"golang.org/x/sync/singleflight"

	"github.com/meowshed/meowctl/internal/lock"
	gostarlark "go.starlark.net/starlark"
	"go.starlark.net/syntax"
)

// CompositeLoader dispatches load() calls to the appropriate sub-loader by URL scheme.
// It also caches module globals so each module is evaluated at most once per
// CompositeLoader instance.
//
// Supported schemes:
//   - self//    → FileSystemLoader (local dotfiles)
//   - user://   → FileSystemLoader rooted at the user config directory
//   - @name//   → RegistryLoader
//   - github:// → GitHubLoader
type CompositeLoader struct {
	root        string
	userRoot    string
	cacheDir    string
	lockPath    string
	registryURL string
	replaces    map[string]string // module name → local path override
	fileOpts    *syntax.FileOptions
	client      *http.Client

	// githubAPIBase and githubRawBase override the default GitHub endpoints.
	// Empty string means production defaults are used; overridden in tests via NewCompositeLoaderForTest.
	githubAPIBase string
	githubRawBase string

	// sf deduplicates concurrent load calls for the same module URL so evaluation
	// happens at most once even when multiple goroutines race to load the same module.
	sf            singleflight.Group
	mu            sync.RWMutex
	cache         map[string]gostarlark.StringDict
	resolvedPaths map[string]string // moduleURL → resolved filesystem directory
}

// CompositeLoaderOptions holds optional configuration for NewCompositeLoader.
type CompositeLoaderOptions struct {
	// UserRoot is the directory for user:// scheme resolution (typically ~/.config/meowctl).
	UserRoot string
	// CacheDir is the directory for downloaded module caches (typically ~/.cache/meowctl).
	CacheDir string
	// RegistryURL overrides the default meowshed registry URL for @name// scheme.
	// If empty, the default is used.
	RegistryURL string
	// LockPath is the path to the meowctl lock file.
	LockPath string
	// Client overrides the default HTTP client. Useful for tests.
	Client *http.Client
	// Replaces maps registry module names to local filesystem paths.
	// When set, @name// loads are served from the local path instead of the registry.
	// This is equivalent to a per-invocation replace() directive.
	// Example: {"stdlib": "/path/to/meowctl-stdlib"}
	Replaces map[string]string
}

// NewCompositeLoader creates a CompositeLoader. root is the dotfiles root directory,
// used by FileSystemLoader to resolve self// paths. opts provides optional config
// for the remote loaders; zero-value opts disables remote loading (github:// and @name//
// will return errors until their dependencies are configured).
func NewCompositeLoader(root string, fileOpts *syntax.FileOptions, opts CompositeLoaderOptions) *CompositeLoader {
	client := opts.Client
	if client == nil {
		client = &http.Client{Timeout: 30 * time.Second}
	}
	return &CompositeLoader{
		root:          root,
		userRoot:      opts.UserRoot,
		cacheDir:      opts.CacheDir,
		lockPath:      opts.LockPath,
		registryURL:   opts.RegistryURL,
		replaces:      opts.Replaces,
		fileOpts:      fileOpts,
		client:        client,
		cache:         make(map[string]gostarlark.StringDict),
		resolvedPaths: make(map[string]string),
	}
}

// Load implements the gostarlark.Thread.Load contract:
// func(thread *Thread, module string) (StringDict, error).
// It dispatches by URL scheme and caches results.
// predeclared is passed through to sub-loaders so builtins are available in loaded modules.
func (c *CompositeLoader) Load(thread *gostarlark.Thread, moduleURL string, predeclared gostarlark.StringDict) (gostarlark.StringDict, error) {
	// Fast path: return from cache without taking an exclusive lock.
	c.mu.RLock()
	if cached, ok := c.cache[moduleURL]; ok {
		c.mu.RUnlock()
		return cached, nil
	}
	c.mu.RUnlock()

	// Slow path: use singleflight so concurrent callers for the same URL share one evaluation.
	v, err, _ := c.sf.Do(moduleURL, func() (any, error) {
		var sub Loader
		switch {
		case strings.HasPrefix(moduleURL, selfScheme):
			sub = NewFileSystemLoader(c.root, c.fileOpts)
		case strings.HasPrefix(moduleURL, userScheme):
			if c.userRoot == "" {
				return nil, fmt.Errorf("CompositeLoader: user:// requested but UserRoot not configured")
			}
			sub = NewUserLoader(c.userRoot, c.fileOpts)
		case strings.HasPrefix(moduleURL, githubScheme):
			gh := newGitHubLoader(c.cacheDir, c.lockPath, c.client, c.fileOpts)
			if c.githubAPIBase != "" {
				gh.apiBase = c.githubAPIBase
			}
			if c.githubRawBase != "" {
				gh.rawBase = c.githubRawBase
			}
			sub = gh
		case len(moduleURL) > 1 && moduleURL[0] == '@':
			sub = &RegistryLoader{
				RegistryURL:   c.registryURL,
				CacheDir:      filepath.Join(c.cacheDir, "modules"),
				LockPath:      c.lockPath,
				Client:        c.client,
				FileOpts:      c.fileOpts,
				Replaces:      c.replaces,
				GitHubAPIBase: c.githubAPIBase,
			}
		default:
			return nil, fmt.Errorf("CompositeLoader: unknown scheme in module URL %q", moduleURL)
		}
		globals, err := sub.Load(thread, moduleURL, predeclared)
		if err != nil {
			return nil, err
		}
		c.mu.Lock()
		c.cache[moduleURL] = globals
		if dir := c.resolveDir(moduleURL); dir != "" {
			c.resolvedPaths[moduleURL] = dir
		}
		c.mu.Unlock()
		return globals, nil
	})
	if err != nil {
		return nil, err
	}
	return v.(gostarlark.StringDict), nil
}

// ComponentDir returns the resolved filesystem directory for a previously-loaded module URL.
// It returns an error if the module has not been loaded yet or if the URL scheme is unsupported.
// For bare-name local components that are not loaded via CompositeLoader, callers should fall
// back to filepath.Dir(componentFile).
func (c *CompositeLoader) ComponentDir(moduleURL string) (string, error) {
	c.mu.RLock()
	dir, ok := c.resolvedPaths[moduleURL]
	c.mu.RUnlock()
	if ok {
		return dir, nil
	}
	// Not yet loaded — compute from URL without evaluating.
	dir = c.resolveDir(moduleURL)
	if dir == "" {
		return "", fmt.Errorf("CompositeLoader: cannot resolve directory for %q (not loaded or unknown scheme)", moduleURL)
	}
	return dir, nil
}

// resolveDir computes the filesystem directory for a module URL without evaluation.
// Returns "" for unknown or unsupported schemes.
func (c *CompositeLoader) resolveDir(moduleURL string) string {
	switch {
	case strings.HasPrefix(moduleURL, selfScheme):
		rel := strings.TrimPrefix(moduleURL, selfScheme)
		abs := filepath.Join(c.root, filepath.FromSlash(rel))
		return filepath.Dir(abs)
	case strings.HasPrefix(moduleURL, userScheme):
		rel := strings.TrimPrefix(moduleURL, userScheme)
		abs := filepath.Join(c.userRoot, filepath.FromSlash(rel))
		return filepath.Dir(abs)
	case strings.HasPrefix(moduleURL, githubScheme):
		return c.resolveGitHubDir(moduleURL)
	case len(moduleURL) > 1 && moduleURL[0] == '@':
		return c.resolveRegistryDir(moduleURL)
	}
	return ""
}

// resolveGitHubDir resolves the directory for a github:// URL.
func (c *CompositeLoader) resolveGitHubDir(moduleURL string) string {
	u, err := parseGitHubURL(moduleURL)
	if err != nil {
		return ""
	}
	commit := u.ref
	if c.lockPath != "" {
		if lf, readErr := readLockForGitHub(c.lockPath, u.owner, u.repo); readErr == nil {
			commit = lf
		}
	}
	cachePath := filepath.Join(c.cacheDir, "github", u.owner, u.repo, commit, filepath.FromSlash(u.path))
	return filepath.Dir(cachePath)
}

// resolveRegistryDir resolves the directory for an @name or @name//path URL.
func (c *CompositeLoader) resolveRegistryDir(moduleURL string) string {
	withoutAt := moduleURL[1:]
	if !strings.Contains(withoutAt, "//") {
		return c.resolveRegistryRoot(withoutAt)
	}
	return c.resolveRegistryPath(withoutAt)
}

// resolveRegistryRoot resolves the root directory for an @name URL.
func (c *CompositeLoader) resolveRegistryRoot(name string) string {
	if localRoot, ok := c.replaces[name]; ok {
		return localRoot
	}
	return filepath.Join(c.cacheDir, "modules", name, c.lookupVersion(name))
}

// resolveRegistryPath resolves the directory for an @name//path URL.
// If the path has no extension it is treated as a component directory
// (e.g. components/fish-config → …/components/fish-config).
// If it has an extension it is treated as a file and its parent is returned.
func (c *CompositeLoader) resolveRegistryPath(withoutAt string) string {
	parts := strings.SplitN(withoutAt, "//", 2)
	if len(parts) != 2 {
		return ""
	}
	name := parts[0]
	filePath := parts[1]
	if localRoot, ok := c.replaces[name]; ok {
		abs := filepath.Join(localRoot, filepath.FromSlash(filePath))
		return filepath.Dir(abs)
	}
	abs := filepath.Join(c.cacheDir, "modules", name, c.lookupVersion(name), filepath.FromSlash(filePath))
	// Path without extension is a component directory (init.star lives inside).
	if filepath.Ext(filePath) == "" {
		return abs
	}
	return filepath.Dir(abs)
}

// lookupVersion returns the locked version for a registry module, or "latest".
func (c *CompositeLoader) lookupVersion(name string) string {
	if c.lockPath != "" {
		if v, err := readLockForRegistry(c.lockPath, name); err == nil && v != "" {
			return v
		}
	}
	return "latest"
}

// readLockForGitHub reads the lock file and returns the pinned commit SHA for a GitHub file
// identified by owner/repo and ref. Returns an error if the lock file is missing, unreadable,
// or has no entry for the given repo+ref combination.
func readLockForGitHub(lockPath, owner, repo string) (string, error) {
	lf, err := lock.Read(lockPath)
	if err != nil {
		return "", err
	}
	// The lock key format is owner/repo@ref//path; we look for any entry matching owner/repo.
	prefix := owner + "/" + repo + "@"
	for key, entry := range lf.GitHub {
		if strings.HasPrefix(key, prefix) && entry.Commit != "" {
			return entry.Commit, nil
		}
	}
	return "", fmt.Errorf("no lock entry for %s/%s", owner, repo)
}

// readLockForRegistry reads the lock file and returns the resolved version for a registry module.
func readLockForRegistry(lockPath, name string) (string, error) {
	lf, err := lock.Read(lockPath)
	if err != nil {
		return "", err
	}
	if entry, ok := lf.Modules[name]; ok && entry.Version != "" {
		return entry.Version, nil
	}
	return "", fmt.Errorf("no lock entry for module %s", name)
}
