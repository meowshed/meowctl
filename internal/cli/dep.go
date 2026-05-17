package cli

import (
	"encoding/json"
	"errors"
	"fmt"
	"os"
	"path/filepath"
	"regexp"
	"sort"
	"strings"
	"text/tabwriter"

	"github.com/spf13/cobra"

	"github.com/meowshed/meowctl/internal/lock"
	"github.com/meowshed/meowctl/internal/modfile"
)

// newDepCmd creates the "dep" command group.
func newDepCmd(gf *globalFlags) *cobra.Command {
	cmd := &cobra.Command{
		Use:   "dep",
		Short: "Manage module dependencies",
	}
	cmd.AddCommand(
		newDepSyncCmd(gf),
		newDepListCmd(gf),
		newDepAddCmd(gf),
		newDepRemoveCmd(gf),
		newDepUpgradeCmd(gf),
		newDepTidyCmd(gf),
	)
	return cmd
}

// newDepSyncCmd implements "meowctl dep sync".
func newDepSyncCmd(gf *globalFlags) *cobra.Command {
	return &cobra.Command{
		Use:   "sync",
		Short: "Sync deps.mod and deps.local.mod to their lock files",
		Args:  cobra.NoArgs,
		RunE: func(_ *cobra.Command, _ []string) error {
			configDir, err := resolveConfigDir(gf)
			if err != nil {
				return err
			}
			return runSync(configDir)
		},
	}
}

// depEntry is a single row for "dep list".
type depEntry struct {
	Name    string `json:"name"`
	Version string `json:"version,omitempty"`
	Source  string `json:"source,omitempty"`
	Locked  string `json:"locked,omitempty"`
	Local   bool   `json:"local"`
}

// newDepListCmd implements "meowctl dep list".
func newDepListCmd(gf *globalFlags) *cobra.Command {
	var jsonOut bool
	cmd := &cobra.Command{
		Use:   "list",
		Short: "List declared dependencies",
		RunE: func(cmd *cobra.Command, _ []string) error {
			configDir, err := resolveConfigDir(gf)
			if err != nil {
				return err
			}
			return runDepList(cmd, configDir, jsonOut)
		},
	}
	cmd.Flags().BoolVar(&jsonOut, "json", false, "Output as JSON")
	return cmd
}

func runDepList(cmd *cobra.Command, configDir string, jsonOut bool) error {
	entries := collectDepEntries(configDir)

	if jsonOut {
		enc := json.NewEncoder(cmd.OutOrStdout())
		enc.SetIndent("", "  ")
		return enc.Encode(entries)
	}

	w := tabwriter.NewWriter(cmd.OutOrStdout(), 0, 0, 2, ' ', 0)
	if _, err := fmt.Fprint(w, "NAME\tSOURCE/VERSION\tLOCKED\tLOCAL\n"); err != nil {
		return err
	}
	for _, e := range entries {
		sv := e.Version
		if sv == "" {
			sv = e.Source
		}
		local := ""
		if e.Local {
			local = "yes"
		}
		if _, err := fmt.Fprintf(w, "%s\t%s\t%s\t%s\n", e.Name, sv, e.Locked, local); err != nil {
			return err
		}
	}
	return w.Flush()
}

func collectDepEntries(configDir string) []depEntry {
	var entries []depEntry
	entries = append(entries, loadDepEntries(
		filepath.Join(configDir, configModFile),
		filepath.Join(configDir, configLockFile),
		false,
	)...)
	entries = append(entries, loadDepEntries(
		filepath.Join(configDir, configLocalModFile),
		filepath.Join(configDir, configLocalLockFile),
		true,
	)...)
	return entries
}

