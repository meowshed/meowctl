package cli

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/meowshed/meowctl/internal/ctx"
	starlarkpkg "github.com/meowshed/meowctl/internal/starlark"
)

// callExtensionHook runs hooks/<componentID>.star if it exists, setting
// ctx.component_dir to <configDir>/hooks/<componentID>/ so that extension
// hooks can store companion data files alongside their .star file.
//
// baseCaps supplies all Capabilities fields; ComponentDir is overridden here.
// eval must be non-nil.
func callExtensionHook(configDir, componentID, hookName string, baseCaps *ctx.Capabilities, eval *starlarkpkg.Evaluator) error {
	extFile := filepath.Join(configDir, "hooks", componentID+".star")
	if _, err := os.Lstat(extFile); os.IsNotExist(err) {
		return nil
	}

	caps := *baseCaps // shallow copy
	caps.ComponentDir = filepath.Join(configDir, "hooks", componentID)

	ctxVal := ctx.New(&caps)
	result, err := eval.ExecFile(extFile, nil, nil, ctxVal)
	if err != nil {
		return fmt.Errorf("extension hook %s: eval: %w", componentID, err)
	}
	return eval.CallHook(result.Globals, hookName, extFile, ctxVal)
}
