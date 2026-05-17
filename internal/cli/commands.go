package cli

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/meowshed/meowctl/internal/modfile"
	"github.com/spf13/cobra"
)

// starterTemplate is the content written to init.star by meowctl init.
const starterTemplate = `# init.star — dotfiles configuration
# See https://github.com/meowshed/meowctl for documentation.

# Declare the @stdlib source so components can reference it.
# source("@stdlib", "github://meowshed/meow-stdlib")

# Declare components. Each component corresponds to a <name>.star file
# in the components/ directory, or a stdlib component via @stdlib//.
#
# component("shell")
# component("git")
# component("@stdlib//components/zsh")
`

// localStarTemplate is the content written to local.star by meowctl init.
// local.star is gitignored and used for machine-specific additions.
const localStarTemplate = `# local.star — machine-specific component additions (gitignored)
# Add components that should only apply to this machine.
#
# component("work-vpn")
`

// localModTemplate is the content written to deps.local.mod by meowctl init.
// deps.local.mod is gitignored and used for machine-specific module overrides.
const localModTemplate = `# deps.local.mod — machine-specific module declarations (gitignored)
# Declare additional or override deps for this machine only.
# Entries here shadow the same-named entries in deps.mod at runtime.
`

func newInitCmd(gf *globalFlags) *cobra.Command {
	var force bool
	var repoURL string
	cmd := &cobra.Command{
		Use:   "init [<repo-url>]",
		Short: "Scaffold a new config directory, or bootstrap from an existing dotfiles repo",
		Long: `init with no arguments scaffolds a meowctl config directory with a starter
init.star, local.star, deps.mod, and deps.local.mod.

init <repo-url> bootstraps a new machine from an existing dotfiles repository.
The repo must be a public GitHub or GitLab repository. The tarball is fetched
from <repo-url>/archive/refs/heads/main.tar.gz using pure Go HTTPS — no git
subprocess is required.

If init.star already exists in the config directory, init exits with an error
unless --force is passed.`,
		Args: cobra.MaximumNArgs(1),
		RunE: func(_ *cobra.Command, args []string) error {
			configDir, err := resolveConfigDir(gf)
			if err != nil {
				return err
			}
			if len(args) == 1 {
				return runBootstrap(configDir, args[0], force)
			}
			return runInit(configDir, force)
		},
	}
	cmd.Flags().BoolVarP(&force, "force", "f", false, "Overwrite existing config")
	_ = repoURL // consumed via args
	return cmd
}

