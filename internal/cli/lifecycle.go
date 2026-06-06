package cli

import (
	"fmt"
	"os"
	"path/filepath"
	"runtime"
	"strings"

	"github.com/meowshed/meowctl/internal/ctx"
	"github.com/meowshed/meowctl/internal/lifecycle"
	"github.com/meowshed/meowctl/internal/modfile"
	"github.com/meowshed/meowctl/internal/pkg"
	"github.com/meowshed/meowctl/internal/rollback"
	starlarkpkg "github.com/meowshed/meowctl/internal/starlark"
	"github.com/meowshed/meowctl/internal/starlark/loader"
	"github.com/meowshed/meowctl/internal/state"
	"github.com/meowshed/meowctl/internal/tui"
	"github.com/spf13/cobra"
	gostarlark "go.starlark.net/starlark"
)

// goosToPlatformOS returns the meowctl platform OS identifier for the current
// runtime OS. "darwin" → "macos"; all others pass through unchanged.
func goosToPlatformOS() string {
	if runtime.GOOS == "darwin" {
		return "macos"
	}
	return runtime.GOOS
}

// linuxDistroInfo reads /etc/os-release and returns (distro, distroLike).
// distro is the normalised ID value (e.g. "ubuntu", "arch", "alpine").
// distroLike is the first token of ID_LIKE (e.g. "debian", "fedora").
// Both are empty strings on non-Linux or when the file is unreadable.
func linuxDistroInfo() (distro, distroLike string) {
	data, err := os.ReadFile("/etc/os-release")
	if err != nil {
		return "", ""
	}
	for _, line := range strings.Split(string(data), "\n") {
		line = strings.TrimSpace(line)
		k, v, ok := strings.Cut(line, "=")
		if !ok {
			continue
		}
		v = strings.Trim(v, `"'`)
		switch k {
		case "ID":
			distro = strings.ToLower(v)
		case "ID_LIKE":
			// ID_LIKE may be space-separated; take the first token.
			parts := strings.Fields(v)
			if len(parts) > 0 {
				distroLike = strings.ToLower(parts[0])
			}
		}
	}
	return distro, distroLike
}

// readModfileReplaces reads replace() directives from <configDir>/deps.mod and
// returns a map from module name to local filesystem path for path-based overrides.
// Source-based replace() directives are handled at sync time via SyncModules.
// Returns an empty map if the file is absent or cannot be parsed.
func readModfileReplaces(configDir string) map[string]string {
	modPath := filepath.Join(configDir, configModFile)
	mf, err := modfile.Parse(modPath)
	if err != nil {
		return nil
	}
	if len(mf.Replace) == 0 {
		return nil
	}
	result := make(map[string]string, len(mf.Replace))
	for _, r := range mf.Replace {
		if r.Path != "" {
			result[r.Name] = r.Path
		}
		// Source-based replaces are resolved during sync; skip here.
	}
	return result
}

// runConfig holds runtime parameters shared across lifecycle commands.
type runConfig struct {
	ConfigDir  string
	DryRun     bool
	NoRollback bool
	Force      bool
	IgnoreLock bool
	Verbose    bool
	// ExtraPMReg and ExtraURLMap carry PM handlers and tarball URLs loaded
	// before a destructive config edit (e.g. remove). runApply merges them
	// into the main registry so uninstall phases have all required handlers.
	ExtraPMReg  *pkg.PMRegistry
	ExtraURLMap map[lifecycle.ComponentID]string
}

// addLifecycleFlags attaches --dry-run, --no-rollback, and --verbose flags to cmd.
func addLifecycleFlags(cmd *cobra.Command, cfg *runConfig) {
	cmd.Flags().BoolVarP(&cfg.DryRun, "dry-run", "n", false, "Print what would be done without executing")
	cmd.Flags().BoolVar(&cfg.NoRollback, "no-rollback", false, "Skip automatic rollback on failure")
	cmd.Flags().BoolVarP(&cfg.Verbose, "verbose", "v", false, "Enable verbose output (commands, output, phase transitions)")
}

