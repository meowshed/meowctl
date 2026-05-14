package cli

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/meowshed/meowctl/internal/ctx"
	"github.com/meowshed/meowctl/internal/lifecycle"
	"github.com/meowshed/meowctl/internal/pkg"
	"github.com/meowshed/meowctl/internal/rollback"
	starlarkpkg "github.com/meowshed/meowctl/internal/starlark"
	"github.com/meowshed/meowctl/internal/starlark/loader"
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
func buildRunner(cfg runConfig, order []lifecycle.ComponentID, stack *rollback.Stack, sentinel *state.Manager, w tui.Writer, pmReg *pkg.PMRegistry) *lifecycle.Runner {
	eval := &starlarkpkg.Evaluator{}
	caller := &starlarkHookCaller{
		configDir:  cfg.ConfigDir,
		eval:       eval,
		dryRun:     cfg.DryRun,
		stack:      stack,
		pmRegistry: pmReg,
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

// phaseEmitFile returns the absolute path of the shell init file that
// ctx.emit should append to during install-time execution of the given phase.
// Returns empty string if the phase does not write to any rc file, or if the
// shell name is unrecognised.
func phaseEmitFile(home, phase, shellName string) string {
	switch phase {
	case string(lifecycle.PhaseShell):
		switch shellName {
		case "zsh":
			return filepath.Join(home, ".zshrc")
		case "bash":
			return filepath.Join(home, ".bashrc")
		case "fish":
			return filepath.Join(home, ".config", "fish", "config.fish")
		}
	case string(lifecycle.PhaseLogin):
		switch shellName {
		case "zsh":
			return filepath.Join(home, ".zprofile")
		case "bash":
			return filepath.Join(home, ".bash_profile")
		case "fish":
			return filepath.Join(home, ".config", "fish", "conf.d", "meowctl.fish")
		}
	}
	return ""
}

// shellFromEnv returns the base name of the user's login shell derived from
// the SHELL environment variable (e.g. "zsh", "bash", "fish").
func shellFromEnv() string {
	return strings.ToLower(filepath.Base(os.Getenv("SHELL")))
}

type starlarkHookCaller struct {
	configDir  string
	eval       *starlarkpkg.Evaluator
	dryRun     bool
	stack      *rollback.Stack
	pmRegistry *pkg.PMRegistry
	loader     *loader.CompositeLoader // may be nil for bare-name only configs
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
		ComponentDir:  h.resolveComponentDir(componentID, componentFile),
		Phase:         hookName,
		Component:     componentID,
		RollbackStack: h.stack,
		PMRegistry:    h.pmRegistry,
		Env:           envMap(),
	}
	if home, err := os.UserHomeDir(); err == nil {
		caps.Home = home
		caps.StateDir = filepath.Join(home, ".local", "share", "meowctl", componentID)
		caps.EmitFile = phaseEmitFile(home, hookName, shellFromEnv())
	}

	ctxVal := ctx.New(caps)
	result, err := h.eval.ExecFile(componentFile, nil, nil, ctxVal)
	if err != nil {
		return fmt.Errorf("component %s: eval: %w", componentID, err)
	}
	if err := h.eval.CallHook(result.Globals, hookName, componentFile, ctxVal); err != nil {
		return fmt.Errorf("component %s hook %s: %w", componentID, hookName, err)
	}

	// Dispatch pkg() declarations to registered PM handlers.
	if err := dispatchPackages(hookName, result.Declarations.Packages, h.pmRegistry, ctxVal); err != nil {
		return fmt.Errorf("component %s pkg dispatch: %w", componentID, err)
	}

	return nil
}

// resolveComponentDir returns the filesystem directory for a component's source file.
// For URL-named components (containing "//"), it tries CompositeLoader.ComponentDir first.
// For bare-name local components, or when the loader is nil, it falls back to
// filepath.Dir(componentFile).
func (h *starlarkHookCaller) resolveComponentDir(componentID, componentFile string) string {
	if h.loader != nil && strings.Contains(componentID, "//") {
		if dir, err := h.loader.ComponentDir(componentID); err == nil {
			return dir
		}
	}
	return filepath.Dir(componentFile)
}

// dispatchPackages iterates pkg declarations and calls the appropriate PM handler.
// Only install, uninstall, and update phases trigger dispatch; other phases are no-ops.
func dispatchPackages(phase string, packages []starlarkpkg.PkgDecl, reg *pkg.PMRegistry, ctxVal gostarlark.Value) error {
	if reg == nil || len(packages) == 0 {
		return nil
	}
	for _, p := range packages {
		var err error
		if phase == "update" {
			err = reg.DispatchUpdate(p.Manager, p.Name, p.Kwargs, ctxVal)
		} else {
			err = reg.Dispatch(phase, p.Manager, p.Name, p.Version, p.Kwargs, ctxVal)
		}
		if err != nil {
			return err
		}
	}
	return nil
}

// envMap returns a copy of the current process environment as a map.
func envMap() map[string]string {
	env := make(map[string]string, len(os.Environ()))
	for _, kv := range os.Environ() {
		if k, v, ok := strings.Cut(kv, "="); ok {
			env[k] = v
		}
	}
	return env
}

// loadComponentsWithDeps evaluates <configDir>/meowctl.star, performs pass 1 across
// bare-name component files to collect after deps from globals, merges with after=
// kwargs from component() declarations, topo-sorts the result, and returns the
// ordered component IDs (logical names) and a PMRegistry built from pass-1 globals.
func loadComponentsWithDeps(configDir string, filter []string) ([]lifecycle.ComponentID, *pkg.PMRegistry, error) {
	starPath := filepath.Join(configDir, "meowctl.star")
	eval := &starlarkpkg.Evaluator{}

	result, err := eval.ExecFile(starPath, nil, nil, nil)
	if err != nil {
		return nil, nil, exitErrorf(ExitConfig, "load meowctl.star: %v", err)
	}

	decls, err := filterDecls(result.Declarations.Components, filter)
	if err != nil {
		return nil, nil, err
	}

	deps, pmReg := buildDepsAndRegistry(configDir, eval, decls)
	declIdx := buildDeclIndex(decls)

	ordered, sortErr := lifecycle.TopoSort(deps, declIdx)
	if sortErr != nil {
		return nil, nil, exitErrorf(ExitConfig, "component dependency cycle: %v", sortErr)
	}

	return retainDeclared(ordered, declIdx), pmReg, nil
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

// buildDepsAndRegistry runs pass 1: for each component, merges after= kwarg with
// file-global after list (bare-name components only), scans globals for PM registration,
// and returns a deps map and a populated PMRegistry.
func buildDepsAndRegistry(configDir string, eval *starlarkpkg.Evaluator, decls []starlarkpkg.ComponentDecl) (map[lifecycle.ComponentID][]lifecycle.ComponentID, *pkg.PMRegistry) {
	deps := make(map[lifecycle.ComponentID][]lifecycle.ComponentID, len(decls))
	pmReg := pkg.NewPMRegistry()
	for _, c := range decls {
		merged, globals := readComponentGlobals(configDir, eval, c)
		deps[c.LogicalName()] = merged
		if globals != nil {
			h := pkg.ScanGlobals(c.LogicalName(), globals, func(msg string) {
				fmt.Fprintln(os.Stderr, "warning:", msg)
			})
			if h != nil {
				// Register by pm_name (the manager identifier, e.g. "npm"),
				// not by the component's logical name (e.g. "node").
				if pmNameVal, ok := globals["pm_name"]; ok {
					if pmNameStr, ok := pmNameVal.(gostarlark.String); ok {
						pmReg.Register(string(pmNameStr), h)
					}
				}
			}
		}
	}
	return deps, pmReg
}

// readComponentGlobals returns the merged after list and the raw globals dict for a single
// component declaration. globals is nil if the component file is a URL-named component
// or if reading the file failed (non-fatal; deps gracefully degrade).
func readComponentGlobals(configDir string, eval *starlarkpkg.Evaluator, c starlarkpkg.ComponentDecl) ([]string, gostarlark.StringDict) {
	merged := make([]string, len(c.After))
	copy(merged, c.After)
	if strings.Contains(c.Name, "//") {
		return merged, nil
	}
	componentFile := filepath.Join(configDir, "components", c.Name+".star")
	if _, statErr := os.Lstat(componentFile); statErr != nil {
		return merged, nil
	}
	fileResult, readErr := eval.ReadComponentGlobals(componentFile, nil)
	if readErr != nil {
		fmt.Fprintln(os.Stderr, "warning: reading component globals:", readErr)
		return merged, nil
	}
	return mergeAfterFromGlobals(merged, fileResult.Globals), fileResult.Globals
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
	order, pmReg, err := loadComponentsWithDeps(cfg.ConfigDir, filter)
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
	runner := buildRunner(cfg, order, stack, sentinel, w, pmReg)
	return runner.RunPhaseSet(name, phases)
}