func loadDepEntries(modPath, lockPath string, local bool) []depEntry {
	mf, err := modfile.Parse(modPath)
	if err != nil {
		return nil
	}
	lf, _ := lock.Read(lockPath)
	sorted := make([]modfile.DepDecl, len(mf.Deps))
	copy(sorted, mf.Deps)
	sort.Slice(sorted, func(i, j int) bool { return sorted[i].Name < sorted[j].Name })
	var out []depEntry
	for _, d := range sorted {
		e := depEntry{Name: d.Name, Version: d.Version, Source: d.Source, Local: local}
		if lf != nil {
			if me, ok := lf.Modules[d.Name]; ok {
				if me.Version != "" {
					e.Locked = me.Version
				} else if me.CommitSHA != "" {
					e.Locked = me.CommitSHA
				}
			}
		}
		out = append(out, e)
	}
	return out
}

// newDepAddCmd implements "meowctl dep add <name> [--version <v>] [--source <s>] [--local]".
func newDepAddCmd(gf *globalFlags) *cobra.Command {
	var version, source string
	var local bool
	cmd := &cobra.Command{
		Use:   "add <name>",
		Short: "Add a dependency to deps.mod (or deps.local.mod with --local)",
		Args:  cobra.ExactArgs(1),
		RunE: func(_ *cobra.Command, args []string) error {
			configDir, err := resolveConfigDir(gf)
			if err != nil {
				return err
			}
			return runDepAdd(configDir, args[0], version, source, local)
		},
	}
	cmd.Flags().StringVar(&version, "version", "", "Registry version to pin")
	cmd.Flags().StringVar(&source, "source", "", "GitHub source ref (e.g. github:owner/repo@v1.0.0)")
	cmd.Flags().BoolVar(&local, "local", false, "Add to deps.local.mod instead of deps.mod")
	return cmd
}

func runDepAdd(configDir, name, version, source string, local bool) error {
	switch {
	case version == "" && source == "":
		return fmt.Errorf("dep add: exactly one of --version or --source is required")
	case version != "" && source != "":
		return fmt.Errorf("dep add: --version and --source are mutually exclusive")
	}

	modPath := filepath.Join(configDir, configModFile)
	if local {
		modPath = filepath.Join(configDir, configLocalModFile)
	}

	mf, err := parseOrEmpty(modPath)
	if err != nil {
		return err
	}

	// Duplicate check.
	for _, d := range mf.Deps {
		if d.Name == name {
			// Same version/source → no-op.
			if d.Version == version && d.Source == source {
				fmt.Printf("meowctl: dep %q already declared (no-op)\n", name)
				return nil
			}
			return fmt.Errorf("dep add: %q already declared with different version/source — run 'meowctl dep remove %s' first", name, name)
		}
	}

	mf.Deps = append(mf.Deps, modfile.DepDecl{Name: name, Version: version, Source: source})
	if err := modfile.Write(modPath, mf); err != nil {
		return fmt.Errorf("dep add: write modfile: %w", err)
	}
	fmt.Printf("meowctl: added dep %q\n", name)
	return runSync(configDir)
}

// newDepRemoveCmd implements "meowctl dep remove <name>".
func newDepRemoveCmd(gf *globalFlags) *cobra.Command {
	return &cobra.Command{
		Use:   "remove <name>",
		Short: "Remove a dependency from deps.mod or deps.local.mod",
		Args:  cobra.ExactArgs(1),
		RunE: func(_ *cobra.Command, args []string) error {
			configDir, err := resolveConfigDir(gf)
			if err != nil {
				return err
			}
			return runDepRemove(configDir, args[0])
		},
	}
}

