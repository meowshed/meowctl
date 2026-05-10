package cli

import (
	"fmt"
)

// errNotImplemented returns a user-facing error for unimplemented commands.
// TODO: remove once all commands are implemented.
func errNotImplemented(cmd string) error {
	return fmt.Errorf("command %q is not yet implemented", cmd)
}
