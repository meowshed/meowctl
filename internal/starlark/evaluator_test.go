package starlark_test

import (
	"errors"
	"os"
	"path/filepath"
	"testing"

	gostarlark "go.starlark.net/starlark"
	"go.starlark.net/syntax"

	"github.com/meowshed/meowctl/internal/starlark"
	"github.com/meowshed/meowctl/internal/starlark/loader"
)

// newEvaluator returns an Evaluator with no loader and the given platform.
func newEvaluator(p starlark.PlatformInfo) *starlark.Evaluator {
	return &starlark.Evaluator{Platform: p}
}

// newEvaluatorWithLoader returns an Evaluator backed by a CompositeLoader rooted at root.
func newEvaluatorWithLoader(root string, p starlark.PlatformInfo) *starlark.Evaluator {
	return &starlark.Evaluator{
		Loader:   loader.NewCompositeLoader(root, &syntax.FileOptions{}, loader.CompositeLoaderOptions{}),
		Platform: p,
	}
}

// writeStarFile writes a .star file in dir and returns its path.
func writeStarFile(t *testing.T, dir, name, content string) string {
	t.Helper()
	path := filepath.Join(dir, name)
	if err := os.WriteFile(path, []byte(content), 0o600); err != nil {
		t.Fatalf("writeStarFile: %v", err)
	}
	return path
}

// TestExecFile_HappyPath verifies that a simple file evaluates and returns globals.
func TestExecFile_HappyPath(t *testing.T) {
	e := newEvaluator(starlark.PlatformInfo{OS: "linux"})
	result, err := e.ExecFile("test.star", `x = 1 + 1`, nil, nil)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	v, ok := result.Globals["x"]
	if !ok {
		t.Fatal("expected 'x' in globals")
	}
	if v.String() != "2" {
		t.Errorf("expected x=2, got %v", v)
	}
}

// TestExecFile_ParseError verifies that syntax errors produce a ParseError.
func TestExecFile_ParseError(t *testing.T) {
	e := newEvaluator(starlark.PlatformInfo{})
	_, err := e.ExecFile("bad.star", `def unclosed(`, nil, nil)
	if err == nil {
		t.Fatal("expected parse error")
	}
	var pe *starlark.ParseError
	if !errors.As(err, &pe) {
		t.Errorf("expected *ParseError, got %T: %v", err, err)
	}
}

// TestExecFile_EvalError verifies that runtime errors produce an EvalError.
func TestExecFile_EvalError(t *testing.T) {
	e := newEvaluator(starlark.PlatformInfo{})
	_, err := e.ExecFile("runtime.star", `x = 1 // 0`, nil, nil)
	if err == nil {
		t.Fatal("expected eval error")
	}
	var ee *starlark.EvalError
	if !errors.As(err, &ee) {
		t.Errorf("expected *EvalError, got %T: %v", err, err)
	}
}

// TestExecFile_Accumulator verifies that builtins accumulate declarations on the result.
func TestExecFile_Accumulator(t *testing.T) {
	e := newEvaluator(starlark.PlatformInfo{OS: "linux"})
	src := `
module(name="mymod", version="1.0.0")
component(name="shell")
pkg(manager="apt", name="curl")
dep(name="stdlib", version="0.1.0")
`
	result, err := e.ExecFile("meowctl.star", src, nil, nil)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	acc := result.Declarations
	if acc.Module == nil || acc.Module.Name != "mymod" {
		t.Errorf("expected module mymod, got %+v", acc.Module)
	}
	if len(acc.Components) != 1 || acc.Components[0].Name != "shell" {
		t.Errorf("unexpected components: %+v", acc.Components)
	}
	if len(acc.Packages) != 1 || acc.Packages[0].Name != "curl" {
		t.Errorf("unexpected packages: %+v", acc.Packages)
	}
	if len(acc.Deps) != 1 || acc.Deps[0].Name != "stdlib" || acc.Deps[0].Version != "0.1.0" {
		t.Errorf("unexpected deps: %+v", acc.Deps)
	}
}