// addInstallFlags attaches --force and --ignore-lock flags to install/update commands.
func addInstallFlags(cmd *cobra.Command, cfg *runConfig) {
	cmd.Flags().BoolVarP(&cfg.Force, "force", "f", false, "Force re-install even if already completed")
	cmd.Flags().BoolVar(&cfg.IgnoreLock, "ignore-lock", false, "Ignore lock file when resolving modules")
}

// buildHookCaller constructs a starlarkHookCaller with a CompositeLoader wired
// with replace directives read from <configDir>/deps.mod.
func buildHookCaller(cfg runConfig) *starlarkHookCaller {
	distro, distroLike := "", ""
	if runtime.GOOS == "linux" {
		distro, distroLike = linuxDistroInfo()
	}
	eval := &starlarkpkg.Evaluator{
		Platform: starlarkpkg.PlatformInfo{
			OS:         goosToPlatformOS(),
			Distro:     distro,
			DistroLike: distroLike,
		},
	}
	var cl *loader.CompositeLoader
	if home, err := os.UserHomeDir(); err == nil {
		cl = loader.NewCompositeLoader(cfg.ConfigDir, nil, loader.CompositeLoaderOptions{
			CacheDir: filepath.Join(home, ".cache", "meowctl"),
			LockPath: filepath.Join(cfg.ConfigDir, configLockFile),
			Replaces: readModfileReplaces(cfg.ConfigDir),
		})
	}
	return &starlarkHookCaller{
		configDir: cfg.ConfigDir,
		eval:      eval,
		dryRun:    cfg.DryRun,
		verbose:   cfg.Verbose,
		loader:    cl,
	}
}

// buildRunner constructs a Runner for the given config and component order.
func buildRunner(cfg runConfig, order []lifecycle.ComponentID, stack *rollback.Stack, sentinel *state.Manager, w tui.Writer, pmReg *pkg.PMRegistry, caller *starlarkHookCaller) *lifecycle.Runner {
	caller.stack = stack
	caller.pmRegistry = pmReg
	return &lifecycle.Runner{
		Order:      order,
		Caller:     caller,
		Stack:      stack,
		Sentinel:   sentinel,
		NoRollback: cfg.NoRollback,
		Force:      cfg.Force,
		Verbose:    cfg.Verbose,
		Writer:     w,
	}
}

type starlarkHookCaller struct {
	configDir     string
	eval          *starlarkpkg.Evaluator
	dryRun        bool
	verbose       bool
	stack         *rollback.Stack
	pmRegistry    *pkg.PMRegistry
	loader        *loader.CompositeLoader // may be nil for bare-name only configs
	urlComponents map[string]string       // logical name → original URL for URL-named components
	// pkgsPins records packages dispatched during install/upgrade per component.
	// Key is the component logical name; value is the list of PkgDecls dispatched.
	pkgsPins map[string][]starlarkpkg.PkgDecl
}

// CallHook evaluates the component's Starlark file and calls the named hook.
func (h *starlarkHookCaller) CallHook(componentID, hookName string) error {
	// Resolve component file using the init.star convention:
	//   <configDir>/components/<componentID>/init.star
	componentFile := filepath.Join(h.configDir, "components", componentID, "init.star")
	if _, err := os.Lstat(componentFile); os.IsNotExist(err) {
		// No local file — check whether this component is URL-named
		// (e.g. @stdlib//components/mise). If so, load it via the
		// CompositeLoader instead of returning nil.
		if h.loader != nil {
			if origURL, ok := h.urlComponents[componentID]; ok {
				return h.callHookFromURL(componentID, origURL, hookName)
			}
		}
		// No component file and no URL mapping — skip silently.
		return nil
	}

	return h.callHookFromFile(componentID, componentFile, hookName)
}

