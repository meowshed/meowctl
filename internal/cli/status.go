package cli

import (
	"encoding/json"
	"fmt"
	"path/filepath"

	"github.com/meowshed/meowctl/internal/state"
	"github.com/spf13/cobra"
)

func newStatusCmd(gf *globalFlags) *cobra.Command {
	var jsonOut bool
	cmd := &cobra.Command{
		Use:   "status",
		Short: "Show the last-run metadata and completed components",
		Args:  cobra.NoArgs,
		RunE: func(_ *cobra.Command, _ []string) error {
			configDir, err := resolveConfigDir(gf)
			if err != nil {
				return err
			}

			statePath := filepath.Join(configDir, "state.toml")
			sm := state.NewManager(statePath)
			sentinel, err := sm.Load()
			if err != nil {
				return fmt.Errorf("status: %w", err)
			}

			if jsonOut {
				out, _ := json.MarshalIndent(sentinel, "", "  ")
				fmt.Println(string(out))
				return nil
			}

			lr := sentinel.LastRun
			if lr.StartedAt.IsZero() {
				fmt.Println("No runs recorded.")
				return nil
			}
			fmt.Printf("Last run:  %s\n", lr.StartedAt.Format("2006-01-02 15:04:05 MST"))
			fmt.Printf("Phase set: %s\n", lr.PhaseSet)
			if lr.Completed {
				fmt.Println("Result:    completed")
			} else if lr.RolledBack != "" {
				fmt.Printf("Result:    rolled back (%s)\n", lr.RolledBack)
			} else {
				fmt.Println("Result:    in progress / interrupted")
			}
			fmt.Printf("\nCompleted components (%d):\n", len(sentinel.CompletedComponents))
			for _, cc := range sentinel.CompletedComponents {
				fmt.Printf("  %s  (phase: %s, at %s)\n", cc.Component, cc.Phase, cc.CompletedAt.Format("2006-01-02 15:04:05"))
			}
			return nil
		},
	}
	cmd.Flags().BoolVar(&jsonOut, "json", false, "Output as JSON")
	return cmd
}
