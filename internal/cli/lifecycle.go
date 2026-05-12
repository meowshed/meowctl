package cli

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/meowshed/meowctl/internal/ctx"
	"github.com/meowshed/meowctl/internal/lifecycle"
	"github.com/meowshed/meowctl/internal/rollback"
	starlarkpkg "github.com/meowshed/meowctl/internal/starlark"
	"github.com/spf13/cobra"
)

// runConfig holds runtime parameters shared across lifecycle commands.
type runConfig struct {
	DotfilesDir string
	DryRun      bool
	NoRollback  bool
}

// defaultRunConfig returns a runConfig populated from the environment.
// It panics if no home directory can be determined and MEOWCTL_DOTFILES is
// unset, because there is no safe default path to use.
func defaultRunConfig() runConfig {
	dir := os.Getenv("MEOWCTL_DOTFILES")
	if dir == "" {
		home, err := os.UserHomeDir()
		if err != nil {
			// Without a home directory there is no reasonable default; abort early
			// rather than silently using an empty path.
			panic(fmt.Sprintf("meowctl: cannot determine home directory: %v", err))
		}
		dir = filepath.Join(home, "dotfiles")
	}
	return runConfig{DotfilesDir: dir}
}

// addLifecycleFlags attaches --dry-run and --no-rollback flags to cmd.
func addLifecycleFlags(cmd *cobra.Command, cfg *runConfig) {
	cmd.Flags().BoolVar(&cfg.DryRun, "dry-run", false, "Print what would be done without executing")
	cmd.Flags().BoolVar(&cfg.NoRollback, "no-rollback", false, "Skip automatic rollback on failure")
}

// buildRunner constructs a Runner for the given config and component order.
func buildRunner(cfg runConfig, order []lifecycle.ComponentID, stack *rollback.Stack) *lifecycle.Runner {
	eval := &starlarkpkg.Evaluator{}
	caller := &starlarkHookCaller{
		dotfilesDir: cfg.DotfilesDir,
		eval:        eval,
		dryRun:      cfg.DryRun,
		stack:       stack,
	}
	return &lifecycle.Runner{
		Order:      order,
		Caller:     caller,
		Stack:      stack,
		NoRollback: cfg.NoRollback,
	}
}

// starlarkHookCaller implements lifecycle.HookCaller via the Starlark evaluator.
type starlarkHookCaller struct {
	dotfilesDir string
	eval        *starlarkpkg.Evaluator
	dryRun      bool
	stack       *rollback.Stack
}

// CallHook evaluates the component's Starlark file and calls the named hook.
func (h *starlarkHookCaller) CallHook(componentID, hookName string) error {
	// Resolve component file: <dotfiles>/<componentID>.star
	componentFile := filepath.Join(h.dotfilesDir, componentID+".star")
	if _, err := os.Stat(componentFile); os.IsNotExist(err) {
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

func newInstallCmd() *cobra.Command {
	cfg := defaultRunConfig()
	cmd := &cobra.Command{
		Use:   "install",
		Short: "Run the full install phase set for all components",
		Args:  cobra.NoArgs,
		RunE: func(_ *cobra.Command, _ []string) error {
			return runLifecyclePhaseSet("install", lifecycle.PhaseSetInstall, cfg)
		},
	}
	addLifecycleFlags(cmd, &cfg)
	return cmd
}

func newUninstallCmd() *cobra.Command {
	cfg := defaultRunConfig()
	cmd := &cobra.Command{
		Use:   "uninstall",
		Short: "Run the uninstall phase for all components",
		Args:  cobra.NoArgs,
		RunE: func(_ *cobra.Command, _ []string) error {
			return runLifecyclePhaseSet("uninstall", lifecycle.PhaseSetUninstall, cfg)
		},
	}
	addLifecycleFlags(cmd, &cfg)
	return cmd
}

func newVerifyCmd() *cobra.Command {
	cfg := defaultRunConfig()
	cmd := &cobra.Command{
		Use:   "verify",
		Short: "Verify the current environment against the dotfiles config",
		Args:  cobra.NoArgs,
		RunE: func(_ *cobra.Command, _ []string) error {
			return runLifecyclePhaseSet("verify", lifecycle.PhaseSetVerify, cfg)
		},
	}
	addLifecycleFlags(cmd, &cfg)
	return cmd
}

// runLifecyclePhaseSet executes a named phase set using a minimal component order.
// In this milestone the component list is empty — components will be populated
// once meowctl.star parsing is wired in M6.
func runLifecyclePhaseSet(name string, phases []lifecycle.Phase, cfg runConfig) error {
	home, err := os.UserHomeDir()
	if err != nil {
		return fmt.Errorf("cannot determine home directory: %w", err)
	}
	journalPath := filepath.Join(home, ".config", "meowctl", "rollback.jsonl")
	var stack *rollback.Stack
	if !cfg.DryRun {
		var openErr error
		stack, openErr = rollback.Open(journalPath)
		if openErr != nil {
			return fmt.Errorf("cannot open rollback journal: %w", openErr)
		}
		defer func() { _ = stack.Close() }()
	}

	// Component order will be populated from the lock file once component discovery is wired.
	order := []lifecycle.ComponentID{}
	if len(order) == 0 {
		fmt.Fprintf(os.Stderr, "meowctl: warning: no components in order; nothing to do (component discovery not yet wired)\n")
	}
	runner := buildRunner(cfg, order, stack)
	return runner.RunPhaseSet(name, phases)
}
