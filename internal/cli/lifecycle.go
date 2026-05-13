package cli

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/meowshed/meowctl/internal/ctx"
	"github.com/meowshed/meowctl/internal/lifecycle"
	"github.com/meowshed/meowctl/internal/rollback"
	starlarkpkg "github.com/meowshed/meowctl/internal/starlark"
	"github.com/meowshed/meowctl/internal/state"
	"github.com/spf13/cobra"
)

// runConfig holds runtime parameters shared across lifecycle commands.
type runConfig struct {
	ConfigDir  string
	DryRun     bool
	NoRollback bool
	Force      bool
	IgnoreLock bool
}

// addLifecycleFlags attaches --dry-run and --no-rollback flags to cmd.
func addLifecycleFlags(cmd *cobra.Command, cfg *runConfig) {
	cmd.Flags().BoolVarP(&cfg.DryRun, "dry-run", "n", false, "Print what would be done without executing")
	cmd.Flags().BoolVar(&cfg.NoRollback, "no-rollback", false, "Skip automatic rollback on failure")
}

// addInstallFlags attaches --force and --ignore-lock flags to install/update commands.
func addInstallFlags(cmd *cobra.Command, cfg *runConfig) {
	cmd.Flags().BoolVarP(&cfg.Force, "force", "f", false, "Force re-install even if already completed")
	cmd.Flags().BoolVar(&cfg.IgnoreLock, "ignore-lock", false, "Ignore lock file when resolving modules")
}

// buildRunner constructs a Runner for the given config and component order.
func buildRunner(cfg runConfig, order []lifecycle.ComponentID, stack *rollback.Stack, sentinel *state.Manager) *lifecycle.Runner {
	eval := &starlarkpkg.Evaluator{}
	caller := &starlarkHookCaller{
		configDir: cfg.ConfigDir,
		eval:      eval,
		dryRun:    cfg.DryRun,
		stack:     stack,
	}
	return &lifecycle.Runner{
		Order:      order,
		Caller:     caller,
		Stack:      stack,
		Sentinel:   sentinel,
		NoRollback: cfg.NoRollback,
	}
}

// starlarkHookCaller implements lifecycle.HookCaller via the Starlark evaluator.
type starlarkHookCaller struct {
	configDir string
	eval      *starlarkpkg.Evaluator
	dryRun    bool
	stack     *rollback.Stack
}

// CallHook evaluates the component's Starlark file and calls the named hook.
func (h *starlarkHookCaller) CallHook(componentID, hookName string) error {
	// Resolve component file: <configDir>/components/<componentID>.star
	componentFile := filepath.Join(h.configDir, "components", componentID+".star")
	if _, err := os.Lstat(componentFile); os.IsNotExist(err) {
		// No component file — skip silently (component may have no hooks).
		return nil
	}

	caps := &ctx.Capabilities{
		DryRun:        h.dryRun,
		ComponentDir:  filepath.Dir(componentFile),
		Phase:         hookName,
		Component:     componentID,
		RollbackStack: h.stack,
		Env:           envMap(),
	}
	if home, err := os.UserHomeDir(); err == nil {
		caps.Home = home
		caps.StateDir = filepath.Join(home, ".local", "share", "meowctl", componentID)
	}

	ctxVal := ctx.New(caps)
	result, err := h.eval.ExecFile(componentFile, nil, nil, ctxVal)
	if err != nil {
		return fmt.Errorf("component %s: eval: %w", componentID, err)
	}
	if err := h.eval.CallHook(result.Globals, hookName, componentFile, ctxVal); err != nil {
		return fmt.Errorf("component %s hook %s: %w", componentID, hookName, err)
	}
	return nil
}

// envMap returns a copy of the current process environment as a map.
func envMap() map[string]string {
	env := make(map[string]string, len(os.Environ()))
	for _, kv := range os.Environ() {
		for i := 0; i < len(kv); i++ {
			if kv[i] == '=' {
				env[kv[:i]] = kv[i+1:]
				break
			}
		}
	}
	return env
}

