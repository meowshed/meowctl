package cli

import (
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"runtime"
	"sort"
	"strings"

	"github.com/BurntSushi/toml"
	"github.com/meowshed/meowctl/internal/lifecycle"
	"github.com/meowshed/meowctl/internal/lock"
	"github.com/meowshed/meowctl/internal/pkg"
	"github.com/meowshed/meowctl/internal/rewrite"
	starlarkpkg "github.com/meowshed/meowctl/internal/starlark"
	"github.com/meowshed/meowctl/internal/state"
	"github.com/spf13/cobra"
)

// installedComponent records one installed component and the resolved module
// version it was installed from. Version is empty for bare/local components that
// belong to no module.
type installedComponent struct {
	Name    string `toml:"name"`
	Version string `toml:"version,omitempty"`
}

// installedLock is the TOML structure for installed.lock.
//
// schema_version 2 records a per-component resolved module version under
// [[installed]] so apply can detect when a module was bumped and re-link the
// affected components. The legacy schema_version 1 `components = [names...]`
// array is still read (versions treated as unknown) for backward compatibility.
type installedLock struct {
	SchemaVersion int                  `toml:"schema_version"`
	Components    []string             `toml:"components,omitempty"` // legacy v1 (read-only)
	Installed     []installedComponent `toml:"installed,omitempty"`  // v2
}

// versionMap returns a name → recorded-version map, transparently handling both
// the v2 [[installed]] form and the legacy v1 string-list form (version "").
func (il installedLock) versionMap() map[string]string {
	m := make(map[string]string, len(il.Installed)+len(il.Components))
	if len(il.Installed) > 0 {
		for _, c := range il.Installed {
			m[c.Name] = c.Version
		}
		return m
	}
	for _, n := range il.Components {
		m[n] = ""
	}
	return m
}

// names returns the installed component names, from either schema form.
func (il installedLock) names() []string {
	if len(il.Installed) > 0 {
		out := make([]string, len(il.Installed))
		for i, c := range il.Installed {
			out[i] = c.Name
		}
		return out
	}
	return il.Components
}

// readInstalledLock reads <configDir>/installed.lock.
// Returns an empty installedLock (not an error) if the file is absent.
func readInstalledLock(configDir string) (installedLock, error) {
	path := filepath.Join(configDir, configInstalledFile)
	var il installedLock
	if _, err := os.Lstat(path); os.IsNotExist(err) {
		return il, nil
	}
	if _, err := toml.DecodeFile(path, &il); err != nil { // #nosec G304
		return il, fmt.Errorf("read installed.lock: %w", err)
	}
	return il, nil
}

// writeInstalledLock atomically writes installed.lock (schema v2) with the given
// components and their resolved module versions, sorted by name.
func writeInstalledLock(configDir string, components []installedComponent) error {
	sorted := make([]installedComponent, len(components))
	copy(sorted, components)
	sort.Slice(sorted, func(i, j int) bool { return sorted[i].Name < sorted[j].Name })

	il := installedLock{
		SchemaVersion: 2,
		Installed:     sorted,
	}

	path := filepath.Join(configDir, configInstalledFile)
	tmpPath := path + ".tmp"

	f, err := os.OpenFile(tmpPath, os.O_CREATE|os.O_WRONLY|os.O_TRUNC, 0o600) // #nosec G304
	if err != nil {
		return fmt.Errorf("write installed.lock: create tmp: %w", err)
	}
	if err := toml.NewEncoder(f).Encode(il); err != nil {
		_ = f.Close()
		_ = os.Remove(tmpPath)
		return fmt.Errorf("write installed.lock: encode: %w", err)
	}
	if err := f.Close(); err != nil {
		_ = os.Remove(tmpPath)
		return fmt.Errorf("write installed.lock: close: %w", err)
	}
	if err := os.Rename(tmpPath, path); err != nil {
		_ = os.Remove(tmpPath)
		return fmt.Errorf("write installed.lock: rename: %w", err)
	}
	return nil
}

