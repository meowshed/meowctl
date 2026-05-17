package cli

import (
	"fmt"
	"os"
	"path/filepath"
	"time"

	"github.com/meowshed/meowctl/internal/ctx"
	"github.com/meowshed/meowctl/internal/lifecycle"
	"github.com/meowshed/meowctl/internal/pkg"
	starlarkpkg "github.com/meowshed/meowctl/internal/starlark"
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
}

// CallHook executes the component hook and then any hooks/<name>.star extension.
func (h *runtimeHookCaller) CallHook(componentID, hookName string) error {
	if err := h.callComponentHook(componentID, hookName); err != nil {
		return err
	}
	caps := h.buildCaps(componentID, hookName)
	return callExtensionHook(h.configDir, componentID, hookName, caps, h.eval)
}

// callComponentHook runs the component's own hook with RuntimeHook=true.
func (h *runtimeHookCaller) callComponentHook(componentID, hookName string) error {
	componentFile := filepath.Join(h.configDir, "components", componentID+".star")
	if _, err := os.Lstat(componentFile); os.IsNotExist(err) {
		return nil
	}

	caps := h.buildCaps(componentID, hookName)
	caps.ComponentDir = filepath.Dir(componentFile)
	ctxVal := ctx.New(caps)
	result, err := h.eval.ExecFile(componentFile, nil, nil, ctxVal)
	if err != nil {
		return fmt.Errorf("component %s: eval: %w", componentID, err)
	}
	return h.eval.CallHook(result.Globals, hookName, componentFile, ctxVal)
}

// buildCaps constructs base Capabilities for a runtime hook invocation.
// ComponentDir is not set here; callers override it as appropriate.
func (h *runtimeHookCaller) buildCaps(componentID, hookName string) *ctx.Capabilities {
	caps := &ctx.Capabilities{
		RuntimeHook: true,
		Phase:       hookName,
		Component:   componentID,
		PMRegistry:  h.pmRegistry,
		Env:         envMap(),
	}
	if home, err := os.UserHomeDir(); err == nil {
		caps.Home = home
		caps.StateDir = filepath.Join(home, ".local", "share", "meowctl", componentID)
	}
	return caps
}

// writeHookError writes a .hook-error flag file containing the timestamp and error message.
func writeHookError(configDir string, runErr error) {
	path := filepath.Join(configDir, configHookErrorFile)
	content := fmt.Sprintf("%s\n%s\n", time.Now().UTC().Format(time.RFC3339), runErr.Error())
	_ = os.WriteFile(path, []byte(content), 0o600) // #nosec G304
}

// clearHookError removes the .hook-error flag file if it exists.
func clearHookError(configDir string) {
	path := filepath.Join(configDir, configHookErrorFile)
	_ = os.Remove(path) // ignore error if file doesn't exist
}

// hookErrorExists returns true if a .hook-error flag file is present.
func hookErrorExists(configDir string) bool {
	_, err := os.Lstat(filepath.Join(configDir, configHookErrorFile))
	return err == nil
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

			order, pmReg, _, err := loadComponentsWithDeps(configDir, nil, true)
			if err != nil {
				// Eval failure: write flag file, exit 0 (shell startup must never break).
				writeHookError(configDir, err)
				return nil
			}

			caller := &runtimeHookCaller{
				configDir:  configDir,
				eval:       &starlarkpkg.Evaluator{},
				pmRegistry: pmReg,
			}

			w := tui.New(os.Stdout)
			defer func() { _ = w.Close() }()

			statePath := filepath.Join(configDir, configStateFile)
			sentinel := state.NewManager(statePath)

			runner := &lifecycle.Runner{
				Order:      order,
				Caller:     caller,
				Stack:      nil,
				Sentinel:   sentinel,
				NoRollback: true,
				Writer:     w,
			}
			runErr := runner.RunPhaseSet("hook:"+phase, []lifecycle.Phase{lifecycle.Phase(phase)})
			if runErr != nil {
				// Hook eval failure: write flag file, exit 0.
				writeHookError(configDir, runErr)
				return nil
			}
			clearHookError(configDir)
			return nil
		},
	}
}