// loadComponents evaluates <configDir>/meowctl.star and returns the declared
// component IDs in declaration order.
func loadComponents(configDir string, filter []string) ([]lifecycle.ComponentID, error) {
	starPath := filepath.Join(configDir, "meowctl.star")
	eval := &starlarkpkg.Evaluator{}
	result, err := eval.ExecFile(starPath, nil, nil, nil)
	if err != nil {
		return nil, exitErrorf(ExitConfig, "load meowctl.star: %v", err)
	}

	filterSet := make(map[string]bool, len(filter))
	for _, name := range filter {
		filterSet[name] = true
	}

	ids := make([]lifecycle.ComponentID, 0, len(result.Declarations.Components))
	for _, c := range result.Declarations.Components {
		if len(filterSet) > 0 && !filterSet[c.Name] {
			continue
		}
		ids = append(ids, c.Name)
	}
	if len(filter) > 0 && len(ids) == 0 {
		return nil, exitErrorf(ExitUsage, "no matching components found for: %v", filter)
	}
	return ids, nil
}

func newInstallCmd(gf *globalFlags) *cobra.Command {
	var cfg runConfig
	cmd := &cobra.Command{
		Use:   "install [<component>...]",
		Short: "Run the full install phase set for all (or specified) components",
		RunE: func(_ *cobra.Command, args []string) error {
			configDir, err := resolveConfigDir(gf)
			if err != nil {
				return err
			}
			cfg.ConfigDir = configDir
			return runLifecyclePhaseSet("install", lifecycle.PhaseSetInstall, cfg, args)
		},
	}
	addLifecycleFlags(cmd, &cfg)
	addInstallFlags(cmd, &cfg)
	return cmd
}

func newUpdateCmd(gf *globalFlags) *cobra.Command {
	var cfg runConfig
	cmd := &cobra.Command{
		Use:   "update [<component>...]",
		Short: "Run the update phase set for all (or specified) components",
		RunE: func(_ *cobra.Command, args []string) error {
			configDir, err := resolveConfigDir(gf)
			if err != nil {
				return err
			}
			cfg.ConfigDir = configDir
			return runLifecyclePhaseSet("update", lifecycle.PhaseSetUpdate, cfg, args)
		},
	}
	addLifecycleFlags(cmd, &cfg)
	addInstallFlags(cmd, &cfg)
	return cmd
}

func newUninstallCmd(gf *globalFlags) *cobra.Command {
	var cfg runConfig
	cmd := &cobra.Command{
		Use:   "uninstall [<component>...]",
		Short: "Run the uninstall phase for all (or specified) components",
		RunE: func(_ *cobra.Command, args []string) error {
			configDir, err := resolveConfigDir(gf)
			if err != nil {
				return err
			}
			cfg.ConfigDir = configDir
			return runLifecyclePhaseSet("uninstall", lifecycle.PhaseSetUninstall, cfg, args)
		},
	}
	addLifecycleFlags(cmd, &cfg)
	return cmd
}

func newVerifyCmd(gf *globalFlags) *cobra.Command {
	var cfg runConfig
	cmd := &cobra.Command{
		Use:   "verify [<component>...]",
		Short: "Verify the current environment against the dotfiles config",
		RunE: func(_ *cobra.Command, args []string) error {
			configDir, err := resolveConfigDir(gf)
			if err != nil {
				return err
			}
			cfg.ConfigDir = configDir
			return runLifecyclePhaseSet("verify", lifecycle.PhaseSetVerify, cfg, args)
		},
	}
	addLifecycleFlags(cmd, &cfg)
	return cmd
}

// runLifecyclePhaseSet executes a named phase set, loading components from meowctl.star.
func runLifecyclePhaseSet(name string, phases []lifecycle.Phase, cfg runConfig, filter []string) error {
	order, err := loadComponents(cfg.ConfigDir, filter)
	if err != nil {
		return err
	}

	journalPath := filepath.Join(cfg.ConfigDir, "rollback.jsonl")
	var stack *rollback.Stack
	if !cfg.DryRun {
		var openErr error
		stack, openErr = rollback.Open(journalPath)
		if openErr != nil {
			return fmt.Errorf("cannot open rollback journal: %w", openErr)
		}
		defer func() { _ = stack.Close() }()
	}

	statePath := filepath.Join(cfg.ConfigDir, "state.toml")
	sentinel := state.NewManager(statePath)
	runner := buildRunner(cfg, order, stack, sentinel)
	return runner.RunPhaseSet(name, phases)
}
