package loader

import (
	"errors"
	"fmt"

	gostarlark "go.starlark.net/starlark"
)

// errGitHubNotImplemented is returned by GitHubLoader until full implementation ships.
var errGitHubNotImplemented = errors.New("GitHubLoader: not implemented")

// GitHubLoader will load Starlark modules from GitHub-hosted meowctl module tarballs.
// It handles the github:// URL scheme.
// This stub returns an error at runtime so scheme dispatch compiles and fails clearly.
type GitHubLoader struct{}

// Load implements Loader. Always returns errGitHubNotImplemented.
func (l *GitHubLoader) Load(_ *gostarlark.Thread, moduleURL string, _ gostarlark.StringDict) (gostarlark.StringDict, error) {
	return nil, fmt.Errorf("%w: %s", errGitHubNotImplemented, moduleURL)
}