func runDepRemove(configDir, name string) error {
	sharedModPath := filepath.Join(configDir, configModFile)
	localModPath := filepath.Join(configDir, configLocalModFile)

	inShared := depExistsInFile(sharedModPath, name)
	inLocal := depExistsInFile(localModPath, name)

	switch {
	case inShared && inLocal:
		return fmt.Errorf("dep remove: %q found in both deps.mod and deps.local.mod — remove from one file manually first", name)
	case !inShared && !inLocal:
		return fmt.Errorf("dep remove: %q not found in any modfile", name)
	}

	modPath := sharedModPath
	if inLocal {
		modPath = localModPath
	}

	mf, err := modfile.Parse(modPath)
	if err != nil {
		return fmt.Errorf("dep remove: %w", err)
	}

	filtered := mf.Deps[:0]
	for _, d := range mf.Deps {
		if d.Name != name {
			filtered = append(filtered, d)
		}
	}
	mf.Deps = filtered

	if err := modfile.Write(modPath, mf); err != nil {
		return fmt.Errorf("dep remove: write modfile: %w", err)
	}
	fmt.Printf("meowctl: removed dep %q\n", name)
	return runSync(configDir)
}

// depExistsInFile returns true if a dep with the given name exists in the modfile at path.
func depExistsInFile(path, name string) bool {
	mf, err := modfile.Parse(path)
	if err != nil {
		return false
	}
	for _, d := range mf.Deps {
		if d.Name == name {
			return true
		}
	}
	return false
}

// parseOrEmpty parses the modfile at path or returns an empty ModFile if the file doesn't exist.
func parseOrEmpty(path string) (*modfile.ModFile, error) {
	mf, err := modfile.Parse(path)
	if errors.Is(err, os.ErrNotExist) {
		return &modfile.ModFile{}, nil
	}
	if err != nil {
		return nil, err
	}
	return mf, nil
}

// newDepUpgradeCmd implements "meowctl dep upgrade [<module>...] [--dry-run]".
func newDepUpgradeCmd(gf *globalFlags) *cobra.Command {
	var dryRun bool
	cmd := &cobra.Command{
		Use:   "upgrade [<module>...]",
		Short: "Upgrade registry deps to latest versions; re-resolve non-SHA GitHub refs",
		RunE: func(_ *cobra.Command, args []string) error {
			configDir, err := resolveConfigDir(gf)
			if err != nil {
				return err
			}
			return runDepUpgrade(configDir, args, dryRun)
		},
	}
	cmd.Flags().BoolVarP(&dryRun, "dry-run", "n", false, "Show what would change without writing files")
	return cmd
}

// shaPinned returns true if ref looks like a commit SHA (7–40 hex characters).
var shaRE = regexp.MustCompile(`^[0-9a-f]{7,40}$`)

func shaPinned(ref string) bool {
	return shaRE.MatchString(ref)
}

// upgradeDepEntry pairs a dep declaration with the modfile path that owns it.
type upgradeDepEntry struct {
	dep         modfile.DepDecl
	modfilePath string
}

// loadUpgradeDeps collects all dep entries from both modfiles.
func loadUpgradeDeps(configDir string) ([]upgradeDepEntry, error) {
	modPath := filepath.Join(configDir, configModFile)
	mf, err := modfile.Parse(modPath)
	if err != nil {
		if errors.Is(err, os.ErrNotExist) {
			return nil, fmt.Errorf("dep upgrade: %s not found — run 'meowctl dep add' to declare dependencies first", configModFile)
		}
		return nil, fmt.Errorf("dep upgrade: %w", err)
	}
	var all []upgradeDepEntry
	for _, d := range mf.Deps {
		all = append(all, upgradeDepEntry{d, modPath})
	}

	localModPath := filepath.Join(configDir, configLocalModFile)
	localMF, localErr := modfile.Parse(localModPath)
	if localErr != nil && !errors.Is(localErr, os.ErrNotExist) {
		return nil, fmt.Errorf("dep upgrade: local modfile: %w", localErr)
	}
	if localMF != nil {
		for _, d := range localMF.Deps {
			all = append(all, upgradeDepEntry{d, localModPath})
		}
	}
	return all, nil
}

