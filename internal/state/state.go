// Package state manages the meowctl sentinel state file at
// ~/.config/meowctl/state.toml. The file tracks the current and most recent
// run, enabling meowctl status queries and interrupted-run detection.
package state

import (
	"fmt"
	"os"
	"path/filepath"
	"sync"
	"time"

	"github.com/BurntSushi/toml"
)

// RolledBack describes the outcome of a rollback attempt.
type RolledBack string

// RolledBack constants describe the outcome of a rollback attempt.
const (
	RolledBackNone    RolledBack = ""
	RolledBackOK      RolledBack = "ok"
	RolledBackPartial RolledBack = "partial"
	RolledBackFailed  RolledBack = "failed"
)

// LastRun records the state of the most recent lifecycle run.
type LastRun struct {
	PhaseSet   string     `toml:"phase_set"`
	StartedAt  time.Time  `toml:"started_at"`
	Completed  bool       `toml:"completed"`
	RolledBack RolledBack `toml:"rolled_back"`
}

// CompletedComponent records one completed component within a phase.
type CompletedComponent struct {
	Phase       string    `toml:"phase"`
	Component   string    `toml:"component"`
	CompletedAt time.Time `toml:"completed_at"`
}

// Sentinel is the top-level structure of state.toml.
type Sentinel struct {
	SchemaVersion       int                  `toml:"schema_version"`
	LastRun             LastRun              `toml:"last_run"`
	CompletedComponents []CompletedComponent `toml:"completed_components"`
}

// Manager handles reading and writing the sentinel file atomically.
// All methods are safe for concurrent use from a single process.
type Manager struct {
	mu   sync.Mutex
	path string
}

// NewManager returns a Manager for the sentinel file at path.
func NewManager(path string) *Manager {
	return &Manager{path: path}
}

// Load reads the sentinel file. Returns a zero-value Sentinel if the file does
// not exist (first run).
func (m *Manager) Load() (Sentinel, error) {
	data, err := os.ReadFile(m.path) // #nosec G304
	if os.IsNotExist(err) {
		return Sentinel{SchemaVersion: 1}, nil
	}
	if err != nil {
		return Sentinel{}, fmt.Errorf("state: read sentinel: %w", err)
	}
	var s Sentinel
	if err := toml.Unmarshal(data, &s); err != nil {
		return Sentinel{}, fmt.Errorf("state: parse sentinel: %w", err)
	}
	return s, nil
}

// Save writes s to the sentinel file atomically via a temp file + rename.
func (m *Manager) Save(s Sentinel) error {
	if err := os.MkdirAll(filepath.Dir(m.path), 0o700); err != nil {
		return fmt.Errorf("state: create dir: %w", err)
	}
	// Use os.CreateTemp in the same directory so that the rename is atomic
	// (same filesystem). A fixed ".tmp" suffix would collide under concurrent
	// writers; a unique name avoids that.
	f, err := os.CreateTemp(filepath.Dir(m.path), "state-*.tmp")
	if err != nil {
		return fmt.Errorf("state: open temp file: %w", err)
	}
	tmp := f.Name()
	if err := toml.NewEncoder(f).Encode(s); err != nil {
		_ = f.Close()
		_ = os.Remove(tmp) // #nosec G703 — tmp is the path returned by CreateTemp in a controlled directory
		return fmt.Errorf("state: encode sentinel: %w", err)
	}
	if err := f.Close(); err != nil {
		_ = os.Remove(tmp) // #nosec G703
		return fmt.Errorf("state: close temp file: %w", err)
	}
	if err := os.Rename(tmp, m.path); err != nil { // #nosec G703 -- m.path is a controlled config directory path
		_ = os.Remove(tmp) // #nosec G703
		return fmt.Errorf("state: rename sentinel: %w", err)
	}
	return nil
}

// RecordRunStart updates LastRun for the beginning of a new run.
func (m *Manager) RecordRunStart(phaseSet string) error {
	m.mu.Lock()
	defer m.mu.Unlock()
	s, err := m.Load()
	if err != nil {
		return err
	}
	s.SchemaVersion = 1
	s.LastRun = LastRun{
		PhaseSet:  phaseSet,
		StartedAt: time.Now().UTC(),
	}
	return m.Save(s)
}

// RecordRunEnd marks the last run as completed (or not) and sets rolled_back.
func (m *Manager) RecordRunEnd(completed bool, rb RolledBack) error {
	m.mu.Lock()
	defer m.mu.Unlock()
	s, err := m.Load()
	if err != nil {
		return err
	}
	s.LastRun.Completed = completed
	s.LastRun.RolledBack = rb
	return m.Save(s)
}

// IsCompleted reports whether the given phase+component pair has been recorded
// as completed. Returns false on any read error (fail-open: re-run the component).
func (m *Manager) IsCompleted(phase, component string) bool {
	m.mu.Lock()
	defer m.mu.Unlock()
	s, err := m.Load()
	if err != nil {
		return false
	}
	for _, cc := range s.CompletedComponents {
		if cc.Phase == phase && cc.Component == component {
			return true
		}
	}
	return false
}

// RecordComponent appends a completed component entry, de-duplicating by
// phase+component (last write wins).
func (m *Manager) RecordComponent(phase, component string) error {
	m.mu.Lock()
	defer m.mu.Unlock()
	s, err := m.Load()
	if err != nil {
		return err
	}
	now := time.Now().UTC()
	updated := false
	for i, cc := range s.CompletedComponents {
		if cc.Phase == phase && cc.Component == component {
			s.CompletedComponents[i].CompletedAt = now
			updated = true
			break
		}
	}
	if !updated {
		s.CompletedComponents = append(s.CompletedComponents, CompletedComponent{
			Phase:       phase,
			Component:   component,
			CompletedAt: now,
		})
	}
	return m.Save(s)
}
