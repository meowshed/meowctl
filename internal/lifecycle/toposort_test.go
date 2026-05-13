package lifecycle

import (
	"testing"
)

func TestTopoSort_simple(t *testing.T) {
	// A depends on B; B depends on C → order must be C, B, A.
	deps := map[ComponentID][]ComponentID{
		"A": {"B"},
		"B": {"C"},
	}
	got, err := TopoSort(deps)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if len(got) != 3 {
		t.Fatalf("expected 3 nodes, got %d: %v", len(got), got)
	}
	// C must appear before B, B before A.
	pos := make(map[string]int, len(got))
	for i, id := range got {
		pos[id] = i
	}
	if pos["C"] >= pos["B"] {
		t.Errorf("C must come before B; got %v", got)
	}
	if pos["B"] >= pos["A"] {
		t.Errorf("B must come before A; got %v", got)
	}
}

func TestTopoSort_noDeps(t *testing.T) {
	deps := map[ComponentID][]ComponentID{
		"A": {},
		"B": {},
	}
	got, err := TopoSort(deps)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if len(got) != 2 {
		t.Fatalf("expected 2 nodes, got %d", len(got))
	}
}

func TestTopoSort_cycle(t *testing.T) {
	deps := map[ComponentID][]ComponentID{
		"A": {"B"},
		"B": {"A"},
	}
	_, err := TopoSort(deps)
	if err == nil {
		t.Fatal("expected cycle error, got nil")
	}
}

func TestTopoSort_empty(t *testing.T) {
	got, err := TopoSort(nil)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if len(got) != 0 {
		t.Fatalf("expected empty result, got %v", got)
	}
}

func TestTopoSort_declarationOrderTieBreaker(t *testing.T) {
	// shell declared first, git second — no deps — should come out in declaration order.
	deps := map[ComponentID][]ComponentID{
		"shell": {},
		"git":   {},
	}
	priority := map[ComponentID]int{
		"shell": 0,
		"git":   1,
	}
	got, err := TopoSort(deps, priority)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if len(got) != 2 || got[0] != "shell" || got[1] != "git" {
		t.Errorf("expected [shell git], got %v", got)
	}
}

func TestTopoSort_declarationOrderWithDeps(t *testing.T) {
	// git depends on shell; neovim and tmux have no deps (decl order: shell, git, neovim, tmux).
	// Expected: shell, then git (constrained), then neovim before tmux (decl order).
	deps := map[ComponentID][]ComponentID{
		"shell":  {},
		"git":    {"shell"},
		"neovim": {},
		"tmux":   {},
	}
	priority := map[ComponentID]int{
		"shell":  0,
		"git":    1,
		"neovim": 2,
		"tmux":   3,
	}
	got, err := TopoSort(deps, priority)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	pos := make(map[string]int, len(got))
	for i, id := range got {
		pos[id] = i
	}
	if pos["shell"] >= pos["git"] {
		t.Errorf("shell must come before git; got %v", got)
	}
	if pos["neovim"] >= pos["tmux"] {
		t.Errorf("neovim must come before tmux (decl order); got %v", got)
	}
}
