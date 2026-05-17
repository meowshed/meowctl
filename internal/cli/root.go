// Package cli implements the meowctl command-line interface.
package cli

import (
	"github.com/spf13/cobra"
)

// globalFlags holds values for the root persistent flags.
type globalFlags struct {
	ConfigDir string
	Verbose   bool
}

// RootCmd is the top-level cobra command for meowctl.
var RootCmd = newRootCmd()

func newRootCmd() *cobra.Command {
	var gf globalFlags

	cmd := &cobra.Command{
		Use:   "meowctl",
		Short: "Dotfiles and dev environment manager powered by Starlark",
		Long: `meowctl manages dotfiles and developer environments using
Starlark configuration files.`,
		SilenceUsage:  true,
		SilenceErrors: true,
	}

	// Global persistent flags — inherited by every subcommand.
	cmd.PersistentFlags().StringVar(&gf.ConfigDir, "config", "", "Config directory (default: ~/.config/meowctl)")
	cmd.PersistentFlags().BoolVarP(&gf.Verbose, "verbose", "v", false, "Enable verbose output")

	cmd.AddCommand(
		newVersionCmd(),
		newInitCmd(&gf),
		newApplyCmd(&gf),
		newAddCmd(&gf),
		newRemoveCmd(&gf),
		newUpgradeCmd(&gf),
		newShellCmd(),
		newHookCmd(&gf),
		newVerifyCmd(&gf),
		newDoctorCmd(&gf),
		newStatusCmd(&gf),
		newSelfUpdateCmd(),
		newDepCmd(&gf),
		newCheckCmd(),
	)

	// Completion subcommands are added automatically by Cobra on first Execute.
	// Call explicitly to ensure they are present for testing and help output.
	cmd.InitDefaultCompletionCmd()

	return cmd
}

// resolveConfigDir returns the config directory from the flag if set, or the default.
func resolveConfigDir(gf *globalFlags) (string, error) {
	if gf.ConfigDir != "" {
		return gf.ConfigDir, nil
	}
	return defaultConfigDir()
}
