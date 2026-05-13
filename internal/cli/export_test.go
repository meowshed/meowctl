// Package cli export_test.go exposes internal functions for package-level tests.
// This file is only compiled during testing.
package cli

import "github.com/meowshed/meowctl/internal/lifecycle"

// ExportDefaultConfigDir exposes defaultConfigDir for tests.
var ExportDefaultConfigDir = defaultConfigDir

// ExportLoadComponents exposes loadComponents for tests.
func ExportLoadComponents(configDir string, filter []string) ([]lifecycle.ComponentID, error) {
	return loadComponents(configDir, filter)
}
