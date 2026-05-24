package cli

import (
	"bytes"
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/meowshed/meowctl/internal/state"
	"github.com/spf13/cobra"
)

// newUpdateCmd returns the "meowctl update" command.
func newUpdateCmd(gf *globalFlags) *cobra.Command {
	var cfg runConfig
	var yes bool
	cmd := &cobra.Command{
		Use:   "update",
		Short: "Pull the latest dotfiles from the remote repo and apply changes",
		Long: `update re-downloads the dotfiles repo tarball, diffs it against the
local config directory, and applies any changes. Protected files
(local.star, deps.local.mod, deps.local.lock) are never overwritten.

If the config directory was bootstrapped with a repo URL, update uses that URL.
Otherwise, update exits with an error.`,
		RunE: func(_ *cobra.Command, _ []string) error {
			configDir, err := resolveConfigDir(gf)
			if err != nil {
				return err
			}
			cfg.ConfigDir = configDir
			return runUpdate(cfg, yes)
		},
	}
	addLifecycleFlags(cmd, &cfg)
	cmd.Flags().BoolVarP(&yes, "yes", "y", false, "Auto-apply changes without prompting")
	return cmd
}

// runUpdate implements the tarball re-download + diff + apply flow.
func runUpdate(cfg runConfig, yes bool) error {
	configDir := cfg.ConfigDir

	// 1. Read repo URL from state.toml.
	mgr := state.NewManager(filepath.Join(configDir, configStateFile))
	s, err := mgr.Load()
	if err != nil {
		return fmt.Errorf("update: read state: %w", err)
	}
	if s.RepoURL == "" {
		return exitErrorf(ExitConfig,
			"update: no repo_url stored in state.toml\n  re-run 'meowctl init <repo-url>' or edit state.toml manually")
	}

	// 2. Download tarball to staging directory.
	stagingDir, changes, err := fetchAndDiff(configDir, s.RepoURL)
	if err != nil {
		return err
	}
	defer func() { _ = os.RemoveAll(stagingDir) }()

	if len(changes) == 0 {
		fmt.Println("meowctl: no changes from upstream")
		return nil
	}

	// 3. Present changes.
	fmt.Printf("meowctl: %d file(s) changed:\n", len(changes))
	for _, c := range changes {
		fmt.Printf("  %s  %s\n", c.op, c.path)
	}

	// 4. If dry-run, stop here.
	if cfg.DryRun {
		fmt.Println("meowctl: --dry-run — no changes made")
		return nil
	}

	// 5. Prompt and apply.
	if err := promptAndApply(changes, stagingDir, configDir, yes); err != nil {
		return err
	}

	// 6. Run dep sync if deps.mod changed.
	if err := maybeRunDepSync(changes, configDir); err != nil {
		return err
	}

	// 7. Run apply.
	fmt.Println("meowctl: running apply")
	return runApply(cfg, nil)
}

// fetchAndDiff downloads the tarball for repoURL, extracts it to a staging
// directory inside configDir, and returns the list of changed files.
func fetchAndDiff(configDir, repoURL string) (string, []change, error) {
	url := tarballURL(repoURL)
	fmt.Printf("meowctl: downloading %s\n", url)

	tmpPath, err := downloadTarball(url)
	if err != nil {
		return "", nil, fmt.Errorf("update: download: %w", err)
	}
	defer func() { _ = os.Remove(tmpPath) }()

	stagingDir := filepath.Join(configDir, ".update-staging")
	_ = os.RemoveAll(stagingDir) // Clean up any leftover staging dir.
	if err := os.MkdirAll(stagingDir, 0o700); err != nil {
		return "", nil, fmt.Errorf("update: create staging dir: %w", err)
	}

	fmt.Printf("meowctl: extracting to %s\n", stagingDir)
	if err := extractTarball(tmpPath, stagingDir); err != nil {
		return "", nil, fmt.Errorf("update: extract: %w", err)
	}

	changes, err := diffConfigDirs(stagingDir, configDir)
	if err != nil {
		return "", nil, fmt.Errorf("update: diff: %w", err)
	}

	return stagingDir, changes, nil
}

