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
