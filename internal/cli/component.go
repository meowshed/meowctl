package cli

import (
	"encoding/json"
	"fmt"
	"path/filepath"

	"github.com/meowshed/meowctl/internal/state"
	"github.com/spf13/cobra"
)

func newComponentCmd(gf *globalFlags) *cobra.Command {
	componentCmd := &cobra.Command{
		Use:   "component",
		Short: "Inspect declared components",
		Args:  cobra.NoArgs,
		RunE: func(cmd *cobra.Command, _ []string) error {
			return cmd.Help()
		},
	}
	componentCmd.AddCommand(
		newComponentListCmd(gf),
		newComponentStatusCmd(gf),
	)
	return componentCmd
}

func newComponentListCmd(gf *globalFlags) *cobra.Command {
	return &cobra.Command{
		Use:   "list",
		Short: "List all components declared in meowctl.star",
		Args:  cobra.NoArgs,
		RunE: func(_ *cobra.Command, _ []string) error {
			configDir, err := resolveConfigDir(gf)
			if err != nil {
				return err
			}
			ids, _, err := loadComponentsWithDeps(configDir, nil)
			if err != nil {
				return err
			}
			if len(ids) == 0 {
				fmt.Println("No components declared.")
				return nil
			}
			for _, id := range ids {
				fmt.Println(id)
			}
			return nil
		},
	}
}

func newComponentStatusCmd(gf *globalFlags) *cobra.Command {
	var jsonOut bool
	cmd := &cobra.Command{
		Use:   "status [<name>]",
		Short: "Show install state for all (or a named) component",
		Args:  cobra.MaximumNArgs(1),
		RunE: func(_ *cobra.Command, args []string) error {
			configDir, err := resolveConfigDir(gf)
			if err != nil {
				return err
			}

			ids, _, err := loadComponentsWithDeps(configDir, args)
			if err != nil {
				return err
			}

			statePath := filepath.Join(configDir, "state.toml")
			sm := state.NewManager(statePath)
			sentinel, err := sm.Load()
			if err != nil {
				return fmt.Errorf("component status: %w", err)
			}

			// Build lookup of completed components.
			completedAt := make(map[string]string)
			for _, cc := range sentinel.CompletedComponents {
				completedAt[cc.Component] = cc.CompletedAt.Format("2006-01-02 15:04:05")
			}

			type componentStatus struct {
				Name      string `json:"name"`
				Status    string `json:"status"`
				Completed string `json:"completed_at,omitempty"`
			}
			var results []componentStatus
			for _, id := range ids {
				name := id
				if ts, ok := completedAt[name]; ok {
					results = append(results, componentStatus{name, "installed", ts})
				} else {
					results = append(results, componentStatus{name, "not-installed", ""})
				}
			}

			if jsonOut {
				out, err := json.MarshalIndent(results, "", "  ")
				if err != nil {
					return fmt.Errorf("component status: marshal json: %w", err)
				}
				fmt.Println(string(out))
				return nil
			}
			for _, r := range results {
				if r.Status == "installed" {
					fmt.Printf("  ✓ %-24s installed (%s)\n", r.Name, r.Completed)
				} else {
					fmt.Printf("  - %-24s not installed\n", r.Name)
				}
			}
			return nil
		},
	}
	cmd.Flags().BoolVar(&jsonOut, "json", false, "Output as JSON")
	return cmd
}
