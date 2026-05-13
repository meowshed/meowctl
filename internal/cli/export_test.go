// Package cli export_test.go exposes internal functions for package-level tests.
// This file is only compiled during testing.
package cli

import (
	"github.com/meowshed/meowctl/internal/lifecycle"
	"github.com/spf13/cobra"
)

// ExportDefaultConfigDir exposes defaultConfigDir for tests.
var ExportDefaultConfigDir = defaultConfigDir

// ExportLoadComponents exposes loadComponents for tests.
func ExportLoadComponents(configDir string, filter []string) ([]lifecycle.ComponentID, error) {
	return loadComponents(configDir, filter)
}

// NewRootCmdForTest returns a fresh root command for use in tests.
// This avoids sharing the global RootCmd singleton across test runs.
func NewRootCmdForTest() *cobra.Command {
	return newRootCmd()
}
