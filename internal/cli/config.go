package cli

import (
	"fmt"
	"os"
	"path/filepath"
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
