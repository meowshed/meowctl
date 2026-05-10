package loader

import (
	"errors"
	"fmt"

	gostarlark "go.starlark.net/starlark"
)

// errRegistryNotImplemented is returned by RegistryLoader until full implementation ships.
var errRegistryNotImplemented = errors.New("RegistryLoader: not implemented")

// RegistryLoader will load Starlark modules from the meowctl module registry.
// It handles the @name// URL scheme.
// This stub returns an error at runtime so scheme dispatch compiles and fails clearly.
type RegistryLoader struct{}

// Load implements Loader. Always returns errRegistryNotImplemented.
func (l *RegistryLoader) Load(_ *gostarlark.Thread, moduleURL string, _ gostarlark.StringDict) (gostarlark.StringDict, error) {
	return nil, fmt.Errorf("%w: %s", errRegistryNotImplemented, moduleURL)
}
