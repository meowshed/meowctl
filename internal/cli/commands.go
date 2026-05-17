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

func newBootstrapCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "bootstrap",
		Short: "Bootstrap a new machine from dotfiles (not yet implemented)",
		Args:  cobra.NoArgs,
		RunE: func(_ *cobra.Command, _ []string) error {
			return errNotImplemented("bootstrap")
		},
	}
}

func newInitCmd(gf *globalFlags) *cobra.Command {
	var force bool
	cmd := &cobra.Command{
		Use:   "init",
		Short: "Scaffold a meowctl config directory with a starter init.star",
		Args:  cobra.NoArgs,
		RunE: func(_ *cobra.Command, _ []string) error {
			configDir, err := resolveConfigDir(gf)
			if err != nil {
				return err
			}

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

			if err := os.MkdirAll(configDir, 0o700); err != nil {
				return fmt.Errorf("init: create config directory: %w", err)
			}
			componentsDir := filepath.Join(configDir, "components")
			if err := os.MkdirAll(componentsDir, 0o700); err != nil {
				return fmt.Errorf("init: create components directory: %w", err)
			}
			hooksDir := filepath.Join(configDir, "hooks")
			if err := os.MkdirAll(hooksDir, 0o700); err != nil {
				return fmt.Errorf("init: create hooks directory: %w", err)
			}

			starPath := filepath.Join(configDir, configEntryFile)
			// O_EXCL makes the create atomic — fails if the file already exists.
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

			// Ensure local.star is gitignored.
			if err := ensureGitignore(configDir, configLocalFile); err != nil {
				return fmt.Errorf("init: update .gitignore: %w", err)
			}

			fmt.Printf("Dotfiles initialized at %s\n", configDir)
			fmt.Printf("Edit init.star to declare components, then run 'meowctl apply'.\n")
			return nil
		},
	}
	cmd.Flags().BoolVarP(&force, "force", "f", false, "Overwrite existing init.star")
	return cmd
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

func newSetupCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "setup",
		Short: "Run the setup phase of all components (not yet implemented)",
		Args:  cobra.NoArgs,
		RunE: func(_ *cobra.Command, _ []string) error {
			return errNotImplemented("setup")
		},
	}
}