// runInit scaffolds a meowctl config directory at configDir.
// If force is true, an existing init.star is removed before scaffolding.
func runInit(configDir string, force bool) error {
	// Legacy migration check: if meowctl.star exists but init.star does not,
	// print clear instructions and exit. Never auto-rename.
	legacyPath := filepath.Join(configDir, "meowctl.star")
	newEntryPath := filepath.Join(configDir, configEntryFile)
	if _, legacyErr := os.Lstat(legacyPath); legacyErr == nil {
		if _, newErr := os.Lstat(newEntryPath); os.IsNotExist(newErr) {
			fmt.Fprintf(os.Stderr, "meowctl: found legacy config — rename files to continue:\n")
			fmt.Fprintf(os.Stderr, "  mv %s %s\n", legacyPath, newEntryPath)
			fmt.Fprintf(os.Stderr, "  mv %s %s\n",
				filepath.Join(configDir, "meowctl.mod"), filepath.Join(configDir, configModFile))
			fmt.Fprintf(os.Stderr, "  mv %s %s\n",
				filepath.Join(configDir, "meowctl.lock"), filepath.Join(configDir, configLockFile))
			return exitErrorf(ExitConfig, "legacy config found — see instructions above")
		}
	}

	// Guard: init.star already exists.
	if _, statErr := os.Lstat(newEntryPath); statErr == nil {
		if !force {
			return exitErrorf(ExitConfig, "init.star already exists at %s\n  delete it or run 'meowctl init --force' to overwrite", newEntryPath)
		}
		if removeErr := os.Remove(newEntryPath); removeErr != nil {
			return fmt.Errorf("init: remove existing init.star: %w", removeErr)
		}
	}

	for _, subdir := range []string{configDir, filepath.Join(configDir, "components"), filepath.Join(configDir, "hooks")} {
		if err := os.MkdirAll(subdir, 0o700); err != nil {
			return fmt.Errorf("init: create directory %s: %w", subdir, err)
		}
	}

	if err := scaffoldInitStar(configDir); err != nil {
		return err
	}

	// Write local.star stub (gitignored; machine-specific additions).
	localPath := filepath.Join(configDir, configLocalFile)
	if _, localStatErr := os.Lstat(localPath); os.IsNotExist(localStatErr) {
		if writeErr := os.WriteFile(localPath, []byte(localStarTemplate), 0o600); writeErr != nil {
			return fmt.Errorf("init: write local.star: %w", writeErr)
		}
	}

	// Write companion deps.mod with the module identity.
	modPath := filepath.Join(configDir, configModFile)
	mf := &modfile.ModFile{
		Module: &modfile.ModuleDecl{
			Name:    "my-dotfiles",
			Version: "0.1.0",
		},
	}
	if err := modfile.Write(modPath, mf); err != nil {
		return fmt.Errorf("init: write deps.mod: %w", err)
	}

	// Write deps.local.mod stub (gitignored; machine-specific module overrides).
	localModPath := filepath.Join(configDir, configLocalModFile)
	if _, localModStatErr := os.Lstat(localModPath); os.IsNotExist(localModStatErr) {
		if writeErr := os.WriteFile(localModPath, []byte(localModTemplate), 0o600); writeErr != nil {
			return fmt.Errorf("init: write deps.local.mod: %w", writeErr)
		}
	}

	// Ensure local files are gitignored.
	for _, entry := range []string{configLocalFile, configLocalModFile, configLocalLockFile} {
		if err := ensureGitignore(configDir, entry); err != nil {
			return fmt.Errorf("init: update .gitignore: %w", err)
		}
	}

	fmt.Printf("Dotfiles initialized at %s\n", configDir)
	fmt.Printf("Edit init.star to declare components, then run 'meowctl apply'.\n")
	return nil
}

// scaffoldInitStar creates init.star in configDir using O_EXCL (atomic).
func scaffoldInitStar(configDir string) error {
	starPath := filepath.Join(configDir, configEntryFile)
	f, err := os.OpenFile(starPath, os.O_CREATE|os.O_EXCL|os.O_WRONLY, 0o600) // #nosec G304 -- path is resolved from trusted config dir
	if err != nil {
		if os.IsExist(err) {
			return exitErrorf(ExitConfig, "init.star already exists at %s\n  delete it or run 'meowctl init --force' to overwrite", starPath)
		}
		return fmt.Errorf("init: create init.star: %w", err)
	}
	if _, err := fmt.Fprint(f, starterTemplate); err != nil {
		_ = f.Close()
		return fmt.Errorf("init: write init.star: %w", err)
	}
	if err := f.Close(); err != nil {
		return fmt.Errorf("init: close init.star: %w", err)
	}
	return nil
}

// ensureGitignore appends entry to <dir>/.gitignore if it is not already present.
// Creates the file if absent.
func ensureGitignore(dir, entry string) error {
	gitignorePath := filepath.Join(dir, ".gitignore")
	data, readErr := os.ReadFile(gitignorePath) // #nosec G304
	if readErr != nil && !os.IsNotExist(readErr) {
		return fmt.Errorf("read .gitignore: %w", readErr)
	}
	content := string(data)
	// Check for exact line match.
	for _, line := range splitLines(content) {
		if line == entry {
			return nil
		}
	}
	// Append entry (with leading newline if file is non-empty and doesn't end with newline).
	if len(content) > 0 && content[len(content)-1] != '\n' {
		content += "\n"
	}
	content += entry + "\n"
	return os.WriteFile(gitignorePath, []byte(content), 0o600) // #nosec G703
}

// splitLines splits s into lines, omitting the final empty element produced by a trailing newline.
func splitLines(s string) []string {
	if s == "" {
		return nil
	}
	return strings.Split(strings.TrimSuffix(s, "\n"), "\n")
}
