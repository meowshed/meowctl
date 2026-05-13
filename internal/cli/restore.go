package cli

import (
	"fmt"
	"path/filepath"

	"github.com/meowshed/meowctl/internal/rollback"
	"github.com/spf13/cobra"
)

func newRestoreCmd(gf *globalFlags) *cobra.Command {
	var ignoreLock bool
	cmd := &cobra.Command{
		Use:   "restore",
		Short: "Roll back the last install run by replaying the rollback journal",
		Args:  cobra.NoArgs,
		RunE: func(_ *cobra.Command, _ []string) error {
			configDir, err := resolveConfigDir(gf)
			if err != nil {
				return err
			}
			_ = ignoreLock // reserved for M7 module resolver integration

			journalPath := filepath.Join(configDir, "rollback.jsonl")
			stack, err := rollback.Open(journalPath)
			if err != nil {
				return fmt.Errorf("restore: open rollback journal: %w", err)
			}
			defer func() { _ = stack.Close() }()

			if !stack.Pending() {
				fmt.Println("Nothing to restore: rollback journal is empty.")
				return nil
			}

			result := stack.Execute()
			if result.Err != nil {
				return fmt.Errorf("restore: %w", result.Err)
			}
			if len(result.Failures) > 0 {
				for _, f := range result.Failures {
					fmt.Printf("  failed: %s/%s: %v\n", f.Record.Component, f.Record.Kind, f.Err)
				}
				return exitErrorf(ExitGeneral, "restore: %d operation(s) failed", len(result.Failures))
			}
			if result.SkippedLines > 0 {
				fmt.Printf("Warning: %d malformed journal line(s) skipped (lines: %v).\n", result.SkippedLines, result.SkippedAt)
			}
			fmt.Println("Restore complete.")
			return nil
		},
	}
	cmd.Flags().BoolVar(&ignoreLock, "ignore-lock", false, "Ignore lock file when resolving modules")
	return cmd
}
