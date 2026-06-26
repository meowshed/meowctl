package state_test

import (
	"path/filepath"
	"testing"

	"github.com/meowshed/meowctl/internal/state"
)

func TestSentinel_RoundTrip(t *testing.T) {
	dir := t.TempDir()
	m := state.NewManager(filepath.Join(dir, "state.toml"))

	if err := m.RecordRunStart("install"); err != nil {
		t.Fatalf("RecordRunStart: %v", err)
	}
	if err := m.RecordComponent("install", "pm/brew"); err != nil {
		t.Fatalf("RecordComponent: %v", err)
	}
	if err := m.RecordRunEnd(true, state.RolledBackNone); err != nil {
		t.Fatalf("RecordRunEnd: %v", err)
	}

	s, err := m.Load()
	if err != nil {
		t.Fatalf("Load: %v", err)
	}
	if s.LastRun.PhaseSet != "install" {
		t.Errorf("PhaseSet: got %q, want %q", s.LastRun.PhaseSet, "install")
	}
	if !s.LastRun.Completed {
		t.Error("expected Completed = true")
	}
	if len(s.CompletedComponents) != 1 {
		t.Fatalf("expected 1 completed component, got %d", len(s.CompletedComponents))
	}
	if s.CompletedComponents[0].Component != "pm/brew" {
		t.Errorf("component: got %q, want %q", s.CompletedComponents[0].Component, "pm/brew")
	}
}

func TestSentinel_DeduplicateComponents(t *testing.T) {
	dir := t.TempDir()
	m := state.NewManager(filepath.Join(dir, "state.toml"))

	_ = m.RecordComponent("install", "pm/brew")
	_ = m.RecordComponent("install", "pm/brew") // duplicate

	s, _ := m.Load()
	if len(s.CompletedComponents) != 1 {
		t.Errorf("expected 1 after dedup, got %d", len(s.CompletedComponents))
	}
}

func TestSentinel_ClearComponent(t *testing.T) {
	dir := t.TempDir()
	m := state.NewManager(filepath.Join(dir, "state.toml"))

	_ = m.RecordComponent("install_check", "tmux-config")
	_ = m.RecordComponent("install", "tmux-config")
	_ = m.RecordComponent("install", "git-config")

	if err := m.ClearComponent("tmux-config"); err != nil {
		t.Fatalf("ClearComponent: %v", err)
	}

	if m.IsCompleted("install", "tmux-config") || m.IsCompleted("install_check", "tmux-config") {
		t.Error("tmux-config records should have been cleared from all phases")
	}
	if !m.IsCompleted("install", "git-config") {
		t.Error("git-config should remain completed")
	}

	// Clearing a component with no records is a no-op (no error).
	if err := m.ClearComponent("absent"); err != nil {
		t.Fatalf("ClearComponent(absent): %v", err)
	}
}

func TestSentinel_MissingFile(t *testing.T) {
	dir := t.TempDir()
	m := state.NewManager(filepath.Join(dir, "no-such.toml"))
	s, err := m.Load()
	if err != nil {
		t.Fatalf("Load of missing file: %v", err)
	}
	if s.SchemaVersion != 1 {
		t.Errorf("expected schema_version=1 for missing file, got %d", s.SchemaVersion)
	}
}
