//go:build integration

package cli_test

// Live integration smoke tests — require real network access to GitHub.
// Run with: go test -tags integration ./internal/cli/...
//
// These tests validate the full resolution pipeline:
// 1. Registry index is reachable and contains meowctl-stdlib.
// 2. The stdlib tarball is downloadable, extracted, and integrity-verified.
// 3. loadComponentsWithDeps resolves @meowctl-stdlib// references end-to-end.

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/meowshed/meowctl/internal/cli"
)

// TestLiveLoadComponents_StdlibGit creates a minimal meowctl.star that declares
// a single @meowctl-stdlib//components/git component and verifies that
// loadComponentsWithDeps resolves it successfully against the live registry.
func TestLiveLoadComponents_StdlibGit(t *testing.T) {
	tmp := t.TempDir()

	star := `
module(name = "test", version = "0.1.0")
component("@meowctl-stdlib//components/git")
`
	if err := os.WriteFile(filepath.Join(tmp, "meowctl.star"), []byte(star), 0o600); err != nil {
		t.Fatal(err)
	}

	ids, err := cli.ExportLoadComponents(tmp, nil)
	if err != nil {
		t.Fatalf("loadComponentsWithDeps failed: %v", err)
	}

	found := false
	for _, id := range ids {
		if string(id) == "git" {
			found = true
			break
		}
	}
	if !found {
		t.Fatalf("expected component 'git' in resolved ids, got %v", ids)
	}
	t.Logf("@meowctl-stdlib//components/git resolved successfully — ids: %v", ids)
}
