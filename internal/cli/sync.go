package cli

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/meowshed/meowctl/internal/lifecycle"
	"github.com/meowshed/meowctl/internal/lock"
	"github.com/meowshed/meowctl/internal/modfile"
	"github.com/meowshed/meowctl/internal/starlark/loader"
)

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

// runSync implements "meowctl dep sync".
// It syncs both deps.mod → deps.lock and deps.local.mod → deps.local.lock (when present).
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

	reportSyncResult(oldLock.Modules, result)

	// Sync deps.local.mod → deps.local.lock if the local modfile exists.
	localChanged, err := runSyncLocal(configDir)
	if err != nil {
		return err
	}

	if len(changed) == 0 && len(result.ReplacedPaths) == 0 && !localChanged {
		fmt.Println("meowctl: all modules up to date")
		return nil
	}

	if len(changed) == 0 && !localChanged {
		return nil
	}

	fmt.Printf("meowctl: re-running install for %d changed module(s)\n", len(changed))
	cfg := runConfig{ConfigDir: configDir}
	return runLifecyclePhaseSet("install", lifecycle.PhaseSetInstall, cfg, nil)
}

// runSyncLocal syncs deps.local.mod → deps.local.lock when the local modfile exists.
// Returns true if any local modules changed, false if nothing changed or file absent.
func runSyncLocal(configDir string) (bool, error) {
	localModPath := filepath.Join(configDir, configLocalModFile)
	if _, err := os.Stat(localModPath); os.IsNotExist(err) {
		return false, nil
	}

	mf, err := modfile.Parse(localModPath)
	if err != nil {
		return false, fmt.Errorf("sync local: %w", err)
	}

	cd, err := cacheDir()
	if err != nil {
		return false, err
	}
	localLockPath := filepath.Join(configDir, configLocalLockFile)
	rl := &loader.RegistryLoader{
		CacheDir: cd,
		LockPath: localLockPath,
	}

	oldLock, err := lock.Read(localLockPath)
	if err != nil {
		return false, fmt.Errorf("sync local: read old lock: %w", err)
	}

	deps, replaces := modfileAdapters(mf)
	result, err := rl.SyncModules(deps, replaces)
	if err != nil {
		return false, fmt.Errorf("sync local: %w", err)
	}

	changed := changedModules(oldLock.Modules, result.Resolved)
	reportSyncResult(oldLock.Modules, result)
	return len(changed) > 0, nil
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
		deps[i] = loader.ModfileDep{Name: d.Name, Version: d.Version, Source: d.Source}
	}
	replaces := make([]loader.ModfileReplace, len(mf.Replace))
	for i, r := range mf.Replace {
		replaces[i] = loader.ModfileReplace{Name: r.Name, Path: r.Path, Source: r.Source}
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
