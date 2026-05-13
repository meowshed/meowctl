package starlark

import (
	"errors"
	"fmt"

	"github.com/meowshed/meowctl/internal/starlark/loader"
	gostarlark "go.starlark.net/starlark"
	"go.starlark.net/syntax"
)

// Evaluator executes Starlark configuration files.
// It encodes no policy — callers supply the ctx value and platform info.
type Evaluator struct {
	// Loader resolves module load() calls. If nil, load() is not supported.
	Loader *loader.CompositeLoader

	// Platform provides platform information for platform() and select() builtins.
	Platform PlatformInfo

	// Opts configures Starlark syntax parsing. If nil, default options are used.
	Opts *syntax.FileOptions
}

// EvalResult holds the output of a successful ExecFile call.
type EvalResult struct {
	// Globals contains all top-level names defined in the executed file.
	Globals gostarlark.StringDict

	// Declarations holds all declarations collected by predeclared builtins
	// (component, pkg, dep, module, select) during execution.
	Declarations *Accumulator
}

// fileOpts returns the effective syntax options, falling back to a zero-value
// FileOptions when Opts is nil so callers never have to guard against nil.
func (e *Evaluator) fileOpts() *syntax.FileOptions {
	if e.Opts != nil {
		return e.Opts
	}
	return &syntax.FileOptions{}
}

// ExecFile evaluates a Starlark file and returns its globals and accumulated declarations.
//
//   - filename: path or identifier used in error messages and load() resolution.
//   - src: file content (string or []byte); if nil, filename is read from disk.
//   - predeclared: additional predeclared names injected alongside builtins (must not shadow
//     builtin names such as "component", "pkg", "dep", "module", "select", "platform").
//   - ctx: the ctx object passed as a thread-local for hook calls; may be nil during plain eval.
func (e *Evaluator) ExecFile(filename string, src any, predeclared gostarlark.StringDict, ctx gostarlark.Value) (*EvalResult, error) {
	acc := &Accumulator{}
	builtins := makePredeclared(e.Platform)

	// Reject caller-supplied names that would shadow built-in names. Silently overwriting
	// builtins (e.g. replacing "component") would cause hard-to-diagnose config bugs.
	for k := range predeclared {
		if _, conflict := builtins[k]; conflict {
			return nil, fmt.Errorf("ExecFile: predeclared name %q shadows a builtin", k)
		}
	}

	merged := make(gostarlark.StringDict, len(builtins)+len(predeclared))
	for k, v := range builtins {
		merged[k] = v
	}
	for k, v := range predeclared {
		merged[k] = v
	}

	thread := &gostarlark.Thread{Name: filename}
	thread.SetLocal("acc", acc)
	if ctx != nil {
		thread.SetLocal("ctx", ctx)
	}

	if e.Loader != nil {
		thread.Load = func(t *gostarlark.Thread, module string) (gostarlark.StringDict, error) {
			return e.Loader.Load(t, module, merged)
		}
	}

	globals, err := gostarlark.ExecFileOptions(e.fileOpts(), thread, filename, src, merged)
	if err != nil {
		var evalErr *gostarlark.EvalError
		if errors.As(err, &evalErr) {
			return nil, &EvalError{Err: evalErr, Filename: filename}
		}
		return nil, &ParseError{Err: err}
	}

	return &EvalResult{Globals: globals, Declarations: acc}, nil
}

// ReadComponentGlobals evaluates a Starlark file to read its top-level globals
// without a ctx value. This is used during pass 1 of component loading to extract
// metadata (after, pm_name) before hooks are executed.
// Returns the globals dict and accumulated declarations, or an error.
func (e *Evaluator) ReadComponentGlobals(filename string, src any) (*EvalResult, error) {
	return e.ExecFile(filename, src, nil, nil)
}

// globals must be the Globals field from an EvalResult. filename is the source file path,
// used in error messages. ctx is passed as the sole argument.
func (e *Evaluator) CallHook(globals gostarlark.StringDict, hookName, filename string, ctx gostarlark.Value) error {
	fn, ok := globals[hookName]
	if !ok {
		// Hook not defined — not an error; hooks are optional.
		return nil
	}
	callable, ok := fn.(gostarlark.Callable)
	if !ok {
		return fmt.Errorf("CallHook: %q is not callable (got %s)", hookName, fn.Type())
	}

	thread := &gostarlark.Thread{Name: hookName}
	if ctx != nil {
		thread.SetLocal("ctx", ctx)
	}

	arg := gostarlark.Value(gostarlark.None)
	if ctx != nil {
		arg = ctx
	}
	_, err := gostarlark.Call(thread, callable, gostarlark.Tuple{arg}, nil)
	if err != nil {
		var evalErr *gostarlark.EvalError
		if errors.As(err, &evalErr) {
			return &HookError{Err: evalErr, HookName: hookName, Filename: filename}
		}
		return fmt.Errorf("CallHook %q: %w", hookName, err)
	}
	return nil
}
