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
// declOrder is the declaration-order index of each component; it is used as a
// stable tie-breaker when multiple components are ready at the same time.
// Pass nil or an empty map to fall back to alphabetical order.
//
// Returns an error if the graph contains a cycle.
func TopoSort(deps map[ComponentID][]ComponentID, declOrder ...map[ComponentID]int) ([]ComponentID, error) {
	var priority map[ComponentID]int
	if len(declOrder) > 0 {
		priority = declOrder[0]
	}

	deps = deduplicateDeps(deps)
	allNodes := collectNodes(deps)
	inDegree := buildInDegree(allNodes, deps)
	queue := initialQueue(allNodes, inDegree, priority)

	result := make([]ComponentID, 0, len(allNodes))
	for len(queue) > 0 {
		node := queue[0]
		queue = queue[1:]
		result = append(result, node)

		// For each node that has node as a direct dependency, decrement its
		// in-degree. Collect newly-ready nodes, sort that batch by priority,
		// then append so the overall traversal order is stable across identical inputs.
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
		sortByPriority(newlyReady, priority)
		queue = append(queue, newlyReady...)
	}

	if len(result) != len(allNodes) {
		return nil, fmt.Errorf("lifecycle: dependency graph contains a cycle")
	}
	return result, nil
}

// sortByPriority sorts nodes by priority map value (ascending), falling back to
// alphabetical order for nodes not in the priority map.
func sortByPriority(nodes []ComponentID, priority map[ComponentID]int) {
	if len(priority) == 0 {
		sort.Strings(nodes)
		return
	}
	sort.SliceStable(nodes, func(i, j int) bool {
		pi, iOk := priority[nodes[i]]
		pj, jOk := priority[nodes[j]]
		if iOk && jOk {
			return pi < pj
		}
		if iOk {
			return true // known < unknown
		}
		if jOk {
			return false
		}
		return nodes[i] < nodes[j] // both unknown: alphabetical
	})
}

// deduplicateDeps removes duplicate entries from each dep list to avoid
// inflated in-degree counts and double-enqueue during Kahn's traversal.
func deduplicateDeps(deps map[ComponentID][]ComponentID) map[ComponentID][]ComponentID {
	out := make(map[ComponentID][]ComponentID, len(deps))
	for id, depList := range deps {
		seen := make(map[ComponentID]struct{}, len(depList))
		deduped := make([]ComponentID, 0, len(depList))
		for _, d := range depList {
			if _, ok := seen[d]; !ok {
				seen[d] = struct{}{}
				deduped = append(deduped, d)
			}
		}
		out[id] = deduped
	}
	return out
}

// collectNodes returns the set of all nodes, including those referenced only
// as dependencies (i.e. not appearing as keys in deps).
func collectNodes(deps map[ComponentID][]ComponentID) map[ComponentID]struct{} {
	nodes := make(map[ComponentID]struct{})
	for id, depList := range deps {
		nodes[id] = struct{}{}
		for _, d := range depList {
			nodes[d] = struct{}{}
		}
	}
	return nodes
}

// buildInDegree computes the initial in-degree for each node.
// deps[id] = [d1, d2] means id depends on d1 and d2, so id's in-degree is
// len(deps[id]).
func buildInDegree(allNodes map[ComponentID]struct{}, deps map[ComponentID][]ComponentID) map[ComponentID]int {
	inDegree := make(map[ComponentID]int, len(allNodes))
	for id := range allNodes {
		inDegree[id] = 0
	}
	for id, depList := range deps {
		inDegree[id] += len(depList)
	}
	return inDegree
}

// initialQueue returns the nodes with zero in-degree to seed Kahn's algorithm,
// sorted by priority (declaration order) to ensure deterministic output.
func initialQueue(allNodes map[ComponentID]struct{}, inDegree map[ComponentID]int, priority map[ComponentID]int) []ComponentID {
	queue := make([]ComponentID, 0, len(allNodes))
	for id := range allNodes {
		if inDegree[id] == 0 {
			queue = append(queue, id)
		}
	}
	sortByPriority(queue, priority)
	return queue
}
