package cli

import (
	"github.com/spf13/cobra"
)

// RootCmd is the top-level cobra command for meowctl.
var RootCmd = newRootCmd()

func newRootCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "meowctl",
		Short: "Declarative dotfiles and dev environment manager",
		Long: `meowctl manages dotfiles and developer environments using
Starlark configuration files.`,
		SilenceUsage: true,
	}

	cmd.AddCommand(
		newVersionCmd(),
		newBootstrapCmd(),
		newInitCmd(),
		newInstallCmd(),
		newSetupCmd(),
		newShellCmd(),
		newUninstallCmd(),
		newVerifyCmd(),
		newRestoreCmd(),
	)

	return cmd
}