// moduleKeyFromComponentURL extracts the lock-file module key from a component's
// source reference. Registry components look like "@dotmeow//path" (key
// "dotmeow"); GitHub components like "github.com/o/r//path" (key
// "github.com/o/r"). Bare/local component names contain no "//" and yield
// themselves, which never matches a module entry.
func moduleKeyFromComponentURL(u string) string {
	s := strings.TrimPrefix(u, "@")
	if i := strings.Index(s, "//"); i >= 0 {
		s = s[:i]
	}
	return s
}

// moduleFingerprint returns a compact identity for a resolved module that changes
// whenever its content does: the semver version for registry deps, the commit SHA
// for GitHub deps, falling back to the SRI integrity hash.
func moduleFingerprint(e lock.ModuleEntry) string {
	switch {
	case e.Version != "":
		return e.Version
	case e.CommitSHA != "":
		return e.CommitSHA
	default:
		return e.Integrity
	}
}

// resolveComponentVersions maps each component (by logical name) to the
// fingerprint of the module it resolves from, read from deps.lock and
// deps.local.lock. Components with no module (bare/local) are absent from the map.
func resolveComponentVersions(configDir string, urlMap map[string]string) (map[string]string, error) {
	modules := map[string]lock.ModuleEntry{}
	for _, fn := range []string{configLockFile, configLocalLockFile} {
		lf, err := readPkgsLock(configDir, fn)
		if err != nil {
			return nil, err
		}
		for k, v := range lf.Modules {
			modules[k] = v
		}
	}
	versions := make(map[string]string, len(urlMap))
	for name, url := range urlMap {
		if e, ok := modules[moduleKeyFromComponentURL(url)]; ok {
			versions[name] = moduleFingerprint(e)
		}
	}
	return versions, nil
}

// computeStaleComponents returns the set of components whose resolved module
// version differs from what installed.lock recorded — i.e. a module was bumped
// (or sync'd) since the last apply. Every component belonging to a changed module
// is returned, including transitive ones not tracked in installed.lock directly,
// so that bumping an aggregate module re-links all of its config components.
func computeStaleComponents(allOrder []lifecycle.ComponentID, urlMap, recordedVersions, currentVersions map[string]string) map[string]bool {
	changedModules := make(map[string]bool)
	for name, recorded := range recordedVersions {
		current, tracked := currentVersions[name]
		if !tracked {
			continue // component no longer declared, or has no module
		}
		if recorded != current {
			changedModules[moduleKeyFromComponentURL(urlMap[name])] = true
		}
	}
	if len(changedModules) == 0 {
		return nil
	}
	stale := make(map[string]bool)
	for _, id := range allOrder {
		if changedModules[moduleKeyFromComponentURL(urlMap[id])] {
			stale[id] = true
		}
	}
	return stale
}

// clearStaleSentinels removes completion records for components whose module
// version changed, so the lifecycle runner re-executes (re-links) them this run
// instead of skipping them as already completed.
func clearStaleSentinels(configDir string, staleSet map[string]bool) error {
	if len(staleSet) == 0 {
		return nil
	}
	sm := state.NewManager(filepath.Join(configDir, configStateFile))
	for id := range staleSet {
		if err := sm.ClearComponent(id); err != nil {
			return fmt.Errorf("apply: clear stale component state %q: %w", id, err)
		}
	}
	return nil
}

// readPkgsLock reads <configDir>/<filename> as a lock.LockFile.
// Returns an empty LockFile (not an error) if the file is absent.
func readPkgsLock(configDir, filename string) (lock.LockFile, error) {
	path := filepath.Join(configDir, filename)
	var lf lock.LockFile
	if _, err := os.Lstat(path); errors.Is(err, os.ErrNotExist) {
		return lf, nil
	}
	if _, err := toml.DecodeFile(path, &lf); err != nil { // #nosec G304
		return lf, fmt.Errorf("read %s: %w", filename, err)
	}
	return lf, nil
}

