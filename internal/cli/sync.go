package cli

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"text/tabwriter"

	"github.com/meowshed/meowctl/internal/lifecycle"
	"github.com/meowshed/meowctl/internal/lock"
	"github.com/meowshed/meowctl/internal/modfile"
	"github.com/meowshed/meowctl/internal/rewrite"
	"github.com/meowshed/meowctl/internal/starlark/loader"
	"github.com/spf13/cobra"
)

// newSyncCmd returns the "meowctl sync" command.
// sync reads deps.mod, resolves all deps against the registry, downloads
// changed tarballs, updates the lock file, and re-runs install for any
// component whose module version changed.
func newSyncCmd(gf *globalFlags) *cobra.Command {
	return &cobra.Command{
		Use:   "sync",
		Short: "Resolve and download modules declared in deps.mod",
		Long: `sync reads deps.mod, resolves all dep() declarations against the registry,
downloads any changed tarballs, updates deps.lock, and re-runs the install
phase for every component whose module version changed.

replace() directives are honoured: replaced modules are read from the local
path and are never fetched from the registry. A replace() with a non-existent
local path is a hard error.`,
		Args: cobra.NoArgs,
		RunE: func(_ *cobra.Command, _ []string) error {
			configDir, err := resolveConfigDir(gf)
			if err != nil {
				return err
			}
			return runSync(configDir)
		},
	}
}

// newGetCmd returns the "meowctl get" command.
// get rewrites a dep version in deps.mod and refreshes the lock entry.
func newGetCmd(gf *globalFlags) *cobra.Command {
	return &cobra.Command{
		Use:   "get <module>[@<version>]",
		Short: "Add or update a module dependency in deps.mod",
		Long: `get rewrites the dep() version for <module> in deps.mod and updates
the lock file. Use @latest to resolve the newest available version.

Replaced modules (replace() directives) are skipped with a warning.`,
		Args: cobra.ExactArgs(1),
		RunE: func(_ *cobra.Command, args []string) error {
			configDir, err := resolveConfigDir(gf)
			if err != nil {
				return err
			}
			return runGet(configDir, args[0])
		},
	}
}

// newOutdatedCmd returns the "meowctl outdated" command.
// outdated prints a table of deps that have newer versions available.
func newOutdatedCmd(gf *globalFlags) *cobra.Command {
	return &cobra.Command{
		Use:   "outdated",
		Short: "List module dependencies with available upgrades",
		Long: `outdated reads deps.lock and queries the registry for the latest version
of each module. It prints a table of deps where a newer version is available.

No files are modified. Replaced modules are skipped with a warning.`,
		Args: cobra.NoArgs,
		RunE: func(_ *cobra.Command, _ []string) error {
			configDir, err := resolveConfigDir(gf)
			if err != nil {
				return err
			}
			return runOutdated(configDir)
		},
	}
}

// cacheDir returns the module cache directory (~/.cache/meowctl/modules).
func cacheDir() (string, error) {
	home, err := os.UserHomeDir()
	if err != nil {
		return "", fmt.Errorf("resolve home: %w", err)
	}
	return filepath.Join(home, ".cache", "meowctl", "modules"), nil
}

// newRegistryLoader builds a RegistryLoader for the given config directory.
func newRegistryLoader(configDir string) (*loader.RegistryLoader, error) {
	cd, err := cacheDir()
	if err != nil {
		return nil, err
	}
	return &loader.RegistryLoader{
		CacheDir: cd,
		LockPath: filepath.Join(configDir, configLockFile),
	}, nil
}

// runSync implements "meowctl sync".
func runSync(configDir string) error {
	mf, rl, oldLock, err := syncPrepare(configDir)
	if err != nil {
		return err
	}

	deps, replaces := modfileAdapters(mf)
	result, err := rl.SyncModules(deps, replaces)
	if err != nil {
		return fmt.Errorf("sync: %w", err)
	}

	changed := changedModules(oldLock.Modules, result.Resolved)
	if len(changed) == 0 && len(result.ReplacedPaths) == 0 {
		fmt.Println("meowctl: all modules up to date")
		return nil
	}

	reportSyncResult(oldLock.Modules, result)

	if len(changed) == 0 {
		return nil
	}

	fmt.Printf("meowctl: re-running install for %d changed module(s)\n", len(changed))
	cfg := runConfig{ConfigDir: configDir}
	return runLifecyclePhaseSet("install", lifecycle.PhaseSetInstall, cfg, nil)
}

// syncPrepare loads the modfile, registry loader, and old lock for runSync.
func syncPrepare(configDir string) (*modfile.ModFile, *loader.RegistryLoader, *lock.LockFile, error) {
	modPath := filepath.Join(configDir, configModFile)
	mf, err := modfile.Parse(modPath)
	if err != nil {
		return nil, nil, nil, fmt.Errorf("sync: %w", err)
	}
	rl, err := newRegistryLoader(configDir)
	if err != nil {
		return nil, nil, nil, err
	}
	lockPath := filepath.Join(configDir, configLockFile)
	oldLock, err := lock.Read(lockPath)
	if err != nil {
		return nil, nil, nil, fmt.Errorf("sync: read old lock: %w", err)
	}
	return mf, rl, oldLock, nil
}

// modfileAdapters converts modfile dep/replace slices to loader adapter types.
func modfileAdapters(mf *modfile.ModFile) ([]loader.ModfileDep, []loader.ModfileReplace) {
	deps := make([]loader.ModfileDep, len(mf.Deps))
	for i, d := range mf.Deps {
		deps[i] = loader.ModfileDep{Name: d.Name, Version: d.Version}
	}
	replaces := make([]loader.ModfileReplace, len(mf.Replace))
	for i, r := range mf.Replace {
		replaces[i] = loader.ModfileReplace{Module: r.Module, Path: r.Path}
	}
	return deps, replaces
}

