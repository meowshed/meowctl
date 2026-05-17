// Package modfile parses and writes deps.mod files.
// deps.mod is a machine-managed Starlark file that declares the module identity,
// its dependencies, and local path replacements. It is separate from init.star
// (user-owned) to allow tooling to rewrite dependency versions without touching
// user configuration.
//
// Syntax (subset of Starlark):
//
//	module(name = "my-dotfiles", version = "0.1.0")
//	dep(name = "stdlib", version = "0.1.1")
//	replace(module = "github://owner/repo", path = "/local/path")
package modfile

import (
	"fmt"
	"os"
	"strings"

	gostarlark "go.starlark.net/starlark"
	"go.starlark.net/syntax"
)

// ModFile holds the parsed contents of a meowctl.mod file.
type ModFile struct {
	// Module is the module identity declaration (name + version).
	Module *ModuleDecl
	// Deps is the list of dep() declarations in declaration order.
	Deps []DepDecl
	// Replace is the list of replace() declarations in declaration order.
	Replace []ReplaceDecl
}

// ModuleDecl records the module(name, version) declaration.
type ModuleDecl struct {
	Name    string
	Version string
}

// DepDecl records a dep(name, version) declaration.
type DepDecl struct {
	Name    string // registry module name (e.g. "stdlib")
	Version string // required version, v-prefixed semver (e.g. "v0.1.1")
}

// ReplaceDecl records a replace(module, path) declaration.
type ReplaceDecl struct {
	Module string
	Path   string
}

// accumulator collects declarations during restricted Starlark execution.
type accumulator struct {
	module  *ModuleDecl
	deps    []DepDecl
	replace []ReplaceDecl
}

func builtinModule(thread *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (gostarlark.Value, error) {
	var name, version gostarlark.String
	if err := gostarlark.UnpackArgs("module", args, kwargs, "name", &name, "version", &version); err != nil {
		return nil, err
	}
	acc := accFromThread(thread)
	if acc.module != nil {
		return nil, fmt.Errorf("module: already declared")
	}
	acc.module = &ModuleDecl{Name: string(name), Version: string(version)}
	return gostarlark.None, nil
}

func builtinDep(thread *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (gostarlark.Value, error) {
	var name, version gostarlark.String
	if err := gostarlark.UnpackArgs("dep", args, kwargs, "name", &name, "version", &version); err != nil {
		return nil, err
	}
	acc := accFromThread(thread)
	acc.deps = append(acc.deps, DepDecl{Name: string(name), Version: string(version)})
	return gostarlark.None, nil
}

func builtinReplace(thread *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (gostarlark.Value, error) {
	var module, path gostarlark.String
	if err := gostarlark.UnpackArgs("replace", args, kwargs, "module", &module, "path", &path); err != nil {
		return nil, err
	}
	acc := accFromThread(thread)
	acc.replace = append(acc.replace, ReplaceDecl{Module: string(module), Path: string(path)})
	return gostarlark.None, nil
}

func accFromThread(t *gostarlark.Thread) *accumulator {
	v := t.Local("acc")
	if v == nil {
		return &accumulator{}
	}
	a, ok := v.(*accumulator)
	if !ok {
		panic(fmt.Sprintf("accFromThread: unexpected type %T stored under \"acc\"", v))
	}
	return a
}

// Parse reads and evaluates the meowctl.mod file at path.
// Returns an error if the file does not exist or contains invalid syntax.
func Parse(path string) (*ModFile, error) {
	src, err := os.ReadFile(path) // #nosec G304 -- caller-controlled path; modfile is trusted config
	if err != nil {
		return nil, fmt.Errorf("modfile: read %s: %w", path, err)
	}
	return ParseBytes(path, src)
}

// ParseBytes evaluates modfile content from src. filename is used in error messages.
func ParseBytes(filename string, src []byte) (*ModFile, error) {
	predeclared := gostarlark.StringDict{
		"module":  gostarlark.NewBuiltin("module", builtinModule),
		"dep":     gostarlark.NewBuiltin("dep", builtinDep),
		"replace": gostarlark.NewBuiltin("replace", builtinReplace),
	}

	acc := &accumulator{}
	thread := &gostarlark.Thread{Name: filename}
	thread.SetLocal("acc", acc)

	opts := &syntax.FileOptions{}
	if _, err := gostarlark.ExecFileOptions(opts, thread, filename, src, predeclared); err != nil {
		return nil, fmt.Errorf("modfile: parse %s: %w", filename, err)
	}

	return &ModFile{
		Module:  acc.module,
		Deps:    acc.deps,
		Replace: acc.replace,
	}, nil
}

// Write serialises mf to path, creating or truncating the file.
func Write(path string, mf *ModFile) error {
	var sb strings.Builder
	sb.WriteString("# meowctl.mod — machine-managed module file. Do not edit by hand.\n\n")

	if mf.Module != nil {
		fmt.Fprintf(&sb, "module(\n    name = %q,\n    version = %q,\n)\n\n", mf.Module.Name, mf.Module.Version)
	}

	for _, d := range mf.Deps {
		fmt.Fprintf(&sb, "dep(name = %q, version = %q)\n", d.Name, d.Version)
	}
	if len(mf.Deps) > 0 {
		sb.WriteString("\n")
	}

	for _, r := range mf.Replace {
		fmt.Fprintf(&sb, "replace(module = %q, path = %q)\n", r.Module, r.Path)
	}

	return os.WriteFile(path, []byte(sb.String()), 0o600)
}
