package cli

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/meowshed/meowctl/internal/pkg"
	starlarkpkg "github.com/meowshed/meowctl/internal/starlark"
	"github.com/spf13/cobra"
)

// newCheckCmd returns the `meowctl check <dir>` subcommand.
// It structurally validates every .star file in <dir> by evaluating it with
// ReadComponentGlobals and scanning its globals with ScanGlobals. Files that
// declare pm_name but are missing required PM functions are reported as errors.
// Exit 0 if no errors, non-zero otherwise.
func newCheckCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "check <dir>",
		Short: "Validate component .star files in a directory",
		Args:  cobra.ExactArgs(1),
		RunE: func(_ *cobra.Command, args []string) error {
			dir := args[0]
			entries, err := os.ReadDir(dir)
			if err != nil {
				return fmt.Errorf("check: read dir %q: %w", dir, err)
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
			return exitErrorf(ExitConfig, "check: %d error(s) found", len(errs))
		},
	}
}
