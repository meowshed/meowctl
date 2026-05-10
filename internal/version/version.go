// Package version provides build-time version information for meowctl.
package version

import (
	"fmt"
	"runtime"
)

// Build-time variables injected via -ldflags.
var (
	Version   = "dev"
	Commit    = "unknown"
	BuildDate = "unknown"
)

// String returns the full version string.
// Format: meowctl <version> (<goos>/<goarch>, commit <short>, built <date>)
func String() string {
	return fmt.Sprintf(
		"meowctl %s (%s/%s, commit %s, built %s)",
		Version,
		runtime.GOOS,
		runtime.GOARCH,
		Commit,
		BuildDate,
	)
}
