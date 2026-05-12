// Smoke tests run end-to-end lifecycle tests against the testdata/smoke fixture.
// These tests exercise the full pipeline: Starlark eval → hook dispatch →
// ctx methods → rollback tracking.
package lifecycle_test

import (
	"errors"
	"os"
	"path/filepath"
	"testing"

	"github.com/meowshed/meowctl/internal/ctx"
	"github.com/meowshed/meowctl/internal/lifecycle"
	"github.com/meowshed/meowctl/internal/rollback"
	starlarkpkg "github.com/meowshed/meowctl/internal/starlark"
)

// smokeHookCaller executes hooks from the testdata/smoke fixture.
type smokeHookCaller struct {
	dotfilesDir string
	eval        *starlarkpkg.Evaluator
	dryRun      bool
}

func (h *smokeHookCaller) CallHook(componentID, hookName string) error {
	file := filepath.Join(h.dotfilesDir, componentID+".star")
	// Note: unlike the production starlarkHookCaller, this test helper does not
	// stat-guard the file path — tests are responsible for ensuring the fixture exists.
	caps := &ctx.Capabilities{
		DryRun:       h.dryRun,
		ComponentDir: filepath.Dir(file),
		Phase:        hookName,
		Component:    componentID,
		Env:          map[string]string{},
	}
	ctxVal := ctx.New(caps)
	result, err := h.eval.ExecFile(file, nil, nil, ctxVal)
	if err != nil {
		return err
	}
	return h.eval.CallHook(result.Globals, hookName, file, ctxVal)
}

// TestSmoke_DryRunInstall verifies that meowctl install --dry-run completes
// without error and without writing any files.
func TestSmoke_DryRunInstall(t *testing.T) {
	fixtureDir := filepath.Join("..", "..", "testdata", "smoke", "components")
	caller := &smokeHookCaller{
		dotfilesDir: fixtureDir,
		eval:        &starlarkpkg.Evaluator{},
		dryRun:      true,
	}
	runner := &lifecycle.Runner{
		Order:  []lifecycle.ComponentID{"smoke"},
		Caller: caller,
		// No rollback stack in dry-run.
	}
	if err := runner.RunPhaseSet("install", lifecycle.PhaseSetInstall); err != nil {
		t.Fatalf("dry-run install: %v", err)
	}
}

// failAfterN is a HookCaller that succeeds for the first n calls then fails.
type failAfterN struct {
	n     int
	calls int
}

func (f *failAfterN) CallHook(_, _ string) error {
	f.calls++
	if f.calls > f.n {
		return errors.New("simulated component failure")
	}
	return nil
}

// TestRunner_MidPhaseFailure verifies that when a component fails mid-phase
// the runner returns a *PhaseError and the rollback stack is executed.
func TestRunner_MidPhaseFailure(t *testing.T) {
	dir := t.TempDir()
	journalPath := filepath.Join(dir, "rollback.jsonl")
	stack, err := rollback.Open(journalPath)
	if err != nil {
		t.Fatalf("open stack: %v", err)
	}
	defer func() { _ = stack.Close() }()

	// Write a file and record the inverse so rollback can delete it.
	filePath := filepath.Join(dir, "written.txt")
	if err := stack.AppendWriteFile("install", "comp-a", filePath, "", false); err != nil {
		t.Fatalf("AppendWriteFile: %v", err)
	}
	if err := os.WriteFile(filePath, []byte("created"), 0o600); err != nil {
		t.Fatal(err)
	}

	// 2-component order: comp-a (already tracked above), comp-b fails.
	caller := &failAfterN{n: 1} // comp-a succeeds, comp-b fails
	runner := &lifecycle.Runner{
		Order:  []lifecycle.ComponentID{"comp-a", "comp-b"},
		Caller: caller,
		Stack:  stack,
	}

	err = runner.RunPhaseSet("install", []lifecycle.Phase{lifecycle.PhaseInstall})
	if err == nil {
		t.Fatal("expected error from mid-phase failure, got nil")
	}
	var phaseErr *lifecycle.PhaseError
	if !errors.As(err, &phaseErr) {
		t.Fatalf("expected *PhaseError, got %T: %v", err, err)
	}
	if len(phaseErr.Failures) != 1 || phaseErr.Failures[0].Component != "comp-b" {
		t.Errorf("unexpected failures: %v", phaseErr.Failures)
	}

	// Rollback should have deleted the file written by comp-a.
	if _, err := os.Stat(filePath); !os.IsNotExist(err) {
		t.Errorf("expected rollback to delete %s", filePath)
	}
}

func TestSmoke_DryRunUninstall(t *testing.T) {
	fixtureDir := filepath.Join("..", "..", "testdata", "smoke", "components")
	caller := &smokeHookCaller{
		dotfilesDir: fixtureDir,
		eval:        &starlarkpkg.Evaluator{},
		dryRun:      true,
	}
	runner := &lifecycle.Runner{
		Order:  []lifecycle.ComponentID{"smoke"},
		Caller: caller,
	}
	if err := runner.RunPhaseSet("uninstall", lifecycle.PhaseSetUninstall); err != nil {
		t.Fatalf("dry-run uninstall: %v", err)
	}
}