// runDepUpgrade upgrades all (or the named) modules:
//   - Registry deps: clear version pin so SyncModules re-resolves to latest; rewrite modfile.
//   - GitHub source deps: if not SHA-pinned, dep sync will re-resolve to latest commit SHA.
//   - SHA-pinned GitHub deps are skipped.
func runDepUpgrade(configDir string, filter []string, dryRun bool) error {
	filterSet := make(map[string]struct{}, len(filter))
	for _, name := range filter {
		filterSet[name] = struct{}{}
	}
	wantUpgrade := func(name string) bool {
		if len(filterSet) == 0 {
			return true
		}
		_, ok := filterSet[name]
		return ok
	}

	allDeps, err := loadUpgradeDeps(configDir)
	if err != nil {
		return err
	}

	lockPath := filepath.Join(configDir, configLockFile)
	lf, lockReadErr := lock.Read(lockPath)
	if lockReadErr != nil {
		return fmt.Errorf("dep upgrade: read lock: %w", lockReadErr)
	}

	changed := false
	for _, ds := range allDeps {
		if !wantUpgrade(ds.dep.Name) {
			continue
		}
		didChange, upgradeErr := applyOneUpgrade(ds.dep, ds.modfilePath, lf.Modules, dryRun)
		if upgradeErr != nil {
			return upgradeErr
		}
		if didChange {
			changed = true
		}
	}

	if !changed {
		fmt.Println("meowctl: nothing to upgrade")
		return nil
	}
	if dryRun {
		return nil
	}
	// Write the cleared lock before calling runSync so that the pre-sync state is
	// persisted even if runSync fails partway through (idempotent re-run is then safe).
	if err := lock.Write(lockPath, lf); err != nil {
		return fmt.Errorf("dep upgrade: write lock: %w", err)
	}
	return runSync(configDir)
}

// applyOneUpgrade processes a single dep for upgrade. It mutates modules (lock entries)
// in-place and returns whether any change was made.
func applyOneUpgrade(d modfile.DepDecl, modfilePath string, modules map[string]lock.ModuleEntry, dryRun bool) (bool, error) {
	if d.Source != "" {
		ref := githubRefFromSource(d.Source)
		if shaPinned(ref) {
			return false, nil // SHA-pinned: skip.
		}
		if dryRun {
			fmt.Printf("  would re-resolve %s (ref: %s)\n", d.Name, ref)
		} else {
			fmt.Printf("  re-resolving %s (ref: %s)\n", d.Name, ref)
			delete(modules, d.Name)
		}
		return true, nil
	}
	// Registry dep: clear version pin to force latest resolution.
	if dryRun {
		fmt.Printf("  would upgrade %s (version: %s → latest)\n", d.Name, d.Version)
	} else {
		fmt.Printf("  upgrading %s (version: %s → latest)\n", d.Name, d.Version)
		delete(modules, d.Name)
		if err := rewriteDepVersion(modfilePath, d.Name, "latest"); err != nil {
			return false, fmt.Errorf("dep upgrade: rewrite %s: %w", d.Name, err)
		}
	}
	return true, nil
}

// rewriteDepVersion updates the version field for a named dep in a modfile.
// An empty version string signals "latest" (SyncModules interprets "" as latest).
func rewriteDepVersion(modfilePath, name, version string) error {
	mf, err := modfile.Parse(modfilePath)
	if err != nil {
		return err
	}
	found := false
	for i, d := range mf.Deps {
		if d.Name == name {
			mf.Deps[i].Version = version
			found = true
			break
		}
	}
	if !found {
		return fmt.Errorf("dep %q not found in modfile", name)
	}
	return modfile.Write(modfilePath, mf)
}

// githubRefFromSource extracts the ref portion from a "github:owner/repo@ref" source string.
func githubRefFromSource(source string) string {
	at := strings.LastIndex(source, "@")
	if at < 0 {
		return ""
	}
	return source[at+1:]
}