// promptAndApply asks the user for confirmation (unless yes is true) and then
// copies each changed file from the staging directory into the config directory.
func promptAndApply(changes []change, stagingDir, configDir string, yes bool) error {
	if !yes {
		fmt.Print("Apply changes? [y/N] ")
		var resp string
		if _, err := fmt.Scanln(&resp); err != nil {
			return fmt.Errorf("update: read confirmation: %w", err)
		}
		if strings.ToLower(resp) != "y" && strings.ToLower(resp) != "yes" {
			fmt.Println("meowctl: update cancelled")
			return nil
		}
	}

	for _, c := range changes {
		if err := applyChange(c, stagingDir, configDir); err != nil {
			return fmt.Errorf("update: apply %s: %w", c.path, err)
		}
	}
	return nil
}

// maybeRunDepSync checks whether deps.mod was among the changed files and, if
// so, re-runs dep sync.
func maybeRunDepSync(changes []change, configDir string) error {
	for _, c := range changes {
		if c.path == configModFile {
			fmt.Println("meowctl: deps.mod changed — running dep sync")
			if err := runSync(configDir); err != nil {
				return fmt.Errorf("update: dep sync: %w", err)
			}
			break
		}
	}
	return nil
}

// changeOp describes what happened to a file.
type changeOp string

const (
	opAdd    changeOp = "+"
	opModify changeOp = "~"
)

// change describes a single file difference.
type change struct {
	path string
	op   changeOp
}

// diffConfigDirs walks stagingDir and compares each file with the corresponding
// file in configDir. Returns a list of changes, skipping protected files.
func diffConfigDirs(stagingDir, configDir string) ([]change, error) {
	var changes []change

	err := filepath.Walk(stagingDir, func(stagingPath string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}
		if info.IsDir() {
			return nil
		}

		// Compute relative path from stagingDir.
		rel, err := filepath.Rel(stagingDir, stagingPath)
		if err != nil {
			return err
		}

		// Skip protected machine-local files.
		if isProtectedFile(rel) {
			return nil
		}

		configPath := filepath.Join(configDir, rel)
		configInfo, err := os.Lstat(configPath)
		if os.IsNotExist(err) {
			changes = append(changes, change{path: rel, op: opAdd})
			return nil
		}
		if err != nil {
			return err
		}

		// Skip directories in config (shouldn't happen with Walk, but guard anyway).
		if configInfo.IsDir() {
			return nil
		}

		// Compare contents.
		same, err := filesEqual(stagingPath, configPath)
		if err != nil {
			return err
		}
		if !same {
			changes = append(changes, change{path: rel, op: opModify})
		}

		return nil
	})

	return changes, err
}

// isProtectedFile returns true for files that should never be overwritten
// during an update because they contain machine-local state.
func isProtectedFile(rel string) bool {
	base := filepath.Base(rel)
	switch base {
	case configLocalFile, configLocalModFile, configLocalLockFile:
		return true
	}
	return false
}

// filesEqual compares two files by content. It reads both entirely into memory;
// dotfiles are small enough for this to be practical.
func filesEqual(a, b string) (bool, error) {
	aData, err := os.ReadFile(a) // #nosec G304
	if err != nil {
		return false, err
	}
	bData, err := os.ReadFile(b) // #nosec G304
	if err != nil {
		return false, err
	}
	return bytes.Equal(aData, bData), nil
}

// applyChange copies a single file from staging to the config directory.
// It creates parent directories as needed and overwrites existing files.
func applyChange(c change, stagingDir, configDir string) error {
	src := filepath.Join(stagingDir, c.path)
	dst := filepath.Join(configDir, c.path)

	if err := os.MkdirAll(filepath.Dir(dst), 0o700); err != nil {
		return fmt.Errorf("create parent: %w", err)
	}

	data, err := os.ReadFile(src) // #nosec G304
	if err != nil {
		return fmt.Errorf("read staging: %w", err)
	}

	if err := os.WriteFile(dst, data, 0o600); err != nil { // #nosec G703
		return fmt.Errorf("write config: %w", err)
	}

	return nil
}
