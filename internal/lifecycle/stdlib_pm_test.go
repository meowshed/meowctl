//go:build integration

// Package lifecycle_test — stdlib PM integration tests.
// These tests require STDLIB_PATH to be set to a checkout of meowshed/meowctl-stdlib.
// They are excluded from normal `go test ./...` runs and are executed by stdlib CI:
//
//	go test -tags integration -count=1 -run TestStdlib ./internal/lifecycle/...
//	  env:
//	    STDLIB_PATH: /path/to/meowctl-stdlib
//
// Gates on: all 19 PM .star files parse cleanly, declare required globals
// (pm_name, install_pkg, uninstall_pkg, interrogate), and interrogate returns
// a non-nil list.
package lifecycle_test

import (
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/meowshed/meowctl/internal/pkg"
	starlarkpkg "github.com/meowshed/meowctl/internal/starlark"
	gostarlark "go.starlark.net/starlark"
)

// TestStdlib_PMFilesStructuralCheck validates that every .star file under
// $STDLIB_PATH/components/ parses cleanly and, if it declares pm_name, exports
// all required PM globals (install_pkg, uninstall_pkg, interrogate).
//
// This mirrors `meowctl component check ./components/` but runs as a Go test
// so stdlib CI can use it with the -tags integration gate.
func TestStdlib_PMFilesStructuralCheck(t *testing.T) {
	stdlibPath := os.Getenv("STDLIB_PATH")
	if stdlibPath == "" {
		t.Skip("STDLIB_PATH not set; skipping stdlib integration tests")
	}

	componentsDir := filepath.Join(stdlibPath, "components")
	entries, err := os.ReadDir(componentsDir)
	if err != nil {
		t.Fatalf("read components dir %q: %v", componentsDir, err)
	}

	eval := &starlarkpkg.Evaluator{}

	for _, entry := range entries {
		if entry.IsDir() {
			continue
		}
		name := entry.Name()
		if !strings.HasSuffix(name, ".star") {
			continue
		}
		t.Run(name, func(t *testing.T) {
			filePath := filepath.Join(componentsDir, name)
			result, evalErr := eval.ReadComponentGlobals(filePath, nil)
			if evalErr != nil {
				t.Fatalf("eval %q: %v", name, evalErr)
			}

			var warnMsg string
			h := pkg.ScanGlobals(name, result.Globals, func(msg string) {
				warnMsg = msg
			})
			if warnMsg != "" {
				t.Errorf("ScanGlobals: %s", warnMsg)
			}

			// If this file registered a PM handler, verify interrogate returns a list.
			if h == nil {
				return
			}
			thread := &gostarlark.Thread{Name: "interrogate/" + name}
			res, callErr := gostarlark.Call(thread, h.Interrogate, gostarlark.Tuple{gostarlark.None}, nil)
			if callErr != nil {
				t.Fatalf("interrogate(%q): %v", name, callErr)
			}
			if _, ok := res.(*gostarlark.List); !ok {
				t.Errorf("interrogate(%q) must return a list, got %s", name, res.Type())
			}
		})
	}
}