// writePkgsLockFile atomically writes a lock.LockFile to <configDir>/<filename>.
func writePkgsLockFile(configDir, filename string, lf lock.LockFile) error {
	path := filepath.Join(configDir, filename)
	tmpPath := path + ".tmp"
	f, err := os.OpenFile(tmpPath, os.O_CREATE|os.O_WRONLY|os.O_TRUNC, 0o600) // #nosec G304
	if err != nil {
		return fmt.Errorf("write %s: create tmp: %w", filename, err)
	}
	if err := toml.NewEncoder(f).Encode(lf); err != nil {
		_ = f.Close()
		_ = os.Remove(tmpPath)
		return fmt.Errorf("write %s: encode: %w", filename, err)
	}
	if err := f.Close(); err != nil {
		_ = os.Remove(tmpPath)
		return fmt.Errorf("write %s: close: %w", filename, err)
	}
	if err := os.Rename(tmpPath, path); err != nil {
		_ = os.Remove(tmpPath)
		return fmt.Errorf("write %s: rename: %w", filename, err)
	}
	return nil
}

// appendPkgsLock merges pkgsPins into pkgs.lock (shared) and pkgs.local.lock (local).
// localSet contains the logical names of components declared in local.star.
func appendPkgsLock(configDir string, pkgsPins map[string][]starlarkpkg.PkgDecl, localSet map[string]bool) error {
	sharedLF, err := readPkgsLock(configDir, configPkgsLockFile)
	if err != nil {
		return err
	}
	localLF, err := readPkgsLock(configDir, configPkgsLocalLockFile)
	if err != nil {
		return err
	}

	sharedDirty, localDirty := false, false
	for compName, decls := range pkgsPins {
		isLocal := localSet[compName]
		for _, d := range decls {
			entry := lock.PackageEntry{Requested: d.Version, Installed: d.Version}
			if isLocal {
				if localLF.Packages == nil {
					localLF.Packages = make(map[string]lock.ManagerPackages)
				}
				if localLF.Packages[d.Manager] == nil {
					localLF.Packages[d.Manager] = make(lock.ManagerPackages)
				}
				localLF.Packages[d.Manager][d.Name] = entry
				localDirty = true
			} else {
				if sharedLF.Packages == nil {
					sharedLF.Packages = make(map[string]lock.ManagerPackages)
				}
				if sharedLF.Packages[d.Manager] == nil {
					sharedLF.Packages[d.Manager] = make(lock.ManagerPackages)
				}
				sharedLF.Packages[d.Manager][d.Name] = entry
				sharedDirty = true
			}
		}
	}

	if sharedDirty {
		if err := writePkgsLockFile(configDir, configPkgsLockFile, sharedLF); err != nil {
			return err
		}
	}
	if localDirty {
		if err := writePkgsLockFile(configDir, configPkgsLocalLockFile, localLF); err != nil {
			return err
		}
	}
	return nil
}

// newApplyCmd returns the "meowctl apply" command.
func newApplyCmd(gf *globalFlags) *cobra.Command {
	var cfg runConfig
	cmd := &cobra.Command{
		Use:   "apply [<component>...]",
		Short: "Reconcile installed components with the declared config",
		Long: `apply computes the diff between declared components (init.star + local.star)
and installed components (installed.lock), then installs missing components and
uninstalls removed components.

When component names are provided, apply scopes to those components only:
installs them if missing, never auto-uninstalls in scoped mode.`,
		RunE: func(_ *cobra.Command, args []string) error {
			configDir, err := resolveConfigDir(gf)
			if err != nil {
				return err
			}
			cfg.ConfigDir = configDir
			return runApply(cfg, args)
		},
	}
	addLifecycleFlags(cmd, &cfg)
	addInstallFlags(cmd, &cfg)
	return cmd
}

// buildScopeSet expands a scope filter to the dependency-closed set of component
// IDs it covers, so a scoped apply also (re)installs transitive deps of the named
// components. Returns nil when no filter is given.
func buildScopeSet(configDir string, scopeFilter []string) (map[string]bool, error) {
	if len(scopeFilter) == 0 {
		return nil, nil
	}
	scopedOrder, _, _, err := loadComponentsWithDeps(configDir, scopeFilter, true)
	if err != nil {
		return nil, err
	}
	scopeSet := make(map[string]bool, len(scopedOrder))
	for _, id := range scopedOrder {
		scopeSet[id] = true
	}
	return scopeSet, nil
}

