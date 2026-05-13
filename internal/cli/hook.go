package cli

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/meowshed/meowctl/internal/ctx"
	"github.com/meowshed/meowctl/internal/lifecycle"
	"github.com/meowshed/meowctl/internal/pkg"
	starlarkpkg "github.com/meowshed/meowctl/internal/starlark"
	"github.com/meowshed/meowctl/internal/starlark/loader"
	"github.com/meowshed/meowctl/internal/state"
	"github.com/meowshed/meowctl/internal/tui"
	"github.com/spf13/cobra"
)

// validHookPhases is the set of phases that meowctl hook accepts.
var validHookPhases = []string{
	string(lifecycle.PhaseShell),
	string(lifecycle.PhaseLogin),
}

// runtimeHookCaller implements lifecycle.HookCaller for meowctl hook <phase>.
// It sets RuntimeHook=true so that ctx.emit writes to stdout instead of a file,
// and also runs any hooks/<name>.star extension files after the component hook.
type runtimeHookCaller struct {
	configDir  string
	eval       *starlarkpkg.Evaluator
	pmRegistry *pkg.PMRegistry
	loader     *loader.CompositeLoader // may be nil
}

// CallHook executes the component hook and then any hooks/<name>.star extension.
func (h *runtimeHookCaller) CallHook(componentID, hookName string) error {
	if err := h.callComponentHook(componentID, hookName); err != nil {
		return err
	}
	return h.callExtensionHook(componentID, hookName)
}

// callComponentHook runs the component's own hook with RuntimeHook=true.
func (h *runtimeHookCaller) callComponentHook(componentID, hookName string) error {
	componentFile := filepath.Join(h.configDir, "components", componentID+".star")
	if _, err := os.Lstat(componentFile); os.IsNotExist(err) {
		return nil
	}

	caps := h.buildCaps(componentID, componentFile, hookName)
	ctxVal := ctx.New(caps)
	result, err := h.eval.ExecFile(componentFile, nil, nil, ctxVal)
	if err != nil {
		return fmt.Errorf("component %s: eval: %w", componentID, err)
	}
	return h.eval.CallHook(result.Globals, hookName, componentFile, ctxVal)
}

// callExtensionHook runs hooks/<componentID>.star if it exists.
func (h *runtimeHookCaller) callExtensionHook(componentID, hookName string) error {
	extFile := filepath.Join(h.configDir, "hooks", componentID+".star")
	if _, err := os.Lstat(extFile); os.IsNotExist(err) {
		return nil
	}

	caps := h.buildCaps(componentID, extFile, hookName)
	ctxVal := ctx.New(caps)
	result, err := h.eval.ExecFile(extFile, nil, nil, ctxVal)
	if err != nil {
		return fmt.Errorf("extension hook %s: eval: %w", componentID, err)
	}
	return h.eval.CallHook(result.Globals, hookName, extFile, ctxVal)
}

// buildCaps constructs Capabilities for a runtime hook invocation.
func (h *runtimeHookCaller) buildCaps(componentID, componentFile, hookName string) *ctx.Capabilities {
	caps := &ctx.Capabilities{
		RuntimeHook:  true,
		ComponentDir: h.resolveComponentDir(componentID, componentFile),
		Phase:        hookName,
		Component:    componentID,
		PMRegistry:   h.pmRegistry,
		Env:          envMap(),
	}
	if home, err := os.UserHomeDir(); err == nil {
		caps.Home = home
		caps.StateDir = filepath.Join(home, ".local", "share", "meowctl", componentID)
	}
	return caps
}

// resolveComponentDir mirrors starlarkHookCaller.resolveComponentDir.
func (h *runtimeHookCaller) resolveComponentDir(componentID, componentFile string) string {
	if strings.Contains(componentID, "//") {
		if h.loader != nil {
			if dir, err := h.loader.ComponentDir(componentID); err == nil && dir != "" {
				return dir
			}
		}
		return ""
	}
	if h.loader != nil {
		if dir, err := h.loader.ComponentDir(componentID); err == nil && dir != "" {
			return dir
		}
	}
	return filepath.Dir(componentFile)
}

func newHookCmd(gf *globalFlags) *cobra.Command {
	return &cobra.Command{
		Use:       "hook <phase>",
		Short:     "Run runtime hooks for the given phase and emit shell code to stdout",
		Args:      cobra.ExactArgs(1),
		ValidArgs: validHookPhases,
		RunE: func(_ *cobra.Command, args []string) error {
			phase := args[0]
			var valid bool
			for _, p := range validHookPhases {
				if p == phase {
					valid = true
					break
				}
			}
			if !valid {
				return exitErrorf(ExitUsage, "unsupported phase %q (supported: %v)", phase, validHookPhases)
			}

			configDir, err := resolveConfigDir(gf)
			if err != nil {
				return err
			}

			order, pmReg, err := loadComponentsWithDeps(configDir, nil)
			if err != nil {
				return err
			}

			caller := &runtimeHookCaller{
				configDir:  configDir,
				eval:       &starlarkpkg.Evaluator{},
				pmRegistry: pmReg,
			}

			w := tui.New(os.Stdout)
			defer func() { _ = w.Close() }()

			statePath := filepath.Join(configDir, "state.toml")
			sentinel := state.NewManager(statePath)

			runner := &lifecycle.Runner{
				Order:      order,
				Caller:     caller,
				Stack:      nil,
				Sentinel:   sentinel,
				NoRollback: true,
				Writer:     w,
			}
			return runner.RunPhaseSet("hook:"+phase, []lifecycle.Phase{lifecycle.Phase(phase)})
		},
	}
}
