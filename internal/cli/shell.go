package cli

import (
	"github.com/spf13/cobra"
)

func newShellCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "shell",
		Short: "Emit shell integration code (not yet implemented)",
		Args:  cobra.NoArgs,
		RunE: func(_ *cobra.Command, _ []string) error {
			return errNotImplemented("shell")
		},
	}
}
