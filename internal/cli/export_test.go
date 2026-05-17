// Package cli export_test.go exposes internal functions for package-level tests.
// This file is only compiled during testing.
package cli

import (
	"github.com/meowshed/meowctl/internal/lifecycle"
	"github.com/spf13/cobra"
)

// ExportDefaultConfigDir exposes defaultConfigDir for tests.
var ExportDefaultConfigDir = defaultConfigDir

// ExportLoadComponents exposes loadComponentsWithDeps for tests (install path: synthetic deps included).
func ExportLoadComponents(configDir string, filter []string) ([]lifecycle.ComponentID, error) {
	ids, _, _, err := loadComponentsWithDeps(configDir, filter, true)
	return ids, err
}

// ExportLoadComponentsForUninstall exposes loadComponentsWithDeps for tests (uninstall path: synthetic deps excluded).
func ExportLoadComponentsForUninstall(configDir string, filter []string) ([]lifecycle.ComponentID, error) {
	ids, _, _, err := loadComponentsWithDeps(configDir, filter, false)
	return ids, err
}

// NewRootCmdForTest returns a fresh root command for use in tests.
// This avoids sharing the global RootCmd singleton across test runs.
func NewRootCmdForTest() *cobra.Command {
	return newRootCmd()
}