// callHookFromFile evaluates a local .star file and dispatches its hook,
// then runs any hooks/<componentID>.star extension hook.
func (h *starlarkHookCaller) callHookFromFile(componentID, componentFile, hookName string) error {
	// If a subdirectory with the component's name exists alongside the .star
	// file, use it as ComponentDir. This supports dotmeow-style components where
	// data files live in a subdir (e.g. components/git-config/config).
	componentDir := filepath.Dir(componentFile)
	if subdir := filepath.Join(componentDir, componentID); isDir(subdir) {
		componentDir = subdir
	}

	caps := &ctx.Capabilities{
		DryRun:        h.dryRun,
		Verbose:       h.verbose,
		ComponentDir:  componentDir,
		Phase:         hookName,
		Component:     componentID,
		RollbackStack: h.stack,
		PMRegistry:    h.pmRegistry,
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
	if err := dispatchRepos(hookName, result.Declarations.Repos, h.pmRegistry, ctxVal); err != nil {
		return fmt.Errorf("component %s repo dispatch: %w", componentID, err)
	}
	if err := dispatchPackages(hookName, result.Declarations.Packages, h.pmRegistry, ctxVal); err != nil {
		return fmt.Errorf("component %s pkg dispatch: %w", componentID, err)
	}
	if (hookName == "install" || hookName == "upgrade") && len(result.Declarations.Packages) > 0 {
		if h.pkgsPins == nil {
			h.pkgsPins = make(map[string][]starlarkpkg.PkgDecl)
		}
		h.pkgsPins[componentID] = append(h.pkgsPins[componentID], result.Declarations.Packages...)
	}
	return callExtensionHook(h.configDir, componentID, hookName, caps, h.eval)
}

// callHookFromURL loads a URL-named component via the CompositeLoader and dispatches its hook.
func (h *starlarkHookCaller) callHookFromURL(componentID, moduleURL, hookName string) error {
	var componentDir string
	if dir, err := h.loader.ComponentDir(moduleURL); err == nil {
		componentDir = dir
	}

	caps := &ctx.Capabilities{
		DryRun:        h.dryRun,
		Verbose:       h.verbose,
		ComponentDir:  componentDir,
		Phase:         hookName,
		Component:     componentID,
		RollbackStack: h.stack,
		PMRegistry:    h.pmRegistry,
		Env:           envMap(),
	}
	if home, err := os.UserHomeDir(); err == nil {
		caps.Home = home
		caps.StateDir = filepath.Join(home, ".local", "share", "meowctl", componentID)
	}

	ctxVal := ctx.New(caps)

	// Wire an Accumulator so that top-level pkg() declarations in the module are
	// captured during loading. Without this, pkg() will error when acc is absent.
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

	// Use repo() declarations collected during module loading for dispatch (before pkgs).
	if err := dispatchRepos(hookName, acc.Repos, h.pmRegistry, ctxVal); err != nil {
		return fmt.Errorf("component %s repo dispatch: %w", componentID, err)
	}
	// Use pkg() declarations collected during module loading for dispatch.
	if err := dispatchPackages(hookName, acc.Packages, h.pmRegistry, ctxVal); err != nil {
		return fmt.Errorf("component %s pkg dispatch: %w", componentID, err)
	}
	if (hookName == "install" || hookName == "upgrade") && len(acc.Packages) > 0 {
		if h.pkgsPins == nil {
			h.pkgsPins = make(map[string][]starlarkpkg.PkgDecl)
		}
		h.pkgsPins[componentID] = append(h.pkgsPins[componentID], acc.Packages...)
	}
	return callExtensionHook(h.configDir, componentID, hookName, caps, h.eval)
}

// dispatchPackages iterates pkg declarations and calls the appropriate PM handler.
// Only install, uninstall, and upgrade phases trigger dispatch; other phases are no-ops.
func dispatchPackages(phase string, packages []starlarkpkg.PkgDecl, reg *pkg.PMRegistry, ctxVal gostarlark.Value) error {
	if reg == nil || len(packages) == 0 {
		return nil
	}
	for _, p := range packages {
		var err error
		if phase == "upgrade" {
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

// dispatchRepos iterates repo declarations and calls the appropriate PM add_repo handler.
// Only the install phase triggers repo dispatch; other phases are no-ops.
func dispatchRepos(phase string, repos []starlarkpkg.RepoDecl, reg *pkg.PMRegistry, ctxVal gostarlark.Value) error {
	if reg == nil || len(repos) == 0 || phase != "install" {
		return nil
	}
	for _, r := range repos {
		if err := reg.DispatchAddRepo(r.Manager, r.Kwargs, ctxVal); err != nil {
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

// checkLegacyConfig checks if a legacy meowctl.star exists without init.star
// and returns an error with migration instructions if so.
func checkLegacyConfig(configDir, starPath string) error {
	legacyPath := filepath.Join(configDir, "meowctl.star")
	if _, legacyErr := os.Lstat(legacyPath); legacyErr == nil {
		if _, newErr := os.Lstat(starPath); os.IsNotExist(newErr) {
			fmt.Fprintf(os.Stderr, "meowctl: found legacy config — rename files to continue:\n")
			fmt.Fprintf(os.Stderr, "  mv %s %s\n", legacyPath, starPath)
			fmt.Fprintf(os.Stderr, "  mv %s %s\n",
				filepath.Join(configDir, "meowctl.mod"), filepath.Join(configDir, configModFile))
			fmt.Fprintf(os.Stderr, "  mv %s %s\n",
				filepath.Join(configDir, "meowctl.lock"), filepath.Join(configDir, configLockFile))
			return exitErrorf(ExitConfig, "legacy config found — see instructions above")
		}
	}
	return nil
}

// mergeLocalStar loads local.star if present and appends its components into result,
// skipping any component already declared in init.star (first-wins dedup).
func mergeLocalStar(configDir string, eval *starlarkpkg.Evaluator, result *starlarkpkg.EvalResult) error {
	localPath := filepath.Join(configDir, configLocalFile)
	if _, localErr := os.Lstat(localPath); localErr != nil {
		return nil
	}
	localResult, localLoadErr := eval.ExecFile(localPath, nil, nil, nil)
	if localLoadErr != nil {
		return exitErrorf(ExitConfig, "load local.star: %v", localLoadErr)
	}
	initNames := make(map[string]bool, len(result.Declarations.Components))
	for _, c := range result.Declarations.Components {
		initNames[c.LogicalName()] = true
	}
	for _, c := range localResult.Declarations.Components {
		if !initNames[c.LogicalName()] {
			result.Declarations.Components = append(result.Declarations.Components, c)
		}
	}
	return nil
}

// loadComponentsWithDeps evaluates <configDir>/init.star, then iteratively
// discovers transitive after= dependencies by reading each component's file-level
// globals (including URL-named @registry// components loaded via CompositeLoader).
// Unknown deps encountered during expansion are auto-discovered and added as
// synthetic ComponentDecl entries so callers don't need to pre-declare every
// transitive dependency in init.star.
// Replace directives are read from <configDir>/deps.mod automatically.
// Returns the topo-sorted component IDs (logical names) and a PMRegistry.
//
// runSyntheticHooks controls whether synthetic (auto-discovered) dependency components
// run their own lifecycle hooks. Pass true for install/upgrade (deps should install
// themselves first), false for uninstall (only the declared components are uninstalled,
// not their shared PM dependencies like brew or mise).
func loadComponentsWithDeps(configDir string, filter []string, runSyntheticHooks bool) ([]lifecycle.ComponentID, *pkg.PMRegistry, map[string]string, error) {
	starPath := filepath.Join(configDir, configEntryFile)

	if err := checkLegacyConfig(configDir, starPath); err != nil {
		return nil, nil, nil, err
	}

	distro, distroLike := "", ""
	if runtime.GOOS == "linux" {
		distro, distroLike = linuxDistroInfo()
	}
	eval := &starlarkpkg.Evaluator{
		Platform: starlarkpkg.PlatformInfo{
			OS:         goosToPlatformOS(),
			Distro:     distro,
			DistroLike: distroLike,
		},
	}

	result, err := eval.ExecFile(starPath, nil, nil, nil)
	if err != nil {
		return nil, nil, nil, exitErrorf(ExitConfig, "load init.star: %v", err)
	}

	if err := mergeLocalStar(configDir, eval, result); err != nil {
		return nil, nil, nil, err
	}

	// Build a CompositeLoader for resolving URL-named components (e.g. @stdlib//...).
	// Replace directives come from deps.mod.
	var cl *loader.CompositeLoader
	if home, err := os.UserHomeDir(); err == nil {
		cl = loader.NewCompositeLoader(configDir, nil, loader.CompositeLoaderOptions{
			CacheDir: filepath.Join(home, ".cache", "meowctl"),
			LockPath: filepath.Join(configDir, configLockFile),
			Replaces: readModfileReplaces(configDir),
		})
	}

	// Seed from init.star declarations; filter to requested components first.
	seed, err := filterDecls(result.Declarations.Components, filter)
	if err != nil {
		return nil, nil, nil, err
	}

	// Iteratively expand transitive deps. Unknown deps (not in init.star) are
	// auto-discovered by reading their component file globals, creating synthetic
	// ComponentDecl entries. This allows test-*.star to declare
	// after = ["@stdlib//components/apt"] without pre-declaring apt in init.star.
	// globalsCache holds the already-read globals for each component (logical name),
	// passed to buildDepsAndRegistry to avoid re-reading files.
	allDecls, globalsCache := expandWithFileDeps(configDir, eval, cl, seed, result.Declarations.Components)

	deps, pmReg := buildDepsAndRegistry(eval, allDecls, globalsCache)

	// For uninstall-like phases, only the user-declared (seed) components should
	// have their hooks executed. Synthetic deps (e.g. brew, mise) are included in
	// the topo sort for ordering purposes, but should not run their own uninstall
	// hooks — otherwise uninstalling a single tool would tear down the whole PM.
	declBase := allDecls
	if !runSyntheticHooks {
		declBase = seed
	}
	declIdx := buildDeclIndex(declBase)

	ordered, sortErr := lifecycle.TopoSort(deps, declIdx)
	if sortErr != nil {
		return nil, nil, nil, exitErrorf(ExitConfig, "component dependency cycle: %v", sortErr)
	}

	// Build logical-name → original-URL map for URL-named components so that
	// CallHook can load them via CompositeLoader when no local file exists.
	// This includes registry components (@name) and URL-path components (@name//path).
	urlMap := make(map[string]string)
	for _, c := range allDecls {
		if strings.Contains(c.Name, "//") || strings.HasPrefix(c.Name, "@") {
			urlMap[c.LogicalName()] = c.Name
		}
	}

	return retainDeclared(ordered, declIdx), pmReg, urlMap, nil
}

// expandWithFileDeps iteratively discovers all transitive after= dependencies
// starting from seed, reading each component's file-level globals to find deps
// that may not be declared in meowctl.star. URL-named components are loaded via cl.
// Returns the full set of ComponentDecl entries (seed + all transitive deps) in
// a dependency-first order suitable for topo-sort, and a globals cache keyed by
// logical name so callers avoid re-reading files.
func expandWithFileDeps(configDir string, eval *starlarkpkg.Evaluator, cl *loader.CompositeLoader, seed []starlarkpkg.ComponentDecl, declared []starlarkpkg.ComponentDecl) ([]starlarkpkg.ComponentDecl, map[string]gostarlark.StringDict) {
	// Build index of all known declarations by logical name.
	byName := make(map[string]starlarkpkg.ComponentDecl, len(declared)+len(seed))
	for _, c := range declared {
		byName[c.LogicalName()] = c
	}
	for _, c := range seed {
		byName[c.LogicalName()] = c
	}

	included := make(map[string]bool)
	globalsCache := make(map[string]gostarlark.StringDict)
	var result []starlarkpkg.ComponentDecl

	var walk func(c starlarkpkg.ComponentDecl)
	walk = func(c starlarkpkg.ComponentDecl) {
		ln := c.LogicalName()
		if included[ln] {
			return
		}

		// Read file-level after= to discover additional deps before marking included,
		// so that deps are appended before this component (dependency-first).
		fileDeps, globals := readComponentGlobals(configDir, eval, c, cl)
		globalsCache[ln] = globals

		// Walk all after deps (from declaration kwargs + file globals).
		allDeps := mergeUniq(c.After, fileDeps)
		// URL-named components may declare bare-string deps for sibling
		// components in the same module (e.g. "fish-config" inside
		// "@dotmeow" or "@dotmeow//dotmeow"). Prefix them so they become
		// resolvable URLs.
		if strings.HasPrefix(c.Name, "@") {
			for i, dep := range allDeps {
				allDeps[i] = resolveBareDep(c.Name, dep)
			}
		}
		for _, dep := range allDeps {
			depLN := starlarkpkg.LogicalNameOf(dep)
			if !included[depLN] {
				if dc, ok := byName[depLN]; ok {
					walk(dc)
				} else {
					// Auto-discover: create a synthetic decl for this dep.
					synthetic := starlarkpkg.ComponentDecl{Name: dep, Synthetic: true}
					byName[depLN] = synthetic
					walk(synthetic)
				}
			}
		}

		included[ln] = true
		result = append(result, c)
	}

	for _, c := range seed {
		walk(c)
	}
	return result, globalsCache
}

// modulePrefix extracts the module prefix from a component URL.
// For "@dotmeow//dotmeow" it returns "@dotmeow//".
// For "@dotmeow" (root component) it returns "@dotmeow//".
// For "self//components/mycomp" it returns "self//".
// For local names (no "@") it returns "".
func modulePrefix(url string) string {
	if idx := strings.Index(url, "//"); idx >= 0 {
		return url[:idx+2]
	}
	// @name without // — treat as a module prefix for component resolution.
	if strings.HasPrefix(url, "@") {
		return url + "//"
	}
	return ""
}

// resolveBareDep converts a bare-string dependency discovered inside a URL-named
// component into a fully-qualified module URL. If dep already contains "//" or starts
// with "@" it is returned unchanged. For local components (parentURL has no "@") dep
// is also returned unchanged.
func resolveBareDep(parentURL, dep string) string {
	if strings.Contains(dep, "//") || strings.HasPrefix(dep, "@") {
		return dep
	}
	prefix := modulePrefix(parentURL)
	if prefix == "" {
		return dep
	}
	return prefix + "components/" + dep
}

// mergeUniq returns a deduplicated union of a and b, preserving order.
func mergeUniq(a, b []string) []string {
	seen := make(map[string]bool, len(a)+len(b))
	out := make([]string, 0, len(a)+len(b))
	for _, s := range a {
		if !seen[s] {
			seen[s] = true
			out = append(out, s)
		}
	}
	for _, s := range b {
		if !seen[s] {
			seen[s] = true
			out = append(out, s)
		}
	}
	return out
}

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

// platformMatches returns true if the globals dict does not declare a "platforms" list,
// or if the list contains the current OS.
func platformMatches(globals gostarlark.StringDict, os string) bool {
	platformsVal, ok := globals["platforms"]
	if !ok {
		return true
	}
	platformsList, ok := platformsVal.(*gostarlark.List)
	if !ok {
		return true
	}
	for i := 0; i < platformsList.Len(); i++ {
		if s, ok := platformsList.Index(i).(gostarlark.String); ok && string(s) == os {
			return true
		}
	}
	return false
}

// distroMatches returns true if the globals dict does not declare a "distros" list,
// or if the list contains the current distro or distro_like value.
func distroMatches(globals gostarlark.StringDict, distro, distroLike string) bool {
	distrosVal, ok := globals["distros"]
	if !ok {
		return true
	}
	distrosList, ok := distrosVal.(*gostarlark.List)
	if !ok {
		return true
	}
	for i := 0; i < distrosList.Len(); i++ {
		if s, ok := distrosList.Index(i).(gostarlark.String); ok {
			d := string(s)
			if d == distro || d == distroLike {
				return true
			}
		}
	}
	return false
}

// buildDepsAndRegistry builds the deps map and PMRegistry from already-read globals.
// globalsCache maps logical component name to its StringDict (nil if file was absent or errored).
// Components that declare a platforms list that does not include the current OS are skipped.
// Components that declare a distros list that does not include the current distro are skipped.
func buildDepsAndRegistry(eval *starlarkpkg.Evaluator, decls []starlarkpkg.ComponentDecl, globalsCache map[string]gostarlark.StringDict) (map[lifecycle.ComponentID][]lifecycle.ComponentID, *pkg.PMRegistry) {
	deps := make(map[lifecycle.ComponentID][]lifecycle.ComponentID, len(decls))
	pmReg := pkg.NewPMRegistry()
	skipped := make(map[lifecycle.ComponentID]bool)

	for _, c := range decls {
		ln := c.LogicalName()
		globals := globalsCache[ln]

		// Reconstruct merged after list from decl kwargs + file globals.
		merged := make([]string, len(c.After))
		copy(merged, c.After)
		merged = mergeAfterFromGlobals(merged, globals, eval)

		// Skip component if platforms or distros filter does not match current platform.
		if globals != nil {
			if !platformMatches(globals, eval.Platform.OS) {
				skipped[ln] = true
				continue
			}
			if !distroMatches(globals, eval.Platform.Distro, eval.Platform.DistroLike) {
				skipped[ln] = true
				continue
			}
		}

		// Filter out after-deps that point to skipped components.
		filtered := merged[:0:len(merged)]
		for _, dep := range merged {
			depLN := starlarkpkg.LogicalNameOf(dep)
			if !skipped[depLN] {
				filtered = append(filtered, depLN)
			}
		}
		deps[ln] = filtered

		if globals != nil {
			h := pkg.ScanGlobals(ln, globals, func(msg string) {
				fmt.Fprintln(os.Stderr, "warning:", msg)
			})
			if h != nil {
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
// component declaration. For registry components (starting with "@"), cl is used to load
// and evaluate the remote/replaced module. For bare-name local components, the file is
// read from <configDir>/components/<name>/init.star. globals is nil on failure (non-fatal).
func readComponentGlobals(configDir string, eval *starlarkpkg.Evaluator, c starlarkpkg.ComponentDecl, cl *loader.CompositeLoader) ([]string, gostarlark.StringDict) {
	merged := make([]string, len(c.After))
	copy(merged, c.After)
	if strings.HasPrefix(c.Name, "@") {
		// Registry component: load via CompositeLoader to get globals (pm_name, after, platforms).
		if cl == nil {
			return merged, nil
		}
		predeclared := eval.StdPredeclared()
		thread := &gostarlark.Thread{
			Name: c.Name,
			Load: func(t *gostarlark.Thread, module string) (gostarlark.StringDict, error) {
				return cl.Load(t, module, predeclared)
			},
		}
		globals, err := cl.Load(thread, c.Name, predeclared)
		if err != nil {
			fmt.Fprintln(os.Stderr, "warning: reading URL component globals:", err)
			return merged, nil
		}
		return mergeAfterFromGlobals(merged, globals, eval), globals
	}
	componentFile := filepath.Join(configDir, "components", c.Name, "init.star")
	if _, statErr := os.Lstat(componentFile); statErr != nil {
		return merged, nil
	}
	fileResult, readErr := eval.ReadComponentGlobals(componentFile, nil)
	if readErr != nil {
		fmt.Fprintln(os.Stderr, "warning: reading component globals:", readErr)
		return merged, nil
	}
	return mergeAfterFromGlobals(merged, fileResult.Globals, eval), fileResult.Globals
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
// after may be a list of strings or a callable(ctx) -> list[str].
func mergeAfterFromGlobals(existing []string, globals gostarlark.StringDict, eval *starlarkpkg.Evaluator) []string {
	afterVal, ok := globals["after"]
	if !ok {
		return existing
	}

	var afterNames []string
	switch v := afterVal.(type) {
	case *gostarlark.List:
		afterNames = make([]string, 0, v.Len())
		for i := 0; i < v.Len(); i++ {
			if s, ok := v.Index(i).(gostarlark.String); ok {
				afterNames = append(afterNames, string(s))
			}
		}
	case gostarlark.Callable:
		names, err := eval.CallAfterCallable(v)
		if err != nil {
			fmt.Fprintln(os.Stderr, "warning: after() callable failed:", err)
			return existing
		}
		afterNames = names
	default:
		return existing
	}

	seen := make(map[string]bool, len(existing))
	for _, e := range existing {
		seen[e] = true
	}
	result := existing
	for _, name := range afterNames {
		if !seen[name] {
			seen[name] = true
			result = append(result, name)
		}
	}
	return result
}

func newUpgradeCmd(gf *globalFlags) *cobra.Command {
	var cfg runConfig
	cmd := &cobra.Command{
		Use:   "upgrade [<component>...]",
		Short: "Run the upgrade phase set for all (or specified) components",
		RunE: func(_ *cobra.Command, args []string) error {
			configDir, err := resolveConfigDir(gf)
			if err != nil {
				return err
			}
			cfg.ConfigDir = configDir
			return runUpgradeWithPkgsLock(cfg, args)
		},
	}
	addLifecycleFlags(cmd, &cfg)
	addInstallFlags(cmd, &cfg)
	return cmd
}

// runUpgradeWithPkgsLock runs the upgrade phase set and writes updated package versions
// to pkgs.lock (shared components) and pkgs.local.lock (local components).
func runUpgradeWithPkgsLock(cfg runConfig, filter []string) error {
	order, pmReg, urlMap, err := loadComponentsWithDeps(cfg.ConfigDir, filter, true)
	if err != nil {
		return err
	}
	caller, err := runLifecyclePhaseSetOrderedWithCaller("upgrade", lifecycle.PhaseSetUpgrade, cfg, order, pmReg, urlMap)
	if err != nil {
		return err
	}
	if caller != nil && len(caller.pkgsPins) > 0 {
		_, localNames, loadErr := loadDeclaredNames(cfg.ConfigDir)
		if loadErr != nil {
			return loadErr
		}
		localSet := make(map[string]bool, len(localNames))
		for _, n := range localNames {
			localSet[n] = true
		}
		if err := appendPkgsLock(cfg.ConfigDir, caller.pkgsPins, localSet); err != nil {
			return err
		}
	}
	return nil
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

// runLifecyclePhaseSet executes a named phase set, loading components from init.star.
func runLifecyclePhaseSet(name string, phases []lifecycle.Phase, cfg runConfig, filter []string) error {
	// For uninstall, synthetic deps (PMs like brew/mise) should not run their
	// own uninstall hooks — they are only needed for ordering.
	runSyntheticHooks := name != "uninstall"
	order, pmReg, urlMap, err := loadComponentsWithDeps(cfg.ConfigDir, filter, runSyntheticHooks)
	if err != nil {
		return err
	}
	return runLifecyclePhaseSetOrdered(name, phases, cfg, order, pmReg, urlMap)
}

// runLifecyclePhaseSetOrdered executes a lifecycle phase set against a pre-computed
// component order, PMRegistry, and URL map. Used by apply to avoid redundant
// config reloads when install and uninstall phases share context.
func runLifecyclePhaseSetOrdered(name string, phases []lifecycle.Phase, cfg runConfig, order []lifecycle.ComponentID, pmReg *pkg.PMRegistry, urlMap map[string]string) error {
	_, err := runLifecyclePhaseSetOrderedWithCaller(name, phases, cfg, order, pmReg, urlMap)
	return err
}

// runLifecyclePhaseSetOrderedWithCaller is like runLifecyclePhaseSetOrdered but
// also returns the hook caller so callers can access pkgsPins after the run.
func runLifecyclePhaseSetOrderedWithCaller(name string, phases []lifecycle.Phase, cfg runConfig, order []lifecycle.ComponentID, pmReg *pkg.PMRegistry, urlMap map[string]string) (*starlarkHookCaller, error) {
	journalPath := filepath.Join(cfg.ConfigDir, "rollback.jsonl")
	var stack *rollback.Stack
	if !cfg.DryRun {
		var openErr error
		stack, openErr = rollback.Open(journalPath)
		if openErr != nil {
			return nil, fmt.Errorf("cannot open rollback journal: %w", openErr)
		}
		defer func() { _ = stack.Close() }()
	}

	w := tui.New(os.Stdout)
	defer func() { _ = w.Close() }()

	// Build hook caller with loader; replace directives come from deps.mod.
	caller := buildHookCaller(cfg)
	caller.urlComponents = urlMap

	statePath := filepath.Join(cfg.ConfigDir, "state.toml")
	sentinel := state.NewManager(statePath)
	runner := buildRunner(cfg, order, stack, sentinel, w, pmReg, caller)
	return caller, runner.RunPhaseSet(name, phases)
}

// isDir returns true if path exists and is a directory.
func isDir(path string) bool {
	fi, err := os.Stat(path)
	return err == nil && fi.IsDir()
}