// computeToInstall returns the components to install in topo order: those not yet
// installed (declared \ installed) plus any whose resolved module version changed
// (staleSet), so a module bump re-links its components.
func computeToInstall(allOrder []lifecycle.ComponentID, installedSet, scopeSet, staleSet map[string]bool, scoped bool) []lifecycle.ComponentID {
	var toInstall []lifecycle.ComponentID
	for _, id := range allOrder {
		if scoped && !scopeSet[id] {
			continue
		}
		if !installedSet[id] || staleSet[id] {
			toInstall = append(toInstall, id)
		}
	}
	return toInstall
}

// computeToUninstall returns the components to uninstall: installed \ declared, for full reconcile.
// allOrder contains only declared components (and their synthetic deps, which are also in declaredSet),
// so any installed component absent from declaredSet cannot be found by walking allOrder — we
// must walk installedComponents directly.
func computeToUninstall(_ []lifecycle.ComponentID, declaredSet, _ map[string]bool, installedComponents []string) []lifecycle.ComponentID {
	var toUninstall []lifecycle.ComponentID
	for _, name := range installedComponents {
		if !declaredSet[name] {
			toUninstall = append(toUninstall, name)
		}
	}
	return toUninstall
}

// mergeInstalledAfterApply computes the new installed component list after a
// scoped apply: the previously-installed components (with their recorded
// versions) plus the just-installed ones (with their current versions). Current
// versions win for any overlap so a scoped re-link refreshes the stored version.
func mergeInstalledAfterApply(recordedVersions map[string]string, toInstall []lifecycle.ComponentID, currentVersions map[string]string) []installedComponent {
	merged := make(map[string]string, len(recordedVersions)+len(toInstall))
	for name, v := range recordedVersions {
		merged[name] = v
	}
	for _, id := range toInstall {
		merged[id] = currentVersions[id]
	}
	result := make([]installedComponent, 0, len(merged))
	for name, v := range merged {
		result = append(result, installedComponent{Name: name, Version: v})
	}
	return result
}

// execApplyPhases runs the install and uninstall phase sets for the given component lists.
// It returns the install caller (for pkgsPins access) which may be nil if no installs ran.
func execApplyPhases(cfg runConfig, toInstall, toUninstall []lifecycle.ComponentID, pmReg *pkg.PMRegistry, urlMap map[string]string) (*starlarkHookCaller, error) {
	var installCaller *starlarkHookCaller
	if len(toInstall) > 0 {
		var err error
		installCaller, err = runLifecyclePhaseSetOrderedWithCaller("install", lifecycle.PhaseSetInstall, cfg, toInstall, pmReg, urlMap)
		if err != nil {
			return nil, err
		}
	}
	if len(toUninstall) > 0 {
		if err := runLifecyclePhaseSetOrdered("uninstall", lifecycle.PhaseSetUninstall, cfg, toUninstall, pmReg, urlMap); err != nil {
			return nil, err
		}
	}
	return installCaller, nil
}

// computeNewInstalled returns the components (with resolved versions) to write to
// installed.lock after apply.
func computeNewInstalled(scoped bool, allOrder []lifecycle.ComponentID, recordedVersions map[string]string, toInstall []lifecycle.ComponentID, currentVersions map[string]string) []installedComponent {
	if scoped {
		return mergeInstalledAfterApply(recordedVersions, toInstall, currentVersions)
	}
	newInstalled := make([]installedComponent, 0, len(allOrder))
	for _, id := range allOrder {
		newInstalled = append(newInstalled, installedComponent{Name: id, Version: currentVersions[id]})
	}
	return newInstalled
}

