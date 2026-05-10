package cli

import (
	"fmt"
)

// errNotImplemented returns a user-facing error for unimplemented commands.
func errNotImplemented(cmd string) error {
	return fmt.Errorf("command %q is not yet implemented", cmd)
}
