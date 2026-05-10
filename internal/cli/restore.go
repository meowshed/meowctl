package cli

import (
	"github.com/spf13/cobra"
)

func newRestoreCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "restore",
		Short: "Roll back the last install run (not yet implemented)",
		Args:  cobra.NoArgs,
		RunE: func(_ *cobra.Command, _ []string) error {
			return errNotImplemented("restore")
		},
	}
}