// newDepTidyCmd implements "meowctl dep tidy [--dry-run]".
func newDepTidyCmd(gf *globalFlags) *cobra.Command {
	var dryRun bool
	cmd := &cobra.Command{
		Use:   "tidy",
		Short: "Remove deps not referenced in star files; warn on unknown refs",
		RunE: func(_ *cobra.Command, _ []string) error {
			configDir, err := resolveConfigDir(gf)
			if err != nil {
				return err
			}
			return runDepTidy(configDir, dryRun)
		},
	}
	cmd.Flags().BoolVar(&dryRun, "dry-run", false, "Show what would change without writing files")
	return cmd
}

// depRefRE matches @name// load-URL refs in star files.
var depRefRE = regexp.MustCompile(`@(\w[\w-]*)//`)

func runDepTidy(configDir string, dryRun bool) error {
	refs := collectStarRefs(configDir)

	modPath := filepath.Join(configDir, configModFile)
	mf, err := modfile.Parse(modPath)
	if err != nil {
		return fmt.Errorf("dep tidy: %w", err)
	}

	// Note: deps.local.mod is not tidied — local overrides are user-managed.
	kept, orphans := partitionDeps(mf.Deps, refs)
	for _, name := range orphans {
		fmt.Printf("  orphan %s (not referenced in any star file)\n", name)
	}

	if err := warnUnknownRefs(refs, mf.Deps, filepath.Join(configDir, configLocalModFile)); err != nil {
		return err
	}

	if len(orphans) == 0 {
		fmt.Println("meowctl: nothing to tidy")
		return nil
	}

	if dryRun {
		fmt.Printf("meowctl: would remove %d orphan dep(s)\n", len(orphans))
		return nil
	}

	mf.Deps = kept
	if err := modfile.Write(modPath, mf); err != nil {
		return fmt.Errorf("dep tidy: write modfile: %w", err)
	}
	fmt.Printf("meowctl: removed %d orphan dep(s)\n", len(orphans))
	return runSync(configDir)
}

// collectStarRefs scans init.star, local.star, and hooks/*.star for @name// refs.
func collectStarRefs(configDir string) map[string]bool {
	refs := map[string]bool{}
	starFiles := []string{
		filepath.Join(configDir, "init.star"),
		filepath.Join(configDir, configLocalFile),
	}
	hooksDir := filepath.Join(configDir, "hooks")
	if entries, err := os.ReadDir(hooksDir); err == nil {
		for _, e := range entries {
			if !e.IsDir() && strings.HasSuffix(e.Name(), ".star") {
				starFiles = append(starFiles, filepath.Join(hooksDir, e.Name()))
			}
		}
	}
	for _, f := range starFiles {
		data, err := os.ReadFile(f) // #nosec G304 -- configDir is trusted
		if err != nil {
			continue
		}
		for _, m := range depRefRE.FindAllSubmatch(data, -1) {
			refs[string(m[1])] = true
		}
	}
	return refs
}

// partitionDeps splits deps into kept (referenced) and orphaned (unreferenced) name slices.
func partitionDeps(deps []modfile.DepDecl, refs map[string]bool) (kept []modfile.DepDecl, orphans []string) {
	for _, d := range deps {
		if refs[d.Name] {
			kept = append(kept, d)
		} else {
			orphans = append(orphans, d.Name)
		}
	}
	return kept, orphans
}

// warnUnknownRefs prints a warning for any ref not declared in shared or local modfile.
// Warnings go to os.Stderr directly (not the cobra output writer) so they appear even
// when stdout is captured or redirected.
func warnUnknownRefs(refs map[string]bool, sharedDeps []modfile.DepDecl, localModPath string) error {
	declared := map[string]bool{}
	for _, d := range sharedDeps {
		declared[d.Name] = true
	}
	if localMF, err := modfile.Parse(localModPath); err == nil {
		for _, d := range localMF.Deps {
			declared[d.Name] = true
		}
	}
	for ref := range refs {
		if !declared[ref] {
			if _, err := fmt.Fprintf(os.Stderr, "meowctl: warning: @%s// referenced but not declared in any modfile\n", ref); err != nil {
				return err
			}
		}
	}
	return nil
}
