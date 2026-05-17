package cli

import (
	"fmt"
	"os"
	"path/filepath"
)

// Config file name constants. All file references MUST use these constants;
// no raw filename string literals are permitted elsewhere.
const (
	configEntryFile     = "init.star"
	configLocalFile     = "local.star"
	configModFile       = "deps.mod"
	configLockFile      = "deps.lock"
	configInstalledFile = "installed.lock"
)

// defaultConfigDir returns the default meowctl config directory.
// It honours XDG_CONFIG_HOME when set, otherwise uses ~/.config/meowctl.
func defaultConfigDir() (string, error) {
	if xdg := os.Getenv("XDG_CONFIG_HOME"); xdg != "" {
		return filepath.Join(xdg, "meowctl"), nil
	}
	home, err := os.UserHomeDir()
	if err != nil {
		return "", fmt.Errorf("meowctl: cannot determine home directory: %w", err)
	}
	return filepath.Join(home, ".config", "meowctl"), nil
}
