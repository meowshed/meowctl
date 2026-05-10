// Package mvs implements Minimal Version Selection (MVS) for the meowctl module system.
// MVS is described in https://research.swtch.com/vgo-mvs. Given a root module and a
// dependency graph, BuildList returns the minimum version of every required module that
// satisfies all constraints simultaneously.
package mvs

import (
	"fmt"

	"golang.org/x/mod/semver"
)

// Module identifies a versioned module.
type Module struct {
	// Name is the canonical module path (e.g. "github.com/meowshed/meowctl-stdlib").
	Name string
	// Version is a semver string (e.g. "v1.2.3") or "none" to mean no requirement.
	Version string
}

// Reqs is the interface that callers must implement to supply the dependency graph
// to BuildList.
type Reqs interface {
	// Required returns the direct dependencies of module m as declared in its
	// module manifest. Returning an empty slice means m has no dependencies.
	Required(m Module) ([]Module, error)

	// Max returns the maximum of versions v1 and v2 according to semver ordering.
	// "none" is treated as less than any valid semver version.
	Max(v1, v2 string) string
}

// BuildList computes the build list for root using Minimal Version Selection. It returns
// the list of modules (including root) that would be compiled given root's transitive
// dependencies. Each module appears at most once, at the maximum version required by any
// path in the dependency graph.
//
// BuildList returns an error if Required returns an error for any reachable module or if
// a dependency declares an invalid semver version.
func BuildList(root Module, reqs Reqs) ([]Module, error) {
	// selected tracks the maximum required version per module name.
	selected := map[string]string{root.Name: root.Version}
	// queue is a BFS frontier of modules whose dependencies have not yet been visited.
	queue := []Module{root}
	// visited prevents re-expanding a module at the same version.
	visited := map[Module]bool{root: true}

	for len(queue) > 0 {
		m := queue[0]
		queue = queue[1:]

		deps, err := reqs.Required(m)
		if err != nil {
			return nil, fmt.Errorf("mvs: fetching requirements for %s@%s: %w", m.Name, m.Version, err)
		}

		for _, dep := range deps {
			if dep.Version != "none" && !semver.IsValid(dep.Version) {
				return nil, fmt.Errorf("mvs: invalid semver %q required by %s@%s", dep.Version, m.Name, m.Version)
			}
			cur, ok := selected[dep.Name]
			if !ok {
				cur = "none"
			}
			next := reqs.Max(cur, dep.Version)
			selected[dep.Name] = next
			// Enqueue at the new maximum version if we haven't visited it yet.
			// When ok is false (first time seeing this module), cur is "none" and
			// next == dep.Version, so next != cur always holds for any valid version.
			// A dep.Version of "none" yields next == cur == "none" and is never
			// enqueued — its transitive dependencies are intentionally not fetched.
			candidate := Module{Name: dep.Name, Version: next}
			if next != cur && !visited[candidate] {
				visited[candidate] = true
				queue = append(queue, candidate)
			}
		}
	}

	// Build the result list: root first, then remaining modules in unspecified order.
	// Callers must not depend on ordering beyond root appearing first.
	list := []Module{{Name: root.Name, Version: selected[root.Name]}}
	for name, version := range selected {
		if name != root.Name {
			list = append(list, Module{Name: name, Version: version})
		}
	}
	return list, nil
}

// Max is the canonical semver max function for use in Reqs implementations.
// "none" sorts below all valid semver versions.
func Max(v1, v2 string) string {
	if v1 == "none" {
		return v2
	}
	if v2 == "none" {
		return v1
	}
	if semver.Compare(v1, v2) >= 0 {
		return v1
	}
	return v2
}
