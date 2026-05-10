package loader

import (
	"fmt"
	"strings"
	"sync"

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
//   - @name//   → RegistryLoader
//   - github:// → GitHubLoader
type CompositeLoader struct {
	root     string
	fileOpts *syntax.FileOptions

	// sf deduplicates concurrent load calls for the same module URL so evaluation
	// happens at most once even when multiple goroutines race to load the same module.
	sf    singleflight.Group
	mu    sync.RWMutex
	cache map[string]gostarlark.StringDict
}

// NewCompositeLoader creates a CompositeLoader. root is the dotfiles root directory,
// used by FileSystemLoader to resolve self// paths.
func NewCompositeLoader(root string, opts *syntax.FileOptions) *CompositeLoader {
	return &CompositeLoader{
		root:     root,
		fileOpts: opts,
		cache:    make(map[string]gostarlark.StringDict),
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
		case strings.HasPrefix(moduleURL, "github://"):
			sub = &GitHubLoader{}
		case len(moduleURL) > 1 && moduleURL[0] == '@' && strings.Contains(moduleURL, "//"):
			sub = &RegistryLoader{}
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
