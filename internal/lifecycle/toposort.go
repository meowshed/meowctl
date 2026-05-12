// Package lifecycle implements the meowctl phase runner, component execution order,
// and topological sort of the component dependency graph.
package lifecycle

import (
	"fmt"
	"sort"
)

// ComponentID identifies a component by its path within the dotfiles repo
// (e.g. "components/neovim" or "pm/brew").
type ComponentID = string

// TopoSort returns the components in dependency-first order from the provided
// adjacency list. deps maps each component to the list of components it depends
// on. Components with no entry are treated as having no deps.
//
// Returns an error if the graph contains a cycle.
func TopoSort(deps map[ComponentID][]ComponentID) ([]ComponentID, error) {
	// Deduplicate dep lists to avoid inflated in-degree counts and
	// double-enqueue during Kahn's traversal.
	dedupedDeps := make(map[ComponentID][]ComponentID, len(deps))
	for id, depList := range deps {
		seen := make(map[ComponentID]struct{}, len(depList))
		deduped := depList[:0]
		for _, d := range depList {
			if _, ok := seen[d]; !ok {
				seen[d] = struct{}{}
				deduped = append(deduped, d)
			}
		}
		dedupedDeps[id] = deduped
	}
	deps = dedupedDeps

	// Collect all nodes, including those referenced only as deps.
	allNodes := make(map[ComponentID]struct{})
	for id, depList := range deps {
		allNodes[id] = struct{}{}
		for _, d := range depList {
			allNodes[d] = struct{}{}
		}
	}

	// Compute in-degree for each node.
	// deps[id] = [d1, d2] means id depends on d1, d2 — so id cannot run until
	// d1 and d2 have run. In-degree of id = len(deps[id]).
	inDegree := make(map[ComponentID]int, len(allNodes))
	for id := range allNodes {
		inDegree[id] = 0
	}
	for id, depList := range deps {
		inDegree[id] += len(depList)
	}

	// Kahn's algorithm: seed queue with zero in-degree nodes.
	// Use a deterministic order so output is stable across runs.
	queue := make([]ComponentID, 0, len(allNodes))
	for id := range allNodes {
		if inDegree[id] == 0 {
			queue = append(queue, id)
		}
	}
	sort.Strings(queue)

	result := make([]ComponentID, 0, len(allNodes))
	for len(queue) > 0 {
		// Dequeue first element (deterministic).
		node := queue[0]
		queue = queue[1:]
		result = append(result, node)

		// For each node that has node as a direct dependency, decrement its
		// in-degree. Collect newly-ready nodes, sort that batch, then append
		// so the overall traversal order is stable across identical inputs.
		// O(V*E) overall: for each of V nodes we scan all edges E.
		var newlyReady []ComponentID
		for id, depList := range deps {
			for _, d := range depList {
				if d == node {
					inDegree[id]--
					if inDegree[id] == 0 {
						newlyReady = append(newlyReady, id)
					}
				}
			}
		}
		sort.Strings(newlyReady)
		queue = append(queue, newlyReady...)
	}

	if len(result) != len(allNodes) {
		return nil, fmt.Errorf("lifecycle: dependency graph contains a cycle")
	}
	return result, nil
}
