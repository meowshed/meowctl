package lock

import (
	"bytes"
	"fmt"
	"os"
	"path/filepath"

	"github.com/BurntSushi/toml"
)

// Write encodes lf as TOML and atomically replaces the file at path.
// Atomicity is achieved by writing to a sibling temp file and renaming;
// this prevents a partially-written lock file from being observed by
// concurrent readers or a process that crashes mid-write.
func Write(path string, lf *LockFile) error {
	dir := filepath.Dir(path)
	//nolint:gosec // 0o755 is intentional: config dirs must be user-readable/executable.
	if err := os.MkdirAll(dir, 0o755); err != nil {
		return fmt.Errorf("lock: creating parent dir for %s: %w", path, err)
	}

	var buf bytes.Buffer
	enc := toml.NewEncoder(&buf)
	if err := enc.Encode(lf); err != nil {
		return fmt.Errorf("lock: encoding lock file: %w", err)
	}

	// Write to a temp file in the same directory so rename is atomic on POSIX.
	tmp, err := os.CreateTemp(dir, ".meowctl-lock-*.tmp")
	if err != nil {
		return fmt.Errorf("lock: creating temp file: %w", err)
	}
	tmpName := tmp.Name()

	if _, err := tmp.Write(buf.Bytes()); err != nil {
		_ = tmp.Close()
		_ = os.Remove(tmpName) //nolint:gosec // tmpName is produced by os.CreateTemp; not tainted input.
		return fmt.Errorf("lock: writing temp file: %w", err)
	}
	if err := tmp.Close(); err != nil {
		_ = os.Remove(tmpName) //nolint:gosec // tmpName is produced by os.CreateTemp; not tainted input.
		return fmt.Errorf("lock: closing temp file: %w", err)
	}
	//nolint:gosec // path comes from filepath.Dir of a caller-supplied config path; not tainted user input.
	if err := os.Rename(tmpName, path); err != nil {
		_ = os.Remove(tmpName) //nolint:gosec // tmpName is produced by os.CreateTemp; not tainted input.
		return fmt.Errorf("lock: renaming temp file to %s: %w", path, err)
	}
	return nil
}