// reportSyncResult prints a summary of resolved and replaced modules.
func reportSyncResult(oldModules map[string]lock.ModuleEntry, result *loader.SyncResult) {
	for mod, ver := range result.Resolved {
		old := ""
		if e, ok := oldModules[mod]; ok {
			old = e.Version
		}
		if old != "" && old != ver {
			fmt.Printf("  updated %s: %s → %s\n", mod, old, ver)
		} else if old == "" {
			fmt.Printf("  added   %s@%s\n", mod, ver)
		}
	}
	for mod, path := range result.ReplacedPaths {
		fmt.Fprintf(os.Stderr, "meowctl: module %q replaced by local path %q\n", mod, path)
	}
}

// runGet implements "meowctl get <module>[@<version>]".
func runGet(configDir, arg string) error {
	modName, version := splitGetArg(arg)
	if modName == "" {
		return exitErrorf(ExitUsage, "get: invalid argument %q — expected <module> or <module>@<version>", arg)
	}

	modPath := filepath.Join(configDir, configModFile)
	mf, err := modfile.Parse(modPath)
	if err != nil {
		return fmt.Errorf("get: %w", err)
	}

	// Check if module is replaced — skip with warning.
	for _, r := range mf.Replace {
		if r.Module == modName {
			fmt.Fprintf(os.Stderr, "meowctl: warning: module %q has an active replace() directive — skipping get\n", modName)
			return nil
		}
	}

	rl, err := newRegistryLoader(configDir)
	if err != nil {
		return err
	}

	// Resolve version if "latest" or empty.
	if version == "" || version == "latest" {
		latest, latestErr := rl.LatestVersion(modName)
		if latestErr != nil {
			return fmt.Errorf("get: resolve latest for %q: %w", modName, latestErr)
		}
		version = latest
	}

	// Rewrite the dep version in deps.mod.
	if err := rewrite.SetDepVersion(modPath, modName, version); err != nil {
		return fmt.Errorf("get: %w", err)
	}

	// Re-parse to get updated dep list and re-sync lock entry.
	mf, err = modfile.Parse(modPath)
	if err != nil {
		return fmt.Errorf("get: re-parse after rewrite: %w", err)
	}
	deps, replaces := modfileAdapters(mf)
	if _, err := rl.SyncModules(deps, replaces); err != nil {
		return fmt.Errorf("get: sync lock: %w", err)
	}

	fmt.Printf("meowctl: updated %s to %s\n", modName, version)
	return nil
}

// runOutdated implements "meowctl outdated".
func runOutdated(configDir string) error {
	modPath := filepath.Join(configDir, configModFile)
	mf, err := modfile.Parse(modPath)
	if err != nil {
		return fmt.Errorf("outdated: %w", err)
	}

	lockPath := filepath.Join(configDir, configLockFile)
	lf, err := lock.Read(lockPath)
	if err != nil {
		return fmt.Errorf("outdated: read lock: %w", err)
	}

	rl, err := newRegistryLoader(configDir)
	if err != nil {
		return err
	}

	// Build replace set for fast lookup.
	replaceSet := make(map[string]bool, len(mf.Replace))
	for _, r := range mf.Replace {
		replaceSet[r.Module] = true
	}

	type outdatedRow struct {
		module  string
		current string
		latest  string
	}
	var rows []outdatedRow

	for _, dep := range mf.Deps {
		modName := dep.Name
		if replaceSet[modName] {
			fmt.Fprintf(os.Stderr, "meowctl: warning: module %q has an active replace() directive — skipping\n", modName)
			continue
		}

		current := ""
		if e, ok := lf.Modules[modName]; ok {
			current = e.Version
		}

		latest, err := rl.LatestVersion(modName)
		if err != nil {
			fmt.Fprintf(os.Stderr, "meowctl: warning: could not fetch latest version for %q: %v\n", modName, err)
			continue
		}

		if latest != current {
			rows = append(rows, outdatedRow{module: modName, current: current, latest: latest})
		}
	}

	if len(rows) == 0 {
		fmt.Println("meowctl: all modules up to date")
		return nil
	}

	w := tabwriter.NewWriter(os.Stdout, 0, 0, 2, ' ', 0)
	if _, err := fmt.Fprintln(w, "MODULE\tCURRENT\tLATEST"); err != nil {
		return fmt.Errorf("outdated: write header: %w", err)
	}
	for _, row := range rows {
		if _, err := fmt.Fprintf(w, "%s\t%s\t%s\n", row.module, row.current, row.latest); err != nil {
			return fmt.Errorf("outdated: write row: %w", err)
		}
	}
	return w.Flush()
}

// splitGetArg splits "module@version" into (module, version).
// If no @ is present, version is "".
func splitGetArg(arg string) (module, version string) {
	if idx := strings.LastIndex(arg, "@"); idx > 0 {
		return arg[:idx], arg[idx+1:]
	}
	return arg, ""
}

// changedModules returns the set of module names whose version differs between
// the old lock and the freshly resolved map.
func changedModules(old map[string]lock.ModuleEntry, resolved map[string]string) map[string]bool {
	changed := make(map[string]bool)
	for mod, newVer := range resolved {
		if e, ok := old[mod]; !ok || e.Version != newVer {
			changed[mod] = true
		}
	}
	return changed
}
