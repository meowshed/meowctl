package cli

import (
	"fmt"
	"io/fs"
	"os"
	"path/filepath"
	"strings"

	"github.com/meowshed/meowctl/internal/pkg"
	starlarkpkg "github.com/meowshed/meowctl/internal/starlark"
	"github.com/meowshed/meowctl/internal/tui"
	"github.com/spf13/cobra"
)

// componentNameFor derives the name reported for a .star file. Components are
// laid out as <dir>/<component>/init.star, so an init.star is named after the
// directory holding it; a bare .star file is named after itself.
func componentNameFor(path string) string {
	base := filepath.Base(path)
	if base == "init.star" {
		return filepath.Base(filepath.Dir(path))
	}
	return base
}

// newCheckCmd returns the `meowctl check <dir>` subcommand.
// It walks <dir> recursively and structurally validates every .star file by
// evaluating it with ReadComponentGlobals and scanning its globals with
// ScanGlobals. Files that declare pm_name but are missing required PM
// functions are reported as errors. Exit 0 if no errors, non-zero otherwise.
func newCheckCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "check <dir>",
		Short: "Validate component .star files in a directory tree",
		Args:  cobra.ExactArgs(1),
		RunE: func(_ *cobra.Command, args []string) error {
			dir := args[0]
			if _, err := os.Stat(dir); err != nil {
				return fmt.Errorf("check: read dir %q: %w", dir, err)
			}

			eval := &starlarkpkg.Evaluator{}
			type fileError struct {
				file   string
				reason string
			}
			var errs []fileError
			checked := 0

			walkErr := filepath.WalkDir(dir, func(path string, d fs.DirEntry, err error) error {
				if err != nil {
					return err
				}
				if d.IsDir() {
					// Skip dotted directories (.git, .github) so they cannot
					// contribute stray .star files to the report.
					if path != dir && strings.HasPrefix(d.Name(), ".") {
						return fs.SkipDir
					}
					return nil
				}
				if !strings.HasSuffix(d.Name(), ".star") {
					return nil
				}
				checked++
				result, evalErr := eval.ReadComponentGlobals(path, nil)
				if evalErr != nil {
					errs = append(errs, fileError{path, evalErr.Error()})
					return nil
				}
				var warnMsg string
				pkg.ScanGlobals(componentNameFor(path), result.Globals, func(msg string) {
					warnMsg = msg
				})
				if warnMsg != "" {
					errs = append(errs, fileError{path, warnMsg})
				}
				return nil
			})
			if walkErr != nil {
				return fmt.Errorf("check: walk dir %q: %w", dir, walkErr)
			}

			if len(errs) == 0 {
				tui.Note("Checked %d file(s), no errors.", checked)
				return nil
			}
			p := tui.NewPrinter(nil, nil)
			p.Heading("Checked %d file(s), %d error(s)", checked, len(errs))
			for _, e := range errs {
				p.Failure(e.file, e.reason)
			}
			return exitErrorf(ExitConfig, "check: %d error(s) found", len(errs))
		},
	}
}