// TestExecFile_PredeclaredMerge verifies that caller-supplied predeclared names are available.
func TestExecFile_PredeclaredMerge(t *testing.T) {
	e := newEvaluator(starlark.PlatformInfo{})
	predeclared := gostarlark.StringDict{
		"injected": gostarlark.String("hello"),
	}
	result, err := e.ExecFile("merge.star", `out = injected`, predeclared, nil)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if v, ok := result.Globals["out"]; !ok || v.String() != `"hello"` {
		t.Errorf("expected out=\"hello\", got %v", v)
	}
}

// TestExecFile_LoadPropagatesPredeclared verifies that predeclared builtins are available
// in modules loaded via load().
func TestExecFile_LoadPropagatesPredeclared(t *testing.T) {
	dir := t.TempDir()
	writeStarFile(t, dir, "lib.star", `
def get_platform():
    return platform()
`)
	mainPath := writeStarFile(t, dir, "main.star", `
load("self//lib.star", "get_platform")
p = get_platform()
os_name = p.os
`)

	e := newEvaluatorWithLoader(dir, starlark.PlatformInfo{OS: "linux"})
	src, err := os.ReadFile(mainPath) //nolint:gosec // test reads a temp file it just created
	if err != nil {
		t.Fatalf("read main.star: %v", err)
	}
	result, err := e.ExecFile(mainPath, src, nil, nil)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if v, ok := result.Globals["os_name"]; !ok || v.String() != `"linux"` {
		t.Errorf("expected os_name=\"linux\", got %v", v)
	}
}

// TestCallHook_HappyPath verifies that a defined hook function is called without error.
func TestCallHook_HappyPath(t *testing.T) {
	e := newEvaluator(starlark.PlatformInfo{})
	// The hook simply returns None; verifying no error is sufficient.
	result, err := e.ExecFile("hook.star", `
def install(ctx):
    pass
`, nil, nil)
	if err != nil {
		t.Fatalf("ExecFile: %v", err)
	}
	if err := e.CallHook(result.Globals, "install", "test.star", nil); err != nil {
		t.Fatalf("CallHook: %v", err)
	}
}

// TestCallHook_MissingHook verifies that a missing hook is not an error.
func TestCallHook_MissingHook(t *testing.T) {
	e := newEvaluator(starlark.PlatformInfo{})
	result, err := e.ExecFile("noop.star", `x = 1`, nil, nil)
	if err != nil {
		t.Fatalf("ExecFile: %v", err)
	}
	if err := e.CallHook(result.Globals, "nonexistent", "test.star", nil); err != nil {
		t.Errorf("expected nil for missing hook, got %v", err)
	}
}

// TestCallHook_NotCallable verifies that a non-callable hook name returns an error.
func TestCallHook_NotCallable(t *testing.T) {
	e := newEvaluator(starlark.PlatformInfo{})
	result, err := e.ExecFile("notcallable.star", `install = 42`, nil, nil)
	if err != nil {
		t.Fatalf("ExecFile: %v", err)
	}
	if err := e.CallHook(result.Globals, "install", "test.star", nil); err == nil {
		t.Error("expected error for non-callable hook")
	}
}

// TestCallHook_NilCtx verifies that nil ctx is safe (passes None to hook).
func TestCallHook_NilCtx(t *testing.T) {
	e := newEvaluator(starlark.PlatformInfo{})
	result, err := e.ExecFile("nilctx.star", `
def setup(ctx):
    pass
`, nil, nil)
	if err != nil {
		t.Fatalf("ExecFile: %v", err)
	}
	if err := e.CallHook(result.Globals, "setup", "test.star", nil); err != nil {
		t.Fatalf("expected nil error with nil ctx, got %v", err)
	}
}

// TestCallHook_HookError verifies that a hook runtime error produces a HookError.
func TestCallHook_HookError(t *testing.T) {
	e := newEvaluator(starlark.PlatformInfo{})
	result, err := e.ExecFile("hookerr.star", `
def install(ctx):
    1 // 0
`, nil, nil)
	if err != nil {
		t.Fatalf("ExecFile: %v", err)
	}
	err = e.CallHook(result.Globals, "install", "test.star", nil)
	if err == nil {
		t.Fatal("expected error from hook")
	}
	var he *starlark.HookError
	if !errors.As(err, &he) {
		t.Errorf("expected *HookError, got %T: %v", err, err)
	}
}
