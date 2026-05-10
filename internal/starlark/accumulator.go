package starlark

import gostarlark "go.starlark.net/starlark"

// Accumulator collects declarations made by Starlark configuration files during evaluation.
// It is stored as a thread-local value under the key "acc" before each ExecFileOptions call.
type Accumulator struct {
	Components  []ComponentDecl
	Packages    []PkgDecl
	Deps        []DepDecl
	Module      *ModuleDecl
	SelectCases []SelectCase
}

// ComponentDecl records a component() declaration.
type ComponentDecl struct {
	Name   string
	Kwargs map[string]any
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
