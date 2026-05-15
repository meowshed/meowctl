package cli

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/meowshed/meowctl/internal/pkg"
	starlarkpkg "github.com/meowshed/meowctl/internal/starlark"
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
		newComponentCheckCmd(),
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

// newComponentCheckCmd returns the `meowctl component check <dir>` subcommand.
// It structurally validates every .star file in <dir> by evaluating it with
// ExecFile and scanning its globals with ScanGlobals. Files that declare pm_name
// but are missing required PM functions are reported as errors.
// Exit 0 if no errors, non-zero otherwise.
func newComponentCheckCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "check <dir>",
		Short: "Validate component .star files in a directory",
		Args:  cobra.ExactArgs(1),
		RunE: func(_ *cobra.Command, args []string) error {
			dir := args[0]
			entries, err := os.ReadDir(dir)
			if err != nil {
				return fmt.Errorf("component check: read dir %q: %w", dir, err)
			}

			eval := &starlarkpkg.Evaluator{}
			type fileError struct {
				file   string
				reason string
			}
			var errs []fileError
			checked := 0

			for _, entry := range entries {
				if entry.IsDir() {
					continue
				}
				name := entry.Name()
				if !strings.HasSuffix(name, ".star") {
					continue
				}
				checked++
				filePath := filepath.Join(dir, name)
				result, evalErr := eval.ReadComponentGlobals(filePath, nil)
				if evalErr != nil {
					errs = append(errs, fileError{filePath, evalErr.Error()})
					continue
				}
				var warnMsg string
				pkg.ScanGlobals(name, result.Globals, func(msg string) {
					warnMsg = msg
				})
				if warnMsg != "" {
					errs = append(errs, fileError{filePath, warnMsg})
				}
			}

			if len(errs) == 0 {
				fmt.Printf("Checked %d files. 0 errors.\n", checked)
				return nil
			}
			fmt.Printf("Checked %d files. %d errors:\n", checked, len(errs))
			for _, e := range errs {
				fmt.Printf("  %s: %s\n", e.file, e.reason)
			}
			return exitErrorf(ExitConfig, "component check: %d error(s) found", len(errs))
		},
	}
}
