package cli

import (
	"fmt"
)

// ExitError wraps an error with a specific process exit code.
// main.go inspects this type to call os.Exit with the correct code.
type ExitError struct {
	Code int
	Err  error
}

func (e *ExitError) Error() string {
	return e.Err.Error()
}

func (e *ExitError) Unwrap() error {
	return e.Err
}

// exitErrorf constructs an ExitError with a formatted message.
func exitErrorf(code int, format string, args ...any) *ExitError {
	return &ExitError{Code: code, Err: fmt.Errorf(format, args...)}
}

// Exit code constants per spec.
const (
	ExitSuccess = 0
	ExitGeneral = 1 // general error
	ExitUsage   = 2 // usage error
	ExitConfig  = 3 // invalid meowctl.star / config error
	ExitModule  = 4 // module fetch / SRI failure
)