// runApply implements the full reconcile logic for "meowctl apply".
func runApply(cfg runConfig, scopeFilter []string) error {
	allOrder, pmReg, urlMap, err := loadComponentsWithDeps(cfg.ConfigDir, nil, true)
	if err != nil {
		return err
	}

	// declaredSet includes synthetic deps (e.g. mise, github) so they are
	// never treated as "installed but removed" by computeToUninstall.
	declaredSet := make(map[string]bool, len(allOrder))
	for _, id := range allOrder {
		declaredSet[id] = true
	}

	// explicitOrder contains only user-declared components (no synthetic deps).
	// This is what gets written to installed.lock so that auto-discovered PM
	// provider components are never tracked as independently installed/uninstalled.
	explicitOrder, _, _, err := loadComponentsWithDeps(cfg.ConfigDir, nil, false)
	if err != nil {
		return err
	}

	il, err := readInstalledLock(cfg.ConfigDir)
	if err != nil {
		return err
	}
	recordedVersions := il.versionMap()
	installedSet := make(map[string]bool, len(recordedVersions))
	for name := range recordedVersions {
		installedSet[name] = true
	}

	// Map every declared component to the fingerprint of the module it resolves
	// from, so apply can detect modules bumped since the last run.
	currentVersions, err := resolveComponentVersions(cfg.ConfigDir, urlMap)
	if err != nil {
		return err
	}
	staleSet := computeStaleComponents(allOrder, urlMap, recordedVersions, currentVersions)

	scoped := len(scopeFilter) > 0
	scopeSet, err := buildScopeSet(cfg.ConfigDir, scopeFilter)
	if err != nil {
		return err
	}

	toInstall := computeToInstall(allOrder, installedSet, scopeSet, staleSet, scoped)

	var toUninstall []lifecycle.ComponentID
	if !scoped {
		toUninstall = computeToUninstall(allOrder, declaredSet, installedSet, il.names())
	}

	// Components to uninstall may no longer be declared, so their PM handlers
	// may be absent from pmReg. Merge any pre-loaded registry from the caller
	// (populated before a destructive config edit such as runRemove).
	supplementPMRegistry(cfg, pmReg, urlMap)

	if len(toInstall) == 0 && len(toUninstall) == 0 {
		fmt.Println("meowctl: nothing to do")
		return nil
	}

	if cfg.DryRun {
		return printApplyDryRun(toInstall, toUninstall)
	}

	// A changed module version means the component's on-disk artifacts still point
	// at the old version; clear its completion sentinels so the runner re-links it
	// rather than skipping it as "already installed".
	if err := clearStaleSentinels(cfg.ConfigDir, staleSet); err != nil {
		return err
	}

	installCaller, err := execApplyPhases(cfg, toInstall, toUninstall, pmReg, urlMap)
	if err != nil {
		return err
	}

	newInstalled := computeNewInstalled(scoped, explicitOrder, recordedVersions, toInstall, currentVersions)
	if err := writeInstalledLock(cfg.ConfigDir, newInstalled); err != nil {
		return err
	}

	return writePkgsLockAfterApply(cfg.ConfigDir, installCaller)
}

// supplementPMRegistry merges any pre-loaded PM handlers and URL map entries
// from cfg into pmReg / urlMap. This is necessary when components are removed
// from the config before reconcile runs: their PM provider deps (e.g. node for
// npm) are no longer discovered in the main load, yet their packages still need
// to be uninstalled. The caller (runRemove) loads the dep graph before editing
// local.star and stores it in cfg.ExtraPMReg / cfg.ExtraURLMap.
func supplementPMRegistry(cfg runConfig, pmReg *pkg.PMRegistry, urlMap map[lifecycle.ComponentID]string) {
	pmReg.Merge(cfg.ExtraPMReg)
	for k, v := range cfg.ExtraURLMap {
		if _, exists := urlMap[k]; !exists {
			urlMap[k] = v
		}
	}
}

