package cli

import (
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
	"github.com/spf13/cobra"
)

// installedLock is the TOML structure for installed.lock.
type installedLock struct {
	SchemaVersion int      `toml:"schema_version"`
	Components    []string `toml:"components"`
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

// writeInstalledLock atomically writes installed.lock with the given component names.
func writeInstalledLock(configDir string, components []string) error {
	sorted := make([]string, len(components))
	copy(sorted, components)
	sort.Strings(sorted)

	il := installedLock{
		SchemaVersion: 1,
		Components:    sorted,
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

// readPkgsLock reads <configDir>/<filename> as a lock.LockFile.
// Returns an empty LockFile (not an error) if the file is absent.
func readPkgsLock(configDir, filename string) (lock.LockFile, error) {
	path := filepath.Join(configDir, filename)
	var lf lock.LockFile
	if _, err := os.Lstat(path); os.IsNotExist(err) {
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

// computeToInstall returns the components to install: declared \ installed, in topo order.
func computeToInstall(allOrder []lifecycle.ComponentID, installedSet, scopeSet map[string]bool, scoped bool) []lifecycle.ComponentID {
	var toInstall []lifecycle.ComponentID
	for _, id := range allOrder {
		if scoped && !scopeSet[id] {
			continue
		}
		if !installedSet[id] {
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

// mergeInstalledAfterApply computes the new installed component list after a scoped apply.
func mergeInstalledAfterApply(installedSet map[string]bool, toInstall []lifecycle.ComponentID) []string {
	merged := make(map[string]bool, len(installedSet)+len(toInstall))
	for name := range installedSet {
		merged[name] = true
	}
	for _, id := range toInstall {
		merged[id] = true
	}
	result := make([]string, 0, len(merged))
	for name := range merged {
		result = append(result, name)
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

// computeNewInstalled returns the component list to write to installed.lock after apply.
func computeNewInstalled(scoped bool, allOrder []lifecycle.ComponentID, installedSet map[string]bool, toInstall []lifecycle.ComponentID) []string {
	if scoped {
		return mergeInstalledAfterApply(installedSet, toInstall)
	}
	newInstalled := make([]string, 0, len(allOrder))
	newInstalled = append(newInstalled, allOrder...)
	return newInstalled
}

// runApply implements the full reconcile logic for "meowctl apply".
func runApply(cfg runConfig, scopeFilter []string) error {
	allOrder, pmReg, urlMap, err := loadComponentsWithDeps(cfg.ConfigDir, nil, true)
	if err != nil {
		return err
	}

	declaredSet := make(map[string]bool, len(allOrder))
	for _, id := range allOrder {
		declaredSet[id] = true
	}

	il, err := readInstalledLock(cfg.ConfigDir)
	if err != nil {
		return err
	}
	installedSet := make(map[string]bool, len(il.Components))
	for _, name := range il.Components {
		installedSet[name] = true
	}

	scoped := len(scopeFilter) > 0
	scopeSet := make(map[string]bool, len(scopeFilter))
	for _, name := range scopeFilter {
		scopeSet[name] = true
	}

	toInstall := computeToInstall(allOrder, installedSet, scopeSet, scoped)

	var toUninstall []lifecycle.ComponentID
	if !scoped {
		toUninstall = computeToUninstall(allOrder, declaredSet, installedSet, il.Components)
	}

	if len(toInstall) == 0 && len(toUninstall) == 0 {
		fmt.Println("meowctl: nothing to do")
		return nil
	}

	if cfg.DryRun {
		return printApplyDryRun(toInstall, toUninstall)
	}

	installCaller, err := execApplyPhases(cfg, toInstall, toUninstall, pmReg, urlMap)
	if err != nil {
		return err
	}

	newInstalled := computeNewInstalled(scoped, allOrder, installedSet, toInstall)
	if err := writeInstalledLock(cfg.ConfigDir, newInstalled); err != nil {
		return err
	}

	return writePkgsLockAfterApply(cfg.ConfigDir, installCaller)
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
			OS:         goosToPlatformOS(runtime.GOOS),
			Distro:     distro,
			DistroLike: distroLike,
		},
	}
}
