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
	"context"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/meowshed/meowctl/internal/ctx"
	"github.com/meowshed/meowctl/internal/pkg"
	starlarkpkg "github.com/meowshed/meowctl/internal/starlark"
	gostarlark "go.starlark.net/starlark"
)

// stubRunFunc is a RunFunc that returns plausible empty output per command,
// allowing interrogate hooks to succeed without spawning real subprocesses.
// Each command gets a response shaped to match what the real binary would return
// for an empty/fresh install state.
func stubRunFunc(_ context.Context, cmd string, args []string, _ []string) (string, error) {
	// Dispatch on the command name and first argument to return the right shape.
	switch cmd {
	case "mise":
		if len(args) > 0 && args[0] == "ls" {
			// mise ls --installed --json → empty JSON object (no tools installed)
			return "{}", nil
		}
	case "npm":
		// npm list -g --depth=0 --parseable → just the global prefix, no packages
		return "/usr/local/lib\n", nil
	case "gem":
		// gem list --local --no-versions → empty (no gems beyond stdlib header)
		return "", nil
	case "cargo":
		// cargo install --list → empty (no crates installed)
		return "", nil
	case "go":
		if len(args) > 0 && args[0] == "env" {
			// go env GOPATH → a plausible GOPATH
			return "/root/go\n", nil
		}
	case "uv":
		// uv tool list → empty (no tools installed); no header on empty output
		return "", nil
	case "pipx":
		// pipx list --json → empty venvs object
		return `{"venvs": {}}`, nil
	case "dpkg-query":
		// dpkg-query -f '${Package}\n' -W → empty
		return "", nil
	case "dnf":
		// dnf --quiet repoquery --installed --qf '%{name}\n' → empty
		return "", nil
	case "pacman":
		// pacman -Qq → empty
		return "", nil
	case "apk":
		// apk list --installed → empty
		return "", nil
	case "mas":
		// mas list → empty
		return "", nil
	}
	return "", nil
}

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

			// Build a PMRegistry with a stub mise handler so that PM components
			// that delegate to mise via query_pm("mise") (e.g. github_release.star)
			// can resolve the handler without spawning real processes.
			// The stub interrogate returns an empty list, valid for a fresh install state.
			stubMiseInterrogate := gostarlark.NewBuiltin("interrogate", func(
				_ *gostarlark.Thread, _ *gostarlark.Builtin,
				_ gostarlark.Tuple, _ []gostarlark.Tuple,
			) (gostarlark.Value, error) {
				return gostarlark.NewList(nil), nil
			})
			registry := pkg.NewPMRegistry()
			registry.Register("mise", &pkg.PMHandler{
				ComponentName: "mise.star",
				InstallPkg: gostarlark.NewBuiltin("install_pkg", func(_ *gostarlark.Thread, _ *gostarlark.Builtin, _ gostarlark.Tuple, _ []gostarlark.Tuple) (gostarlark.Value, error) {
					return gostarlark.None, nil
				}),
				UninstallPkg: gostarlark.NewBuiltin("uninstall_pkg", func(_ *gostarlark.Thread, _ *gostarlark.Builtin, _ gostarlark.Tuple, _ []gostarlark.Tuple) (gostarlark.Value, error) {
					return gostarlark.None, nil
				}),
				Interrogate: stubMiseInterrogate,
			})

			// Build a ctx with RunFunc and PMRegistry injected so interrogate hooks
			// that call ctx.run() or query_pm() succeed without real subprocesses.
			ctxVal := ctx.New(&ctx.Capabilities{
				RunFunc:    stubRunFunc,
				PMRegistry: registry,
			})

			thread := &gostarlark.Thread{Name: "interrogate/" + name}
			thread.SetLocal("ctx", ctxVal)
			res, callErr := gostarlark.Call(thread, h.Interrogate, gostarlark.Tuple{ctxVal}, nil)
			if callErr != nil {
				t.Fatalf("interrogate(%q): %v", name, callErr)
			}
			if _, ok := res.(*gostarlark.List); !ok {
				t.Errorf("interrogate(%q) must return a list, got %s", name, res.Type())
			}
		})
	}
}