// writePkgsLockAfterApply writes pkgs.lock / pkgs.local.lock after a successful apply.
// It is a no-op when installCaller is nil or has no package pins.
func writePkgsLockAfterApply(configDir string, installCaller *starlarkHookCaller) error {
	if installCaller == nil || len(installCaller.pkgsPins) == 0 {
		return nil
	}
	_, localNames, err := loadDeclaredNames(configDir)
	if err != nil {
		return err
	}
	localSet := make(map[string]bool, len(localNames))
	for _, n := range localNames {
		localSet[n] = true
	}
	return appendPkgsLock(configDir, installCaller.pkgsPins, localSet)
}

// printApplyDryRun prints the install/uninstall plan and returns nil.
func printApplyDryRun(toInstall, toUninstall []lifecycle.ComponentID) error {
	if len(toInstall) > 0 {
		fmt.Println("will install:")
		for _, id := range toInstall {
			fmt.Printf("  + %s\n", id)
		}
	}
	if len(toUninstall) > 0 {
		fmt.Println("will uninstall:")
		for _, id := range toUninstall {
			fmt.Printf("  - %s\n", id)
		}
	}
	return nil
}

// newAddCmd returns the "meowctl add" command.
func newAddCmd(gf *globalFlags) *cobra.Command {
	var cfg runConfig
	cmd := &cobra.Command{
		Use:   "add <component> [<component>...]",
		Short: "Add component(s) to local.star and install them",
		Args:  cobra.MinimumNArgs(1),
		RunE: func(_ *cobra.Command, args []string) error {
			configDir, err := resolveConfigDir(gf)
			if err != nil {
				return err
			}
			cfg.ConfigDir = configDir
			return runAdd(cfg, args)
		},
	}
	addLifecycleFlags(cmd, &cfg)
	addInstallFlags(cmd, &cfg)
	return cmd
}

func runAdd(cfg runConfig, names []string) error {
	initNames, localNames, err := loadDeclaredNames(cfg.ConfigDir)
	if err != nil {
		return err
	}

	initSet := make(map[string]bool, len(initNames))
	for _, n := range initNames {
		initSet[n] = true
	}
	localSet := make(map[string]bool, len(localNames))
	for _, n := range localNames {
		localSet[n] = true
	}

	localPath := filepath.Join(cfg.ConfigDir, configLocalFile)
	var toApply []string

	for _, name := range names {
		if initSet[name] || localSet[name] {
			fmt.Printf("meowctl: %s already declared, skipping\n", name)
			continue
		}
		// Check that the component's module is declared in a modfile.
		if err := checkModuleDeclared(cfg.ConfigDir, name); err != nil {
			return err
		}
		if !cfg.DryRun {
			if err := rewrite.AppendComponent(localPath, name); err != nil {
				return fmt.Errorf("add: append to local.star: %w", err)
			}
		}
		toApply = append(toApply, name)
	}

	if len(toApply) == 0 {
		return nil
	}

	return runApply(cfg, toApply)
}

// newRemoveCmd returns the "meowctl remove" command.
func newRemoveCmd(gf *globalFlags) *cobra.Command {
	var cfg runConfig
	cmd := &cobra.Command{
		Use:   "remove <component> [<component>...]",
		Short: "Remove component(s) from local.star and uninstall them",
		Args:  cobra.MinimumNArgs(1),
		RunE: func(_ *cobra.Command, args []string) error {
			configDir, err := resolveConfigDir(gf)
			if err != nil {
				return err
			}
			cfg.ConfigDir = configDir
			return runRemove(cfg, args)
		},
	}
	addLifecycleFlags(cmd, &cfg)
	return cmd
}

