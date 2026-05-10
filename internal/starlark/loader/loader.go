// Package loader provides Starlark module loaders for meowctl.
package loader

import (
	gostarlark "go.starlark.net/starlark"
)

// Loader resolves and loads Starlark modules by URL.
// It is called by the Starlark thread's Load hook.
type Loader interface {
	// Load retrieves the module identified by moduleURL and returns its exported
	// symbols. predeclared contains the builtins that should be available during
	// evaluation of the loaded module, ensuring consistency across the evaluation
	// tree. thread is the parent evaluation thread.
	Load(thread *gostarlark.Thread, moduleURL string, predeclared gostarlark.StringDict) (gostarlark.StringDict, error)
}
