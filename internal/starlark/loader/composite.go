package loader

import (
	"fmt"
	"net/http"
	"path/filepath"
	"strings"
	"sync"
	"time"

	"golang.org/x/sync/singleflight"

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
	fileOpts    *syntax.FileOptions
	client      *http.Client

	// githubAPIBase and githubRawBase override the default GitHub endpoints.
	// Empty string means production defaults are used; overridden in tests via NewCompositeLoaderForTest.
	githubAPIBase string
	githubRawBase string

	// sf deduplicates concurrent load calls for the same module URL so evaluation
	// happens at most once even when multiple goroutines race to load the same module.
	sf    singleflight.Group
	mu    sync.RWMutex
	cache map[string]gostarlark.StringDict
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
		root:        root,
		userRoot:    opts.UserRoot,
		cacheDir:    opts.CacheDir,
		lockPath:    opts.LockPath,
		registryURL: opts.RegistryURL,
		fileOpts:    fileOpts,
		client:      client,
		cache:       make(map[string]gostarlark.StringDict),
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
		case len(moduleURL) > 1 && moduleURL[0] == '@' && strings.Contains(moduleURL, "//"):
			sub = &RegistryLoader{
				RegistryURL: c.registryURL,
				CacheDir:    filepath.Join(c.cacheDir, "modules"),
				LockPath:    c.lockPath,
				Client:      c.client,
				FileOpts:    c.fileOpts,
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
		c.mu.Unlock()
		return globals, nil
	})
	if err != nil {
		return nil, err
	}
	return v.(gostarlark.StringDict), nil
}
