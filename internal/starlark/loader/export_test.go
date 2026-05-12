package loader

import "go.starlark.net/syntax"

// NewCompositeLoaderForTest creates a CompositeLoader with overridden GitHub API
// and raw content base URLs. Exported for use in external tests only.
func NewCompositeLoaderForTest(root string, fileOpts *syntax.FileOptions, opts CompositeLoaderOptions, apiBase, rawBase string) *CompositeLoader {
	cl := NewCompositeLoader(root, fileOpts, opts)
	cl.githubAPIBase = apiBase
	cl.githubRawBase = rawBase
	return cl
}
