package mvs_test

import (
	"fmt"
	"testing"

	"github.com/meowshed/meowctl/internal/mvs"
)

// mapReqs is a test implementation of mvs.Reqs backed by a static map.
type mapReqs struct {
	deps map[mvs.Module][]mvs.Module
}

func (r *mapReqs) Required(m mvs.Module) ([]mvs.Module, error) {
	return r.deps[m], nil
}

func (r *mapReqs) Max(v1, v2 string) string {
	return mvs.Max(v1, v2)
}

func TestBuildList_NoDeps(t *testing.T) {
	reqs := &mapReqs{deps: map[mvs.Module][]mvs.Module{}}
	root := mvs.Module{Name: "root", Version: "v1.0.0"}

	list, err := mvs.BuildList(root, reqs)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if len(list) != 1 {
		t.Fatalf("expected 1 module, got %d", len(list))
	}
	if list[0] != root {
		t.Errorf("expected %v, got %v", root, list[0])
	}
}

func TestBuildList_DirectDep(t *testing.T) {
	root := mvs.Module{Name: "root", Version: "v1.0.0"}
	dep := mvs.Module{Name: "dep", Version: "v1.2.0"}
	reqs := &mapReqs{deps: map[mvs.Module][]mvs.Module{
		root: {dep},
		dep:  {},
	}}

	list, err := mvs.BuildList(root, reqs)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	got := toMap(list)
	if got["dep"] != "v1.2.0" {
		t.Errorf("expected dep@v1.2.0, got %q", got["dep"])
	}
}

// TestBuildList_MVS_SelectsMax verifies that when two paths require different versions
// of the same module, BuildList selects the maximum (MVS invariant).
func TestBuildList_MVS_SelectsMax(t *testing.T) {
	// Dependency graph:
	//   root → a@v1.0.0, b@v1.0.0
	//   a    → c@v1.1.0
	//   b    → c@v1.3.0
	// Expected: c selected at v1.3.0.
	root := mvs.Module{Name: "root", Version: "v1.0.0"}
	a := mvs.Module{Name: "a", Version: "v1.0.0"}
	b := mvs.Module{Name: "b", Version: "v1.0.0"}
	c1 := mvs.Module{Name: "c", Version: "v1.1.0"}
	c3 := mvs.Module{Name: "c", Version: "v1.3.0"}

	reqs := &mapReqs{deps: map[mvs.Module][]mvs.Module{
		root: {a, b},
		a:    {c1},
		b:    {c3},
		c1:   {},
		c3:   {},
	}}

	list, err := mvs.BuildList(root, reqs)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	got := toMap(list)
	if got["c"] != "v1.3.0" {
		t.Errorf("MVS should select c@v1.3.0, got %q", got["c"])
	}
}

func TestBuildList_RequiredError(t *testing.T) {
	root := mvs.Module{Name: "root", Version: "v1.0.0"}
	bad := mvs.Module{Name: "bad", Version: "v1.0.0"}
	reqs := &errorReqs{
		mapReqs: mapReqs{deps: map[mvs.Module][]mvs.Module{root: {bad}}},
		failOn:  bad,
	}

	_, err := mvs.BuildList(root, reqs)
	if err == nil {
		t.Fatal("expected error, got nil")
	}
}

func TestBuildList_NoneDepExcluded(t *testing.T) {
	// A dependency declared with version "none" should not be enqueued and its
	// transitive dependencies must not be fetched.
	root := mvs.Module{Name: "root", Version: "v1.0.0"}
	excluded := mvs.Module{Name: "excluded", Version: "none"}
	transitive := mvs.Module{Name: "transitive", Version: "v1.0.0"}
	reqs := &mapReqs{deps: map[mvs.Module][]mvs.Module{
		root:       {excluded},
		excluded:   {transitive}, // must never be reached
		transitive: {},
	}}

	list, err := mvs.BuildList(root, reqs)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	got := toMap(list)
	if _, ok := got["transitive"]; ok {
		t.Error("transitive dep of a 'none' module should not appear in build list")
	}
}

func TestBuildList_InvalidSemver(t *testing.T) {
	root := mvs.Module{Name: "root", Version: "v1.0.0"}
	dep := mvs.Module{Name: "dep", Version: "not-semver"}
	reqs := &mapReqs{deps: map[mvs.Module][]mvs.Module{
		root: {dep},
	}}

	_, err := mvs.BuildList(root, reqs)
	if err == nil {
		t.Fatal("expected error for invalid semver, got nil")
	}
}

func TestMax(t *testing.T) {
	cases := []struct {
		v1, v2, want string
	}{
		{"v1.0.0", "v1.2.0", "v1.2.0"},
		{"v1.2.0", "v1.0.0", "v1.2.0"},
		{"none", "v1.0.0", "v1.0.0"},
		{"v1.0.0", "none", "v1.0.0"},
		{"none", "none", "none"},
		{"v2.0.0", "v2.0.0", "v2.0.0"},
	}
	for _, c := range cases {
		t.Run(fmt.Sprintf("Max(%s,%s)", c.v1, c.v2), func(t *testing.T) {
			got := mvs.Max(c.v1, c.v2)
			if got != c.want {
				t.Errorf("got %q, want %q", got, c.want)
			}
		})
	}
}

// toMap converts a build list to a map of name→version for easy assertion.
func toMap(list []mvs.Module) map[string]string {
	m := make(map[string]string, len(list))
	for _, mod := range list {
		m[mod.Name] = mod.Version
	}
	return m
}

// errorReqs is a mapReqs that fails for a specific module.
type errorReqs struct {
	mapReqs
	failOn mvs.Module
}

func (r *errorReqs) Required(m mvs.Module) ([]mvs.Module, error) {
	if m == r.failOn {
		return nil, fmt.Errorf("simulated fetch error for %s", m.Name)
	}
	return r.mapReqs.Required(m)
}