func runRemove(cfg runConfig, names []string) error {
	initNames, localNames, err := loadDeclaredNames(cfg.ConfigDir)
	if err != nil {
		return err
	}

	initSet := make(map[string]bool, len(initNames))
	for _, n := range initNames {
		initSet[n] = true
	}
	localSet := make(map[string]bool, len(localNames))
	for _, n := range localNames {
		localSet[n] = true
	}

	localPath := filepath.Join(cfg.ConfigDir, configLocalFile)
	var toRemove []string

	for _, name := range names {
		if initSet[name] {
			return exitErrorf(ExitUsage, "%s is declared in init.star — edit that file directly", name)
		}
		if !localSet[name] {
			fmt.Printf("meowctl: %s not declared, skipping\n", name)
			continue
		}
		toRemove = append(toRemove, name)
	}

	if len(toRemove) == 0 {
		fmt.Println("meowctl: nothing to do")
		return nil
	}

	if cfg.DryRun {
		fmt.Println("will uninstall:")
		for _, name := range toRemove {
			fmt.Printf("  - %s\n", name)
		}
		return nil
	}

	// Load the full dep graph before rewriting local.star so that ALL PM
	// handlers currently declared (including transitive deps like node→npm,
	// ruby→gem) are captured in cfg.ExtraPMReg. After the rewrite, removed
	// components and their PM providers are no longer discovered by runApply,
	// which would cause "no handler registered" errors during uninstall.
	_, extraReg, extraURLMap, err := loadComponentsWithDeps(cfg.ConfigDir, nil, false)
	if err == nil {
		cfg.ExtraPMReg = extraReg
		cfg.ExtraURLMap = extraURLMap
	}

	for _, name := range toRemove {
		if err := rewrite.RemoveComponent(localPath, name); err != nil {
			return fmt.Errorf("remove: remove from local.star: %w", err)
		}
	}

	// Full reconcile — apply will diff and uninstall the removed component(s).
	return runApply(cfg, nil)
}

// checkModuleDeclared verifies that the module referenced by a component name
// is declared in deps.mod or deps.local.mod. Component names of the form
// "@modname//path/to/component" carry an explicit module; bare names (no "@")
// are local and require no module declaration.
func checkModuleDeclared(configDir, componentName string) error {
	if !strings.HasPrefix(componentName, "@") {
		return nil // bare local component — no module required
	}
	// Extract module name: "@modname//..." → "modname"
	rest := componentName[1:]
	slash := strings.Index(rest, "//")
	if slash < 0 {
		return nil // malformed — let the evaluator catch it
	}
	moduleName := rest[:slash]

	if depExistsInEitherModfile(configDir, moduleName) {
		return nil
	}
	return fmt.Errorf("module %q not found in deps.mod or deps.local.mod — run 'meowctl dep add %s' first", moduleName, moduleName)
}

// depExistsInEitherModfile returns true if name exists in deps.mod or deps.local.mod.
func depExistsInEitherModfile(configDir, name string) bool {
	if depExistsInFile(filepath.Join(configDir, configModFile), name) {
		return true
	}
	return depExistsInFile(filepath.Join(configDir, configLocalModFile), name)
}

// without expanding transitive deps. Returns separate slices for each file.
func loadDeclaredNames(configDir string) (initNames []string, localNames []string, err error) {
	ev := makeEvaluator()

	starPath := filepath.Join(configDir, configEntryFile)
	result, execErr := ev.ExecFile(starPath, nil, nil, nil)
	if execErr != nil {
		return nil, nil, exitErrorf(ExitConfig, "load init.star: %v", execErr)
	}
	for _, c := range result.Declarations.Components {
		initNames = append(initNames, c.LogicalName())
	}

	localPath := filepath.Join(configDir, configLocalFile)
	if _, lstatErr := os.Lstat(localPath); os.IsNotExist(lstatErr) {
		return initNames, nil, nil
	}
	localResult, localExecErr := ev.ExecFile(localPath, nil, nil, nil)
	if localExecErr != nil {
		return nil, nil, exitErrorf(ExitConfig, "load local.star: %v", localExecErr)
	}
	for _, c := range localResult.Declarations.Components {
		localNames = append(localNames, c.LogicalName())
	}
	return initNames, localNames, nil
}

// makeEvaluator creates a fresh starlark Evaluator for the current platform.
func makeEvaluator() *starlarkpkg.Evaluator {
	distro, distroLike := "", ""
	if runtime.GOOS == "linux" {
		distro, distroLike = linuxDistroInfo()
	}
	return &starlarkpkg.Evaluator{
		Platform: starlarkpkg.PlatformInfo{
			OS:         goosToPlatformOS(),
			Distro:     distro,
			DistroLike: distroLike,
		},
	}
}
