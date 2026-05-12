package loader

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	gostarlark "go.starlark.net/starlark"
	"go.starlark.net/syntax"
)

const (
	selfScheme = "self//"
	userScheme = "user://"
)

// FileSystemLoader loads Starlark modules referenced with the self// or user:// scheme.
// The scheme resolves paths relative to a root directory.
//
// Example (self//): self//lib/helpers.star → <root>/lib/helpers.star
// Example (user://): user://modules/helpers.star → <root>/modules/helpers.star
type FileSystemLoader struct {
	Root     string
	scheme   string // "self//" or "user://"
	fileOpts *syntax.FileOptions
}

// NewUserLoader creates a FileSystemLoader rooted at configDir that handles
// the user:// scheme. user:// paths resolve relative to the user's meowctl
// config directory (typically ~/.config/meowctl).
//
// Example: user://modules/helpers.star → <configDir>/modules/helpers.star
func NewUserLoader(configDir string, opts *syntax.FileOptions) *FileSystemLoader {
	return &FileSystemLoader{Root: configDir, fileOpts: opts, scheme: userScheme}
}

// NewFileSystemLoader creates a FileSystemLoader rooted at root.
func NewFileSystemLoader(root string, opts *syntax.FileOptions) *FileSystemLoader {
	return &FileSystemLoader{Root: root, fileOpts: opts, scheme: selfScheme}
}

// Load implements Loader. moduleURL must start with the loader's scheme ("self//" or "user://").
func (l *FileSystemLoader) Load(thread *gostarlark.Thread, moduleURL string, predeclared gostarlark.StringDict) (gostarlark.StringDict, error) {
	if !strings.HasPrefix(moduleURL, l.scheme) {
		return nil, fmt.Errorf("FileSystemLoader: unsupported scheme in %q", moduleURL)
	}
	rel := strings.TrimPrefix(moduleURL, l.scheme)
	abs := filepath.Join(l.Root, filepath.FromSlash(rel))

	// Reject paths that escape the root (e.g. self//../../../etc/passwd).
	cleanRoot := filepath.Clean(l.Root) + string(os.PathSeparator)
	if !strings.HasPrefix(abs+string(os.PathSeparator), cleanRoot) {
		return nil, fmt.Errorf("FileSystemLoader: module path %q escapes root %q", moduleURL, l.Root)
	}

	src, err := os.ReadFile(abs) // #nosec G304 -- path verified to be within l.Root by HasPrefix check above
	if err != nil {
		return nil, fmt.Errorf("FileSystemLoader: cannot read %q: %w", abs, err)
	}

	childThread := &gostarlark.Thread{
		Name: moduleURL,
		Load: thread.Load,
	}
	globals, err := gostarlark.ExecFileOptions(l.fileOpts, childThread, abs, src, predeclared)
	if err != nil {
		return nil, fmt.Errorf("FileSystemLoader: eval %q: %w", abs, err)
	}
	return globals, nil
}
