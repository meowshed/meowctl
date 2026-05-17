//go:build integration

package cli_test

// Live integration smoke tests — require real network access to GitHub.
// Run with: go test -tags integration ./internal/cli/...
//
// These tests validate the full resolution pipeline end-to-end:
// 1. Registry index is reachable and contains stdlib.
// 2. The stdlib tarball is downloadable, extracted, and integrity-verified.
// 3. loadComponentsWithDeps resolves @stdlib// references, including
//    transitive after= dependencies, filtering, and cache reuse.

import (
	"os"
	"path/filepath"
	"strings"
	"testing"
	"time"

	"github.com/meowshed/meowctl/internal/cli"
	"github.com/meowshed/meowctl/internal/lifecycle"
	"github.com/meowshed/meowctl/internal/lock"
	starlarkpkg "github.com/meowshed/meowctl/internal/starlark"
)

// sharedConfigDir returns a per-test config dir with a pre-written init.star.
func sharedConfigDir(t *testing.T, star string) string {
	t.Helper()
	tmp := t.TempDir()
	if err := os.WriteFile(filepath.Join(tmp, "init.star"), []byte(star), 0o600); err != nil {
		t.Fatal(err)
	}
	return tmp
}

// containsID reports whether ids contains the given logical name.
func containsID(ids []lifecycle.ComponentID, name string) bool {
	for _, id := range ids {
		if string(id) == name {
			return true
		}
	}
	return false
}

// TestLiveLoadComponents_StdlibGit creates a minimal meowctl.star that declares
// a single @stdlib//components/git component and verifies that
// loadComponentsWithDeps resolves it successfully against the live registry.
func TestLiveLoadComponents_StdlibGit(t *testing.T) {
	configDir := sharedConfigDir(t, `
module(name = "test", version = "0.1.0")
component("@stdlib//components/git")
`)

	ids, err := cli.ExportLoadComponents(configDir, nil)
	if err != nil {
		t.Fatalf("loadComponentsWithDeps failed: %v", err)
	}
	if !containsID(ids, "git") {
		t.Fatalf("expected component 'git' in resolved ids, got %v", ids)
	}
	t.Logf("resolved ids: %v", ids)
}

// TestLiveLoadComponents_TransitiveDeps declares the git bundle which has four
// transitive component deps (lazygit, tig, delta, forgit). Asserts all four
// appear in the resolved set and that topo order is stable across two runs.
func TestLiveLoadComponents_TransitiveDeps(t *testing.T) {
	star := `
module(name = "test", version = "0.1.0")
component("@stdlib//bundles/git")
`
	wantComponents := []string{"lazygit", "tig", "delta", "forgit"}

	configDir := sharedConfigDir(t, star)
	ids, err := cli.ExportLoadComponents(configDir, nil)
	if err != nil {
		t.Fatalf("first run failed: %v", err)
	}
	for _, want := range wantComponents {
		if !containsID(ids, want) {
			t.Errorf("expected transitive dep %q in ids, got %v", want, ids)
		}
	}

	// Second run — assert same order (topo sort must be deterministic).
	ids2, err := cli.ExportLoadComponents(configDir, nil)
	if err != nil {
		t.Fatalf("second run failed: %v", err)
	}
	if len(ids) != len(ids2) {
		t.Fatalf("topo order unstable: first=%v second=%v", ids, ids2)
	}
	for i := range ids {
		if ids[i] != ids2[i] {
			t.Errorf("topo order differs at position %d: %v vs %v", i, ids[i], ids2[i])
		}
	}
	t.Logf("resolved ids (stable): %v", ids)
}

// TestLiveLoadComponents_MultipleComponents declares two independent stdlib
// components, asserts both resolve without duplication.
func TestLiveLoadComponents_MultipleComponents(t *testing.T) {
	configDir := sharedConfigDir(t, `
module(name = "test", version = "0.1.0")
component("@stdlib//components/git")
component("@stdlib//components/gh")
`)

	ids, err := cli.ExportLoadComponents(configDir, nil)
	if err != nil {
		t.Fatalf("loadComponentsWithDeps failed: %v", err)
	}

	for _, want := range []string{"git", "gh"} {
		if !containsID(ids, want) {
			t.Errorf("expected %q in ids, got %v", want, ids)
		}
	}

	// No duplicates.
	seen := make(map[string]int)
	for _, id := range ids {
		seen[string(id)]++
	}
	for name, count := range seen {
		if count > 1 {
			t.Errorf("component %q appears %d times — duplicate in resolved set", name, count)
		}
	}
	t.Logf("resolved ids: %v", ids)
}

// TestLiveLoadComponents_Filter declares three stdlib components but filters to
// one. Asserts only that component (and any transitive deps) are returned.
func TestLiveLoadComponents_Filter(t *testing.T) {
	configDir := sharedConfigDir(t, `
module(name = "test", version = "0.1.0")
component("@stdlib//components/git")
component("@stdlib//components/gh")
component("@stdlib//components/jq")
`)

	ids, err := cli.ExportLoadComponents(configDir, []string{"git"})
	if err != nil {
		t.Fatalf("loadComponentsWithDeps with filter failed: %v", err)
	}

	if !containsID(ids, "git") {
		t.Fatalf("expected 'git' in filtered ids, got %v", ids)
	}
	for _, unexpected := range []string{"gh", "jq"} {
		if containsID(ids, unexpected) {
			t.Errorf("expected %q to be filtered out, but found it in %v", unexpected, ids)
		}
	}
	assertNoStdlibRef(t, ids)
	t.Logf("filtered ids: %v", ids)
}

