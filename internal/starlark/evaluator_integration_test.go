package starlark_test

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/meowshed/meowctl/internal/ctx"
	"github.com/meowshed/meowctl/internal/starlark"
)

// testFixtureCaps returns a Capabilities suitable for integration tests.
func testFixtureCaps() *ctx.Capabilities {
	return &ctx.Capabilities{
		Home:         "/home/user",
		ComponentDir: "/home/user/.config/meowctl/components/git",
		StateDir:     "/home/user/.local/share/meowctl/_local/git",
		Env:          map[string]string{},
	}
}

// execFixtureWithCtx evaluates testdata/component.star with a real CtxValue and returns
// the evaluator, the result, and the ctx. No loader is configured — dep() records URLs
// without resolving them, and load() statements would fail.
func execFixtureWithCtx(t *testing.T) (*starlark.Evaluator, *starlark.EvalResult, *ctx.CtxValue) {
	t.Helper()
	fixturePath := filepath.Join("testdata", "component.star")
	src, err := os.ReadFile(fixturePath) //nolint:gosec // test reads a known local fixture
	if err != nil {
		t.Fatalf("read fixture: %v", err)
	}
	e := &starlark.Evaluator{Platform: starlark.PlatformInfo{OS: "linux"}}
	ctxVal := ctx.New(testFixtureCaps())
	result, err := e.ExecFile(fixturePath, src, nil, ctxVal)
	if err != nil {
		t.Fatalf("ExecFile: %v", err)
	}
	return e, result, ctxVal
}

// execFixture evaluates testdata/component.star and returns the result.
func execFixture(t *testing.T) *starlark.EvalResult {
	t.Helper()
	_, result, _ := execFixtureWithCtx(t)
	return result
}

// TestIntegration_Accumulator_Module verifies that module() populates the accumulator.
func TestIntegration_Accumulator_Module(t *testing.T) {
	result := execFixture(t)
	acc := result.Declarations
	if acc.Module == nil {
		t.Fatal("accumulator: module not declared")
	}
	if acc.Module.Name != "fixture" || acc.Module.Version != "1.0.0" {
		t.Errorf("module: got name=%q version=%q, want fixture/1.0.0", acc.Module.Name, acc.Module.Version)
	}
}

// TestIntegration_Accumulator_Component verifies that component() populates the accumulator.
func TestIntegration_Accumulator_Component(t *testing.T) {
	result := execFixture(t)
	acc := result.Declarations
	if len(acc.Components) != 1 || acc.Components[0].Name != "git" {
		t.Errorf("components: got %+v, want [{Name:git}]", acc.Components)
	}
}

// TestIntegration_Accumulator_Packages verifies that pkg() populates the accumulator.
func TestIntegration_Accumulator_Packages(t *testing.T) {
	result := execFixture(t)
	acc := result.Declarations
	if len(acc.Packages) != 2 {
		t.Fatalf("packages: got %d, want 2", len(acc.Packages))
	}
	if acc.Packages[0].Manager != "apt" || acc.Packages[0].Name != "git" {
		t.Errorf("packages[0]: got %+v, want {Manager:apt Name:git}", acc.Packages[0])
	}
	if acc.Packages[1].Manager != "brew" || acc.Packages[1].Name != "git" {
		t.Errorf("packages[1]: got %+v, want {Manager:brew Name:git}", acc.Packages[1])
	}
}

// TestIntegration_Accumulator_Dep verifies that dep() populates the accumulator.
func TestIntegration_Accumulator_Dep(t *testing.T) {
	result := execFixture(t)
	acc := result.Declarations
	if len(acc.Deps) != 1 || acc.Deps[0].URL != "self//lib.star" {
		t.Errorf("deps: got %+v, want [{URL:self//lib.star}]", acc.Deps)
	}
}

// TestIntegration_Accumulator_Select verifies that select() records the matched case.
func TestIntegration_Accumulator_Select(t *testing.T) {
	result := execFixture(t)
	acc := result.Declarations
	if len(acc.SelectCases) != 1 {
		t.Fatalf("select_cases: got %d, want 1", len(acc.SelectCases))
	}
	if acc.SelectCases[0].Condition != "//platform:linux" {
		t.Errorf("select_cases[0].Condition = %q, want //platform:linux", acc.SelectCases[0].Condition)
	}
}

// TestIntegration_Hooks verifies that install and setup hooks are callable with a real CtxValue.
func TestIntegration_Hooks(t *testing.T) {
	fixturePath := filepath.Join("testdata", "component.star")
	e, result, ctxVal := execFixtureWithCtx(t)

	if err := e.CallHook(result.Globals, "install", fixturePath, ctxVal); err != nil {
		t.Errorf("CallHook install: %v", err)
	}
	if err := e.CallHook(result.Globals, "setup", fixturePath, ctxVal); err != nil {
		t.Errorf("CallHook setup: %v", err)
	}
}
