package cli

import (
	"path/filepath"
	"testing"

	"github.com/meowshed/meowctl/internal/lifecycle"
	"github.com/meowshed/meowctl/internal/state"
)

// markInstalled records every install-set phase for a component, which is the
// condition under which lifecycle.Runner skips it.
func markInstalled(t *testing.T, dir string, ids ...lifecycle.ComponentID) {
	t.Helper()
	sm := state.NewManager(filepath.Join(dir, configStateFile))
	for _, id := range ids {
		for _, phase := range lifecycle.PhaseSetInstall {
			if err := sm.RecordComponent(string(phase), id); err != nil {
				t.Fatalf("record %s/%s: %v", id, phase, err)
			}
		}
	}
}

func ids(in []lifecycle.ComponentID) []string {
	out := make([]string, 0, len(in))
	return append(out, in...)
}

func TestPartitionByCompletion_SkipsCompleted(t *testing.T) {
	dir := t.TempDir()
	plan := []lifecycle.ComponentID{"brew", "fish", "yazi"}
	markInstalled(t, dir, "brew", "fish")

	willRun, skipped := partitionByCompletion(dir, plan, nil, false)

	if got := ids(willRun); len(got) != 1 || got[0] != "yazi" {
		t.Fatalf("willRun = %v, want [yazi]", got)
	}
	if got := ids(skipped); len(got) != 2 {
		t.Fatalf("skipped = %v, want brew and fish", got)
	}
}

func TestPartitionByCompletion_StaleAlwaysRuns(t *testing.T) {
	dir := t.TempDir()
	plan := []lifecycle.ComponentID{"brew", "ripgrep-config"}
	markInstalled(t, dir, "brew", "ripgrep-config")

	// A bumped module clears sentinels, so its components must re-run even
	// though state.toml still records them as complete.
	stale := map[string]bool{"ripgrep-config": true}
	willRun, skipped := partitionByCompletion(dir, plan, stale, false)

	if got := ids(willRun); len(got) != 1 || got[0] != "ripgrep-config" {
		t.Fatalf("willRun = %v, want [ripgrep-config]", got)
	}
	if got := ids(skipped); len(got) != 1 || got[0] != "brew" {
		t.Fatalf("skipped = %v, want [brew]", got)
	}
}

func TestPartitionByCompletion_ForceRunsEverything(t *testing.T) {
	dir := t.TempDir()
	plan := []lifecycle.ComponentID{"brew", "fish"}
	markInstalled(t, dir, "brew", "fish")

	willRun, skipped := partitionByCompletion(dir, plan, nil, true)

	if len(willRun) != 2 {
		t.Fatalf("willRun = %v, want both components", ids(willRun))
	}
	if len(skipped) != 0 {
		t.Fatalf("skipped = %v, want none under --force", ids(skipped))
	}
}

func TestPartitionByCompletion_PartialPhasesRun(t *testing.T) {
	dir := t.TempDir()
	sm := state.NewManager(filepath.Join(dir, configStateFile))
	// Only the first phase recorded — the runner would still execute the rest.
	if err := sm.RecordComponent(string(lifecycle.PhaseInstallCheck), "half"); err != nil {
		t.Fatal(err)
	}

	willRun, skipped := partitionByCompletion(dir, []lifecycle.ComponentID{"half"}, nil, false)

	if len(willRun) != 1 {
		t.Fatalf("willRun = %v, want [half]", ids(willRun))
	}
	if len(skipped) != 0 {
		t.Fatalf("skipped = %v, want none", ids(skipped))
	}
}
