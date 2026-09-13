package cli

import (
	"encoding/json"
	"fmt"
	"io"
	"os"
	"path/filepath"

	"github.com/meowshed/meowctl/internal/lock"
	"github.com/meowshed/meowctl/internal/state"
	"github.com/meowshed/meowctl/internal/tui"
	"github.com/spf13/cobra"
)

func newDoctorCmd(gf *globalFlags) *cobra.Command {
	var jsonOut bool
	cmd := &cobra.Command{
		Use:   "doctor",
		Short: "Check the meowctl configuration and environment for problems",
		Args:  cobra.NoArgs,
		RunE: func(cmd *cobra.Command, _ []string) error {
			configDir, err := resolveConfigDir(gf)
			if err != nil {
				return err
			}
			return runDoctor(cmd.OutOrStdout(), configDir, jsonOut)
		},
	}
	cmd.Flags().BoolVar(&jsonOut, "json", false, "Output results as JSON")
	return cmd
}

type doctorCheck struct {
	Name   string `json:"name"`
	Status string `json:"status"` // ok | warn | error
	Detail string `json:"detail,omitempty"`
}

func runDoctor(w io.Writer, configDir string, jsonOut bool) error {
	var checks []doctorCheck

	// Check 1: init.star exists.
	starPath := filepath.Join(configDir, configEntryFile)
	if _, _, _, err := loadComponentsWithDeps(configDir, nil, true); err != nil {
		checks = append(checks, doctorCheck{"init.star", "error", err.Error()})
	} else {
		checks = append(checks, doctorCheck{"init.star", "ok", starPath})
	}

	// Check 2: deps.lock readable.
	lockPath := filepath.Join(configDir, configLockFile)
	if lf, err := lock.Read(lockPath); err != nil {
		checks = append(checks, doctorCheck{"lock-file", "warn", fmt.Sprintf("not found or unreadable: %v", err)})
	} else {
		checks = append(checks, doctorCheck{"lock-file", "ok", fmt.Sprintf("%d module(s)", len(lf.Modules))})
	}

	// Check 3: sentinel state readable.
	statePath := filepath.Join(configDir, configStateFile)
	sm := state.NewManager(statePath)
	if _, err := sm.Load(); err != nil {
		checks = append(checks, doctorCheck{"state", "warn", fmt.Sprintf("unreadable: %v", err)})
	} else {
		checks = append(checks, doctorCheck{"state", "ok", statePath})
	}

	// Check 4: hook-error flag file.
	if hookErrorExists(configDir) {
		hookErrPath := filepath.Join(configDir, configHookErrorFile)
		data, readErr := os.ReadFile(hookErrPath) // #nosec G304
		detail := "hook eval failed (see .hook-error for details)"
		if readErr == nil {
			detail = string(data)
		}
		checks = append(checks, doctorCheck{"hook-error", "warn", detail})
	}

	if jsonOut {
		out, _ := json.MarshalIndent(checks, "", "  ")
		_, _ = fmt.Fprintln(w, string(out))
		return nil
	}

	// Hand-rolled glyphs here were the last holdout of the old per-command
	// formatting; the shared theme now supplies them, so doctor degrades on a
	// monochrome or non-UTF-8 terminal like everything else.
	p := tui.NewPrinter(w, nil)
	hasError := false
	items := make([]tui.ItemSpec, 0, len(checks))
	for _, c := range checks {
		status := tui.StatusSuccess
		switch c.Status {
		case "warn":
			status = tui.StatusWarning
		case "error":
			status = tui.StatusFailure
			hasError = true
		}
		items = append(items, tui.ItemSpec{Status: status, Label: c.Name, Note: c.Detail})
	}
	p.ItemList(items)
	if hasError {
		return exitErrorf(ExitConfig, "doctor: one or more checks failed")
	}
	return nil
}
