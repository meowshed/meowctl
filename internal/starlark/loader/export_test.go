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

// NewRegistryLoaderForTest creates a CompositeLoader with overridden GitHub API,
// raw content, and registry index base URLs. Exported for use in external tests only.
func NewRegistryLoaderForTest(root string, fileOpts *syntax.FileOptions, opts CompositeLoaderOptions, apiBase, rawBase, registryBase string) *CompositeLoader {
	cl := NewCompositeLoader(root, fileOpts, opts)
	cl.githubAPIBase = apiBase
	cl.githubRawBase = rawBase
	cl.registryURL = registryBase
	return cl
}

// ParseGitHubSourceForTest exposes parseGitHubSource for external tests.
func ParseGitHubSourceForTest(s string) (owner, repo, ref string, err error) {
	gs, err := parseGitHubSource(s)
	return gs.owner, gs.repo, gs.ref, err
}
