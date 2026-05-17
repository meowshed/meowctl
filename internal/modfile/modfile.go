// Package modfile parses and writes deps.mod files.
// deps.mod is a machine-managed Starlark file that declares the module identity,
// its dependencies, and local path replacements. It is separate from init.star
// (user-owned) to allow tooling to rewrite dependency versions without touching
// user configuration.
//
// Syntax (subset of Starlark):
//
//	module(name = "my-dotfiles", version = "0.1.0")
//
//	# Registry dep — version required
//	dep(name = "stdlib", version = "0.1.1")
//
//	# GitHub dep — source required, version absent
//	dep(name = "myplugin", source = "github:owner/repo@v1.2.3")
//
//	# replace with local path
//	replace(name = "stdlib", path = "/local/checkout")
//
//	# replace with remote fork
//	replace(name = "stdlib", source = "github:myfork/stdlib@v1.2.4")
package modfile

import (
	"fmt"
	"os"
	"strings"

	gostarlark "go.starlark.net/starlark"
	"go.starlark.net/syntax"
)

// ModFile holds the parsed contents of a deps.mod file.
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

// DepDecl records a dep() declaration. Exactly one of Version or Source must be set.
//
//   - Registry dep: Version is set (e.g. "0.1.1"), Source is empty.
//   - GitHub dep:   Source is set (e.g. "github:owner/repo@v1.2.3"), Version is empty.
type DepDecl struct {
	// Name is the logical module name used in @name// load URLs (e.g. "stdlib").
	Name string
	// Version is the required registry version. Set only for registry deps.
	Version string
	// Source is the GitHub source reference. Set only for GitHub deps.
	// Format: "github:owner/repo@ref" where ref is a tag, branch, or commit SHA.
	Source string
}

// ReplaceDecl records a replace() declaration. Exactly one of Path or Source must be set.
//
//   - Local override:  Path is set (e.g. "/local/checkout"), Source is empty.
//   - Remote override: Source is set (e.g. "github:myfork/stdlib@v1.2.4"), Path is empty.
type ReplaceDecl struct {
	// Name is the logical module name to replace (matches a dep Name).
	Name string
	// Path is the local filesystem path for a local override.
	Path string
	// Source is the GitHub source reference for a remote override.
	// Format: "github:owner/repo@ref".
	Source string
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
	var name, version, source gostarlark.String
	// version and source are both optional at the Starlark level; we validate the
	// mutual-exclusivity constraint below in Go.
	if err := gostarlark.UnpackArgs("dep", args, kwargs,
		"name", &name,
		"version?", &version,
		"source?", &source,
	); err != nil {
		return nil, err
	}
	v, s := string(version), string(source)
	switch {
	case v == "" && s == "":
		return nil, fmt.Errorf("dep(%q): exactly one of version or source is required", string(name))
	case v != "" && s != "":
		return nil, fmt.Errorf("dep(%q): version and source are mutually exclusive", string(name))
	}
	acc := accFromThread(thread)
	acc.deps = append(acc.deps, DepDecl{Name: string(name), Version: v, Source: s})
	return gostarlark.None, nil
}

func builtinReplace(thread *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (gostarlark.Value, error) {
	var name, path, source gostarlark.String
	if err := gostarlark.UnpackArgs("replace", args, kwargs,
		"name", &name,
		"path?", &path,
		"source?", &source,
	); err != nil {
		return nil, err
	}
	p, s := string(path), string(source)
	switch {
	case p == "" && s == "":
		return nil, fmt.Errorf("replace(%q): exactly one of path or source is required", string(name))
	case p != "" && s != "":
		return nil, fmt.Errorf("replace(%q): path and source are mutually exclusive", string(name))
	}
	acc := accFromThread(thread)
	acc.replace = append(acc.replace, ReplaceDecl{Name: string(name), Path: p, Source: s})
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

// Parse reads and evaluates the deps.mod file at path.
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
		if d.Source != "" {
			fmt.Fprintf(&sb, "dep(name = %q, source = %q)\n", d.Name, d.Source)
		} else {
			fmt.Fprintf(&sb, "dep(name = %q, version = %q)\n", d.Name, d.Version)
		}
	}
	if len(mf.Deps) > 0 {
		sb.WriteString("\n")
	}

	for _, r := range mf.Replace {
		if r.Source != "" {
			fmt.Fprintf(&sb, "replace(name = %q, source = %q)\n", r.Name, r.Source)
		} else {
			fmt.Fprintf(&sb, "replace(name = %q, path = %q)\n", r.Name, r.Path)
		}
	}

	return os.WriteFile(path, []byte(sb.String()), 0o600)
}
