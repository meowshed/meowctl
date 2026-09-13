package cli

import (
	"fmt"

	"github.com/meowshed/meowctl/internal/version"
	"github.com/spf13/cobra"
)

func newVersionCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "version",
		Short: "Print version information",
		Args:  cobra.NoArgs,
		Run: func(cmd *cobra.Command, _ []string) {
			// Uses the command's writer, not os.Stdout, so the output is
			// redirectable like every other command's.
			_, _ = fmt.Fprintln(cmd.OutOrStdout(), version.String())
		},
	}
}
