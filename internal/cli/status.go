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
		RunE: func(cmd *cobra.Command, _ []string) error {
			configDir, err := resolveConfigDir(gf)
			if err != nil {
				return err
			}

			w := cmd.OutOrStdout()

			// Warn if a previous hook eval failure was recorded.
			if hookErrorExists(configDir) {
				_, _ = fmt.Fprintln(w, "warning: last hook shell eval failed — run 'meowctl doctor' for details")
			}

			statePath := filepath.Join(configDir, configStateFile)
			sm := state.NewManager(statePath)
			sentinel, err := sm.Load()
			if err != nil {
				return fmt.Errorf("status: %w", err)
			}

			if jsonOut {
				out, err := json.MarshalIndent(sentinel, "", "  ")
				if err != nil {
					return fmt.Errorf("status: marshal json: %w", err)
				}
				_, _ = fmt.Fprintln(w, string(out))
				return nil
			}

			lr := sentinel.LastRun
			if lr.StartedAt.IsZero() {
				_, _ = fmt.Fprintln(w, "No runs recorded.")
				return nil
			}
			_, _ = fmt.Fprintf(w, "Last run:  %s\n", lr.StartedAt.Format("2006-01-02 15:04:05 MST"))
			_, _ = fmt.Fprintf(w, "Phase set: %s\n", lr.PhaseSet)
			if lr.Completed {
				_, _ = fmt.Fprintln(w, "Result:    completed")
			} else if lr.RolledBack != "" {
				_, _ = fmt.Fprintf(w, "Result:    rolled back (%s)\n", lr.RolledBack)
			} else {
				_, _ = fmt.Fprintln(w, "Result:    in progress / interrupted")
			}
			_, _ = fmt.Fprintf(w, "\nCompleted components (%d):\n", len(sentinel.CompletedComponents))
			for _, cc := range sentinel.CompletedComponents {
				_, _ = fmt.Fprintf(w, "  %s  (phase: %s, at %s)\n", cc.Component, cc.Phase, cc.CompletedAt.Format("2006-01-02 15:04:05"))
			}
			return nil
		},
	}
	cmd.Flags().BoolVar(&jsonOut, "json", false, "Output as JSON")
	return cmd
}
