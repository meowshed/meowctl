package cli

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/meowshed/meowctl/internal/modfile"
	"github.com/spf13/cobra"
)

// starterTemplate is the content written to meowctl.star by meowctl init.
const starterTemplate = `# meowctl.star — dotfiles configuration
# See https://github.com/meowshed/meowctl for documentation.

module(
    name = "my-dotfiles",
    version = "0.1.0",
)

# Declare components. Each component corresponds to a <name>.star file
# in the components/ directory.
#
# component("shell")
# component("git")
# component("neovim")
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
	return &cobra.Command{
		Use:   "init",
		Short: "Scaffold a meowctl config directory with a starter meowctl.star",
		Args:  cobra.NoArgs,
		RunE: func(_ *cobra.Command, _ []string) error {
			configDir, err := resolveConfigDir(gf)
			if err != nil {
				return err
			}

			if err := os.MkdirAll(configDir, 0o700); err != nil {
				return fmt.Errorf("init: create config directory: %w", err)
			}
			componentsDir := filepath.Join(configDir, "components")
			if err := os.MkdirAll(componentsDir, 0o700); err != nil {
				return fmt.Errorf("init: create components directory: %w", err)
			}

			starPath := filepath.Join(configDir, "meowctl.star")
			// O_EXCL makes the create atomic — fails if the file already exists.
			f, err := os.OpenFile(starPath, os.O_CREATE|os.O_EXCL|os.O_WRONLY, 0o600) // #nosec G304 -- path is resolved from trusted config dir
			if err != nil {
				if os.IsExist(err) {
					return exitErrorf(ExitConfig, "meowctl.star already exists at %s\n  run 'meowctl install' to apply it", starPath)
				}
				return fmt.Errorf("init: create meowctl.star: %w", err)
			}
			if _, err := fmt.Fprint(f, starterTemplate); err != nil {
				_ = f.Close()
				return fmt.Errorf("init: write meowctl.star: %w", err)
			}
			if err := f.Close(); err != nil {
				return fmt.Errorf("init: close meowctl.star: %w", err)
			}

			// Write companion meowctl.mod with the same module identity.
			modPath := filepath.Join(configDir, "meowctl.mod")
			mf := &modfile.ModFile{
				Module: &modfile.ModuleDecl{
					Name:    "my-dotfiles",
					Version: "0.1.0",
				},
			}
			if err := modfile.Write(modPath, mf); err != nil {
				return fmt.Errorf("init: write meowctl.mod: %w", err)
			}

			fmt.Printf("Initialized meowctl config at %s\n\n", configDir)
			fmt.Printf("Next steps:\n")
			fmt.Printf("  1. Edit %s to declare your components\n", starPath)
			fmt.Printf("  2. Add component files to %s/\n", componentsDir)
			fmt.Printf("  3. Run: meowctl install\n")
			return nil
		},
	}
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