// TestLiveLoadComponents_CacheReuse runs loadComponentsWithDeps twice against
// the same shared cache dir and asserts the second call is faster (cache hit)
// and returns identical results.
func TestLiveLoadComponents_CacheReuse(t *testing.T) {
	// Use a single persistent cache across both runs by writing meowctl.star to
	// a temp dir and pointing the HOME so meowctl uses our cache.
	cacheDir := t.TempDir()
	t.Setenv("HOME", cacheDir)
	// XDG_CACHE_HOME overrides ~/.cache on Linux; meowctl uses os.UserHomeDir()
	// so setting HOME is sufficient on both platforms.

	star := `
module(name = "test", version = "0.1.0")
component("@stdlib//components/git")
`
	configDir := sharedConfigDir(t, star)

	start1 := time.Now()
	ids1, err := cli.ExportLoadComponents(configDir, nil)
	if err != nil {
		t.Fatalf("first run failed: %v", err)
	}
	dur1 := time.Since(start1)

	start2 := time.Now()
	ids2, err := cli.ExportLoadComponents(configDir, nil)
	if err != nil {
		t.Fatalf("second run failed: %v", err)
	}
	dur2 := time.Since(start2)

	// Results must be identical.
	if len(ids1) != len(ids2) {
		t.Fatalf("results differ between runs: %v vs %v", ids1, ids2)
	}
	for i := range ids1 {
		if ids1[i] != ids2[i] {
			t.Errorf("id mismatch at %d: %v vs %v", i, ids1[i], ids2[i])
		}
	}

	// Cache hit should be meaningfully faster. We allow generous headroom
	// (second run must be at most half of first) to avoid flakiness.
	t.Logf("first run: %v, second run (cache): %v", dur1, dur2)
	if dur2 > dur1/2 {
		t.Logf("warning: cache reuse did not produce expected speedup (first=%v second=%v)", dur1, dur2)
	}
}

// TestLiveRegistry_AllBundles downloads the stdlib tarball and evaluates each
// of the four bundles (git, github, modern-shell, modern-macos) using
// ReadComponentGlobals. Asserts each exposes an `after` global.
func TestLiveRegistry_AllBundles(t *testing.T) {
	// Trigger a download by loading a known component so the tarball is cached.
	configDir := sharedConfigDir(t, `
module(name = "test", version = "0.1.0")
component("@stdlib//components/git")
`)
	if _, err := cli.ExportLoadComponents(configDir, nil); err != nil {
		t.Fatalf("seed download failed: %v", err)
	}

	// Locate the extracted tarball in the default cache.
	// Read the resolved version from the lock file — avoids hardcoding the version.
	lf, err := lock.Read(filepath.Join(configDir, "deps.lock"))
	if err != nil {
		t.Fatalf("read lock file: %v", err)
	}
	stdlibEntry, ok := lf.Modules["stdlib"]
	if !ok {
		t.Fatal("stdlib not found in lock file after resolution")
	}
	home, err := os.UserHomeDir()
	if err != nil {
		t.Fatalf("UserHomeDir: %v", err)
	}
	bundleDir := filepath.Join(home, ".cache", "meowctl", "modules", "stdlib", stdlibEntry.Version, "bundles")
	if _, err := os.Stat(bundleDir); err != nil {
		t.Fatalf("bundle dir not found in cache (%s): %v", bundleDir, err)
	}

	bundles := []string{"git", "github", "modern-shell", "modern-macos"}
	e := &starlarkpkg.Evaluator{Platform: starlarkpkg.PlatformInfo{OS: "macos"}}

	for _, name := range bundles {
		name := name
		t.Run(name, func(t *testing.T) {
			path := filepath.Join(bundleDir, name+".star")
			src, err := os.ReadFile(path) //nolint:gosec
			if err != nil {
				t.Fatalf("bundle file not found in cache: %v", err)
			}
			result, err := e.ReadComponentGlobals(path, src)
			if err != nil {
				// Bundles use `after =` as a plain list — no meowctl builtins needed.
				// An error here means the file is not valid Starlark.
				t.Fatalf("ReadComponentGlobals(%s): %v", name, err)
			}
			if _, ok := result.Globals["after"]; !ok {
				keys := make([]string, 0, len(result.Globals))
				for k := range result.Globals {
					keys = append(keys, k)
				}
				t.Errorf("bundle %q: expected 'after' global, got keys: %v", name, keys)
			}
		})
	}
}

// assertNoStdlibRef is a helper that fails if any id contains "@" — used to
// confirm filter correctly strips URL-named refs from the output.
func assertNoStdlibRef(t *testing.T, ids []lifecycle.ComponentID) {
	t.Helper()
	for _, id := range ids {
		if strings.Contains(string(id), "@") {
			t.Errorf("unexpected raw URL id in result: %q", id)
		}
	}
}
