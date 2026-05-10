package lock

import (
	"errors"
	"fmt"
	"os"

	"github.com/BurntSushi/toml"
)

// Read parses the lock file at path and returns a pointer to the decoded LockFile.
// If path does not exist, Read returns an empty LockFile and a nil error — callers
// treat a missing lock file as a clean slate rather than an error condition.
func Read(path string) (*LockFile, error) {
	data, err := os.ReadFile(path) // #nosec G304 -- path is caller-supplied config path, not user-provided taint.
	if err != nil {
		if errors.Is(err, os.ErrNotExist) {
			return &LockFile{}, nil
		}
		return nil, fmt.Errorf("lock: reading %s: %w", path, err)
	}

	var lf LockFile
	if err := toml.Unmarshal(data, &lf); err != nil {
		return nil, fmt.Errorf("lock: decoding %s: %w", path, err)
	}
	return &lf, nil
}
