package cli

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/meowshed/meowctl/internal/ctx"
	"github.com/meowshed/meowctl/internal/lifecycle"
	"github.com/meowshed/meowctl/internal/rollback"
	starlarkpkg "github.com/meowshed/meowctl/internal/starlark"
	"github.com/meowshed/meowctl/internal/state"
	"github.com/meowshed/meowctl/internal/tui"
	"github.com/spf13/cobra"
	gostarlark "go.starlark.net/starlark"
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
func buildRunner(cfg runConfig, order []lifecycle.ComponentID, stack *rollback.Stack, sentinel *state.Manager, w tui.Writer) *lifecycle.Runner {
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
		Writer:     w,
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

// loadComponentsWithDeps evaluates <configDir>/meowctl.star, performs pass 1 across
// bare-name component files to collect after deps from globals, merges with after=
// kwargs from component() declarations, topo-sorts the result, and returns the
// ordered component IDs (logical names).
func loadComponentsWithDeps(configDir string, filter []string) ([]lifecycle.ComponentID, error) {
	starPath := filepath.Join(configDir, "meowctl.star")
	eval := &starlarkpkg.Evaluator{}

	result, err := eval.ExecFile(starPath, nil, nil, nil)
	if err != nil {
		return nil, exitErrorf(ExitConfig, "load meowctl.star: %v", err)
	}

	decls, err := filterDecls(result.Declarations.Components, filter)
	if err != nil {
		return nil, err
	}

	deps := buildDepsMap(configDir, eval, decls)
	declIdx := buildDeclIndex(decls)

	ordered, sortErr := lifecycle.TopoSort(deps, declIdx)
	if sortErr != nil {
		return nil, exitErrorf(ExitConfig, "component dependency cycle: %v", sortErr)
	}

	return retainDeclared(ordered, declIdx), nil
}

// filterDecls returns the subset of decls matching the filter (by logical name).
// If filter is non-empty and no match is found, returns ExitUsage error.
func filterDecls(all []starlarkpkg.ComponentDecl, filter []string) ([]starlarkpkg.ComponentDecl, error) {
	if len(filter) == 0 {
		return all, nil
	}
	filterSet := make(map[string]bool, len(filter))
	for _, name := range filter {
		filterSet[name] = true
	}
	out := make([]starlarkpkg.ComponentDecl, 0, len(all))
	for _, c := range all {
		if filterSet[c.LogicalName()] {
			out = append(out, c)
		}
	}
	if len(out) == 0 {
		return nil, exitErrorf(ExitUsage, "no matching components found for: %v", filter)
	}
	return out, nil
}

// buildDepsMap runs pass 1: for each component, merges after= kwarg with
// file-global after list (bare-name components only) and returns a deps map.
func buildDepsMap(configDir string, eval *starlarkpkg.Evaluator, decls []starlarkpkg.ComponentDecl) map[lifecycle.ComponentID][]lifecycle.ComponentID {
	deps := make(map[lifecycle.ComponentID][]lifecycle.ComponentID, len(decls))
	for _, c := range decls {
		merged := readComponentDeps(configDir, eval, c)
		deps[c.LogicalName()] = merged
	}
	return deps
}

// readComponentDeps returns the merged after list for a single component declaration.
func readComponentDeps(configDir string, eval *starlarkpkg.Evaluator, c starlarkpkg.ComponentDecl) []string {
	merged := make([]string, len(c.After))
	copy(merged, c.After)
	if strings.Contains(c.Name, "//") {
		return merged
	}
	componentFile := filepath.Join(configDir, "components", c.Name+".star")
	if _, statErr := os.Lstat(componentFile); statErr != nil {
		return merged
	}
	fileResult, readErr := eval.ReadComponentGlobals(componentFile, nil)
	if readErr != nil {
		return merged
	}
	return mergeAfterFromGlobals(merged, fileResult.Globals)
}

// buildDeclIndex returns a map from logical name to declaration index.
func buildDeclIndex(decls []starlarkpkg.ComponentDecl) map[lifecycle.ComponentID]int {
	idx := make(map[lifecycle.ComponentID]int, len(decls))
	for i, c := range decls {
		idx[c.LogicalName()] = i
	}
	return idx
}

// retainDeclared filters the topo-sorted list to only include declared components.
func retainDeclared(ordered []lifecycle.ComponentID, declIdx map[lifecycle.ComponentID]int) []lifecycle.ComponentID {
	ids := make([]lifecycle.ComponentID, 0, len(declIdx))
	for _, id := range ordered {
		if _, ok := declIdx[id]; ok {
			ids = append(ids, id)
		}
	}
	return ids
}

// mergeAfterFromGlobals reads the "after" global from a component's StringDict
// and merges it into existing deps, deduplicating entries.
func mergeAfterFromGlobals(existing []string, globals gostarlark.StringDict) []string {
	afterVal, ok := globals["after"]
	if !ok {
		return existing
	}
	list, ok := afterVal.(*gostarlark.List)
	if !ok {
		return existing
	}
	seen := make(map[string]bool, len(existing))
	for _, e := range existing {
		seen[e] = true
	}
	result := existing
	for i := 0; i < list.Len(); i++ {
		s, ok := list.Index(i).(gostarlark.String)
		if !ok {
			continue
		}
		name := string(s)
		if !seen[name] {
			seen[name] = true
			result = append(result, name)
		}
	}
	return result
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
	order, err := loadComponentsWithDeps(cfg.ConfigDir, filter)
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

	w := tui.New(os.Stdout)
	defer func() { _ = w.Close() }()

	statePath := filepath.Join(cfg.ConfigDir, "state.toml")
	sentinel := state.NewManager(statePath)
	runner := buildRunner(cfg, order, stack, sentinel, w)
	return runner.RunPhaseSet(name, phases)
}
