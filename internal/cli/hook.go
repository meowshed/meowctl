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
	"github.com/meowshed/meowctl/internal/starlark/loader"
	"github.com/meowshed/meowctl/internal/tui"

	"github.com/spf13/cobra"
	gostarlark "go.starlark.net/starlark"
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
	configDir     string
	eval          *starlarkpkg.Evaluator
	pmRegistry    *pkg.PMRegistry
	loader        *loader.CompositeLoader
	urlComponents map[string]string // logical name → original URL for URL-named components
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
		// No local file — check if this is a URL-named component.
		if h.loader != nil {
			if origURL, ok := h.urlComponents[componentID]; ok {
				return h.callComponentHookFromURL(componentID, origURL, hookName)
			}
		}
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

// callComponentHookFromURL loads a URL-named component via the CompositeLoader
// and dispatches its shell/login hook.
func (h *runtimeHookCaller) callComponentHookFromURL(componentID, moduleURL, hookName string) error {
	var componentDir string
	if dir, err := h.loader.ComponentDir(moduleURL); err == nil {
		componentDir = dir
	}

	caps := h.buildCaps(componentID, hookName)
	caps.ComponentDir = componentDir
	ctxVal := ctx.New(caps)

	acc := &starlarkpkg.Accumulator{}
	predeclared := h.eval.StdPredeclared()
	predeclared["ctx"] = ctxVal
	thread := &gostarlark.Thread{
		Name: moduleURL,
		Load: func(t *gostarlark.Thread, module string) (gostarlark.StringDict, error) {
			return h.loader.Load(t, module, predeclared)
		},
	}
	thread.SetLocal("acc", acc)
	globals, err := h.loader.Load(thread, moduleURL, predeclared)
	if err != nil {
		return fmt.Errorf("component %s: load URL: %w", componentID, err)
	}
	if err := h.eval.CallHook(globals, hookName, moduleURL, ctxVal); err != nil {
		return fmt.Errorf("component %s hook %s: %w", componentID, hookName, err)
	}
	return nil
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
	// Detect the current shell from $SHELL (login shell) for ctx.shell.
	if shellPath := os.Getenv("SHELL"); shellPath != "" {
		caps.Shell = filepath.Base(shellPath)
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

			order, pmReg, urlMap, err := loadComponentsWithDeps(configDir, nil, true)
			if err != nil {
				// Eval failure: write flag file, exit 0 (shell startup must never break).
				writeHookError(configDir, err)
				return nil
			}

			// Build a CompositeLoader for resolving URL-named components (e.g. @stdlib//...).
			var cl *loader.CompositeLoader
			if home, homeErr := os.UserHomeDir(); homeErr == nil {
				cl = loader.NewCompositeLoader(configDir, nil, loader.CompositeLoaderOptions{
					CacheDir: filepath.Join(home, ".cache", "meowctl"),
					LockPath: filepath.Join(configDir, configLockFile),
					Replaces: readModfileReplaces(configDir),
				})
			}

			caller := &runtimeHookCaller{
				configDir: configDir,
				eval: &starlarkpkg.Evaluator{
					Platform: starlarkpkg.PlatformInfo{
						OS: goosToPlatformOS(),
					},
				},
				pmRegistry:    pmReg,
				loader:        cl,
				urlComponents: urlMap,
			}

			w := tui.NewSilentWriter()
			defer func() { _ = w.Close() }()

			runner := &lifecycle.Runner{
				Order:      order,
				Caller:     caller,
				Stack:      nil,
				Sentinel:   nil,
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
