package starlark

import (
	"strings"

	gostarlark "go.starlark.net/starlark"
)

// ReplaceDecl records a replace() declaration.
type ReplaceDecl struct {
	Module string
	Path   string
}

// Accumulator collects declarations made by Starlark configuration files during evaluation.
// It is stored as a thread-local value under the key "acc" before each ExecFileOptions call.
type Accumulator struct {
	Components  []ComponentDecl
	Packages    []PkgDecl
	Deps        []DepDecl
	Module      *ModuleDecl
	SelectCases []SelectCase
	Replaces    []ReplaceDecl
}

// ComponentDecl records a component() declaration.
type ComponentDecl struct {
	Name   string
	After  []string       // logical dep names from after= kwarg; may be nil
	Kwargs map[string]any // remaining kwargs after After is extracted
}

// PkgDecl records a pkg() declaration.
type PkgDecl struct {
	Manager string
	Name    string
	Version string
	Kwargs  map[string]any
}

// DepDecl records a dep() declaration.
type DepDecl struct {
	URL string
}

// ModuleDecl records the module() declaration.
type ModuleDecl struct {
	Name    string
	Version string
}

// SelectCase records one case from a select() call after platform evaluation.
type SelectCase struct {
	Condition string
	Value     gostarlark.Value
}

// LogicalName returns the logical component name: the last non-empty path segment
// of the raw name passed to component().
//
// Examples:
//
//	"@stdlib//components/node" → "node"
//	"github://owner/repo//components/neovim" → "neovim"
//	"shell" → "shell"
func (c ComponentDecl) LogicalName() string {
	return logicalName(c.Name)
}

// LogicalNameOf returns the logical component name for a raw component name string.
// Equivalent to ComponentDecl.LogicalName() but usable without a decl instance.
func LogicalNameOf(name string) string {
	return logicalName(name)
}

// logicalName extracts the last non-empty path segment from a raw component name.
func logicalName(name string) string {
	// Strip any trailing slashes before splitting.
	name = strings.TrimRight(name, "/")
	if idx := strings.LastIndex(name, "/"); idx >= 0 {
		return name[idx+1:]
	}
	return name
}
