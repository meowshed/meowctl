package cli

import (
	"encoding/json"
	"fmt"
	"path/filepath"
	"sort"

	"github.com/meowshed/meowctl/internal/state"
	"github.com/meowshed/meowctl/internal/tui"
	"github.com/spf13/cobra"
)

// recentComponents is how many completion records status shows by default.
// The ledger is append-only and grows without bound — on a settled config it
// runs to several hundred entries — so dumping all of it buries the run
// metadata that the command exists to report. --all prints the lot.
const recentComponents = 10

const statusTimeFormat = "2006-01-02 15:04:05"

func newStatusCmd(gf *globalFlags) *cobra.Command {
	var jsonOut bool
	var all bool
	cmd := &cobra.Command{
		Use:   "status",
		Short: "Show the last-run metadata and completed components",
		Args:  cobra.NoArgs,
		RunE: func(cmd *cobra.Command, _ []string) error {
			configDir, err := resolveConfigDir(gf)
			if err != nil {
				return err
			}
			return runStatus(cmd.OutOrStdout(), cmd.ErrOrStderr(), configDir, jsonOut, all)
		},
	}
	cmd.Flags().BoolVar(&jsonOut, "json", false, "Output as JSON")
	cmd.Flags().BoolVar(&all, "all", false, "List every completed component, not just the most recent")
	return cmd
}

func runStatus(out, errOut interface{ Write([]byte) (int, error) }, configDir string, jsonOut, all bool) error {
	p := tui.NewPrinter(out, errOut)

	statePath := filepath.Join(configDir, configStateFile)
	sm := state.NewManager(statePath)
	sentinel, err := sm.Load()
	if err != nil {
		return fmt.Errorf("status: %w", err)
	}

	if jsonOut {
		enc, err := json.MarshalIndent(sentinel, "", "  ")
		if err != nil {
			return fmt.Errorf("status: marshal json: %w", err)
		}
		_, _ = out.Write(append(enc, '\n'))
		return nil
	}

	// A recorded hook failure is a diagnostic, so it goes to stderr and cannot
	// corrupt `meowctl status | ...`.
	if hookErrorExists(configDir) {
		p.Warn("last hook shell eval failed — run 'meowctl doctor' for details")
	}

	lr := sentinel.LastRun
	if lr.StartedAt.IsZero() {
		p.Note("No runs recorded.")
		return nil
	}

	result, resultStatus := runResult(lr)
	p.KeyValue([][2]string{
		{"Last run", lr.StartedAt.Local().Format(statusTimeFormat)},
		{"Phase set", lr.PhaseSet},
	})
	p.Item(resultStatus, result, "")

	records := sentinel.CompletedComponents
	if len(records) == 0 {
		return nil
	}

	// Most recent first: the tail of an append-only ledger is the part anyone
	// is actually looking for.
	sorted := make([]state.CompletedComponent, len(records))
	copy(sorted, records)
	sort.SliceStable(sorted, func(i, j int) bool {
		return sorted[i].CompletedAt.After(sorted[j].CompletedAt)
	})

	shown := sorted
	if !all && len(shown) > recentComponents {
		shown = shown[:recentComponents]
	}

	p.Blank()
	p.Heading("Completed components (%d)", len(records))
	items := make([]tui.ItemSpec, 0, len(shown))
	for _, cc := range shown {
		items = append(items, tui.ItemSpec{
			Status: tui.StatusSuccess,
			Label:  cc.Component,
			Note:   fmt.Sprintf("%s  %s", cc.Phase, cc.CompletedAt.Local().Format(statusTimeFormat)),
		})
	}
	p.ItemList(items)

	if hidden := len(sorted) - len(shown); hidden > 0 {
		p.Note("%d older — pass --all to list them.", hidden)
	}
	return nil
}

// runResult renders the outcome of the last run and the status it maps to.
func runResult(lr state.LastRun) (string, tui.Status) {
	switch {
	case lr.Completed:
		return "completed", tui.StatusSuccess
	case lr.RolledBack != "":
		return fmt.Sprintf("rolled back (%s)", lr.RolledBack), tui.StatusFailure
	default:
		return "in progress or interrupted", tui.StatusWarning
	}
}
