package loader

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	gostarlark "go.starlark.net/starlark"
	"go.starlark.net/syntax"
)

const selfScheme = "self//"

// FileSystemLoader loads Starlark modules referenced with the self// scheme.
// The scheme resolves paths relative to a root directory (typically the dotfiles root).
//
// Example: self//lib/helpers.star → <root>/lib/helpers.star
type FileSystemLoader struct {
	Root     string
	fileOpts *syntax.FileOptions
}

// NewFileSystemLoader creates a FileSystemLoader rooted at root.
func NewFileSystemLoader(root string, opts *syntax.FileOptions) *FileSystemLoader {
	return &FileSystemLoader{Root: root, fileOpts: opts}
}

// Load implements Loader. moduleURL must start with "self//".
func (l *FileSystemLoader) Load(thread *gostarlark.Thread, moduleURL string, predeclared gostarlark.StringDict) (gostarlark.StringDict, error) {
	if !strings.HasPrefix(moduleURL, selfScheme) {
		return nil, fmt.Errorf("FileSystemLoader: unsupported scheme in %q", moduleURL)
	}
	rel := strings.TrimPrefix(moduleURL, selfScheme)
	abs := filepath.Join(l.Root, filepath.FromSlash(rel))

	// Reject paths that escape the root (e.g. self//../../../etc/passwd).
	cleanRoot := filepath.Clean(l.Root) + string(os.PathSeparator)
	if !strings.HasPrefix(abs+string(os.PathSeparator), cleanRoot) {
		return nil, fmt.Errorf("FileSystemLoader: module path %q escapes root %q", moduleURL, l.Root)
	}

	src, err := os.ReadFile(abs) //nolint:gosec // abs is verified to be within l.Root above
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
