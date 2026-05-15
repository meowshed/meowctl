package starlark

import (
	"fmt"
	"strings"

	gostarlark "go.starlark.net/starlark"
	"go.starlark.net/starlarkstruct"

	starjson "go.starlark.net/lib/json"
)

// PlatformInfo holds the platform data used by platform() and select() builtins.
type PlatformInfo struct {
	OS         string // "macos", "linux", "windows"
	Distro     string // e.g. "ubuntu", "arch", "" on non-linux
	DistroLike string // e.g. "debian", "fedora", "" if unknown
	VersionID  string // e.g. "22.04", "14", ""
	WSL        bool   // true if running inside WSL
}

// makePlatformStruct converts PlatformInfo to a Starlark struct.
func makePlatformStruct(p PlatformInfo) *starlarkstruct.Struct {
	return starlarkstruct.FromStringDict(starlarkstruct.Default, gostarlark.StringDict{
		"os":          gostarlark.String(p.OS),
		"distro":      gostarlark.String(p.Distro),
		"distro_like": gostarlark.String(p.DistroLike),
		"version_id":  gostarlark.String(p.VersionID),
		"wsl":         gostarlark.Bool(p.WSL),
	})
}

// makePredeclared builds the StringDict of predeclared builtins for a Starlark evaluation.
// platform is the platform info to use for platform() and select() evaluation.
func makePredeclared(platform PlatformInfo) gostarlark.StringDict {
	pStruct := makePlatformStruct(platform)
	return gostarlark.StringDict{
		"component": gostarlark.NewBuiltin("component", builtinComponent),
		"pkg":       gostarlark.NewBuiltin("pkg", builtinPkg),
		"dep":       gostarlark.NewBuiltin("dep", builtinDep),
		"module":    gostarlark.NewBuiltin("module", builtinModule),
		"replace":   gostarlark.NewBuiltin("replace", builtinReplace),
		"select":    makeBuiltinSelect(platform),
		"platform":  makeBuiltinPlatform(pStruct),
		"json":      starjson.Module,
	}
}

// builtinComponent implements component(name, after=[], **kwargs).
func builtinComponent(thread *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (gostarlark.Value, error) {
	// Extract name from positional args or kwargs; extract after= as []string; allow arbitrary extra kwargs.
	var nameStr string
	positionalName := false
	if len(args) > 0 {
		s, ok := args[0].(gostarlark.String)
		if !ok {
			return nil, fmt.Errorf("component: name must be a string, got %s", args[0].Type())
		}
		nameStr = string(s)
		positionalName = true
	}
	var after []string
	extra := make(map[string]any, len(kwargs))
	for _, kv := range kwargs {
		key := string(kv[0].(gostarlark.String))
		switch key {
		case "name":
			if positionalName {
				return nil, fmt.Errorf("component: name supplied both positionally and as keyword argument")
			}
			s, ok := kv[1].(gostarlark.String)
			if !ok {
				return nil, fmt.Errorf("component: name must be a string, got %s", kv[1].Type())
			}
			nameStr = string(s)
		case "after":
			list, ok := kv[1].(*gostarlark.List)
			if !ok {
				return nil, fmt.Errorf("component: after must be a list, got %s", kv[1].Type())
			}
			after = make([]string, list.Len())
			for i := range after {
				s, ok := list.Index(i).(gostarlark.String)
				if !ok {
					return nil, fmt.Errorf("component: after[%d] must be a string, got %s", i, list.Index(i).Type())
				}
				after[i] = string(s)
			}
		default:
			extra[key] = kv[1]
		}
	}
	if nameStr == "" {
		return nil, fmt.Errorf("component: missing argument for name")
	}
	acc := accFromThread(thread)
	if acc == nil {
		return nil, fmt.Errorf("component: no accumulator on thread")
	}
	acc.Components = append(acc.Components, ComponentDecl{
		Name:   nameStr,
		After:  after,
		Kwargs: extra,
	})
	return gostarlark.None, nil
}

// builtinPkg implements pkg(manager, name, version="", **kwargs).
func builtinPkg(thread *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (gostarlark.Value, error) {
	// Extract manager and name from positional args or kwargs; allow arbitrary extra kwargs.
	managerStr, nameStr, versionStr, extra, err := parsePkgArgs(args, kwargs)
	if err != nil {
		return nil, err
	}
	acc := accFromThread(thread)
	if acc == nil {
		return nil, fmt.Errorf("pkg: no accumulator on thread")
	}
	acc.Packages = append(acc.Packages, PkgDecl{
		Manager: managerStr,
		Name:    nameStr,
		Version: versionStr,
		Kwargs:  extra,
	})
	return gostarlark.None, nil
}

// parsePkgArgs extracts manager, name, version, and extra kwargs from pkg() arguments.
// manager and name may be positional or keyword; version is keyword-only; all others go into extra.
// Returns an error if a positional argument is also supplied as a keyword argument.
func parsePkgArgs(args gostarlark.Tuple, kwargs []gostarlark.Tuple) (string, string, string, map[string]any, error) {
	positional := []string{"manager", "name"}
	// Track which positional slots were filled to detect double-supply.
	filled := map[string]bool{}
	vals := map[string]*string{}
	var manager, name, version string
	vals["manager"] = &manager
	vals["name"] = &name
	vals["version"] = &version
	extra := make(map[string]any, len(kwargs))

	for i, a := range args {
		if i >= len(positional) {
			break
		}
		s, ok := a.(gostarlark.String)
		if !ok {
			return "", "", "", nil, fmt.Errorf("pkg: %s must be a string, got %s", positional[i], a.Type())
		}
		*vals[positional[i]] = string(s)
		filled[positional[i]] = true
	}
	for _, kv := range kwargs {
		key := string(kv[0].(gostarlark.String))
		if ptr, ok := vals[key]; ok {
			if filled[key] {
				return "", "", "", nil, fmt.Errorf("pkg: %s supplied both positionally and as keyword argument", key)
			}
			s, ok2 := kv[1].(gostarlark.String)
			if !ok2 {
				return "", "", "", nil, fmt.Errorf("pkg: %s must be a string, got %s", key, kv[1].Type())
			}
			*ptr = string(s)
		} else {
			extra[key] = kv[1]
		}
	}
	if manager == "" {
		return "", "", "", nil, fmt.Errorf("pkg: missing argument for manager")
	}
	if name == "" {
		return "", "", "", nil, fmt.Errorf("pkg: missing argument for name")
	}
	return manager, name, version, extra, nil
}

// builtinDep implements dep(url).
func builtinDep(thread *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (gostarlark.Value, error) {
	var url gostarlark.String
	if err := gostarlark.UnpackArgs("dep", args, kwargs, "url", &url); err != nil {
		return nil, err
	}
	acc := accFromThread(thread)
	if acc == nil {
		return nil, fmt.Errorf("dep: no accumulator on thread")
	}
	acc.Deps = append(acc.Deps, DepDecl{URL: string(url)})
	return gostarlark.None, nil
}

// builtinModule implements module(name, version).
func builtinModule(thread *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (gostarlark.Value, error) {
	var name, version gostarlark.String
	if err := gostarlark.UnpackArgs("module", args, kwargs, "name", &name, "version", &version); err != nil {
		return nil, err
	}
	acc := accFromThread(thread)
	if acc == nil {
		return nil, fmt.Errorf("module: no accumulator on thread")
	}
	if acc.Module != nil {
		return nil, fmt.Errorf("module: already declared")
	}
	acc.Module = &ModuleDecl{Name: string(name), Version: string(version)}
	return gostarlark.None, nil
}

// builtinReplace implements replace(module, path).
func builtinReplace(thread *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (gostarlark.Value, error) {
	var module, path gostarlark.String
	if err := gostarlark.UnpackArgs("replace", args, kwargs, "module", &module, "path", &path); err != nil {
		return nil, err
	}
	acc := accFromThread(thread)
	if acc == nil {
		return nil, fmt.Errorf("replace: no accumulator on thread")
	}
	acc.Replaces = append(acc.Replaces, ReplaceDecl{Module: string(module), Path: string(path)})
	return gostarlark.None, nil
}

// makeBuiltinSelect returns a select(cases) builtin bound to the given platform.
// cases is a dict mapping condition strings to values; the first matching condition wins.
// Conditions: //platform:macos, //platform:linux, //platform:linux-debian,
// //platform:linux-arch, //platform:linux-fedora, //platform:wsl,
// //platform:windows, //platform:macos-arm64, //conditions:default.
// Returns an error if no condition matches and no //conditions:default is provided.
func makeBuiltinSelect(platform PlatformInfo) *gostarlark.Builtin {
	// platform is captured by value; the closure is safe across concurrent calls.
	return gostarlark.NewBuiltin("select", func(thread *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (gostarlark.Value, error) {
		var cases *gostarlark.Dict
		if err := gostarlark.UnpackArgs("select", args, kwargs, "cases", &cases); err != nil {
			return nil, err
		}

		var defaultVal gostarlark.Value
		for _, item := range cases.Items() {
			cond, ok := item[0].(gostarlark.String)
			if !ok {
				return nil, fmt.Errorf("select: condition key must be a string, got %s", item[0].Type())
			}
			val := item[1]
			c := string(cond)
			if c == "//conditions:default" {
				defaultVal = val
				continue
			}
			if matchesPlatform(c, platform) {
				acc := accFromThread(thread)
				if acc != nil {
					acc.SelectCases = append(acc.SelectCases, SelectCase{Condition: c, Value: val})
				}
				return val, nil
			}
		}
		if defaultVal != nil {
			acc := accFromThread(thread)
			if acc != nil {
				acc.SelectCases = append(acc.SelectCases, SelectCase{Condition: "//conditions:default", Value: defaultVal})
			}
			return defaultVal, nil
		}
		// No condition matched and no default was provided.
		return nil, fmt.Errorf("select: no condition matched and no //conditions:default provided")
	})
}

// matchesPlatform returns true if the condition string matches the given platform.
func matchesPlatform(cond string, p PlatformInfo) bool {
	switch cond {
	case "//platform:macos":
		return p.OS == "macos"
	case "//platform:macos-arm64":
		// Runtime architecture detection is not yet implemented; return false rather than
		// silently matching all macOS hosts. Full detection ships with the platform package.
		return false
	case "//platform:linux":
		return p.OS == "linux"
	case "//platform:linux-debian":
		return matchesLinuxDistro(p, "debian", "ubuntu", "debian")
	case "//platform:linux-arch":
		return matchesLinuxDistro(p, "arch", "", "arch")
	case "//platform:linux-fedora":
		return matchesLinuxDistro(p, "fedora", "rhel", "fedora")
	case "//platform:wsl":
		return p.WSL
	case "//platform:windows":
		return p.OS == "windows"
	}
	return false
}

// matchesLinuxDistro returns true if the platform is Linux and the distro matches
// by exact name (distro1 or distro2) or by ID_LIKE substring (likeSubstr).
// distro2 and likeSubstr may be empty strings to skip those checks.
func matchesLinuxDistro(p PlatformInfo, distro1, distro2, likeSubstr string) bool {
	if p.OS != "linux" {
		return false
	}
	if p.Distro == distro1 || (distro2 != "" && p.Distro == distro2) {
		return true
	}
	return likeSubstr != "" && strings.Contains(p.DistroLike, likeSubstr)
}

// makeBuiltinPlatform returns a platform() builtin that returns the given struct.
func makeBuiltinPlatform(pStruct *starlarkstruct.Struct) *gostarlark.Builtin {
	return gostarlark.NewBuiltin("platform", func(_ *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (gostarlark.Value, error) {
		if err := gostarlark.UnpackArgs("platform", args, kwargs); err != nil {
			return nil, err
		}
		return pStruct, nil
	})
}

// accFromThread retrieves the Accumulator from thread-local storage.
// Returns nil if no accumulator was set (e.g. during a hook-only call).
// A non-nil value of the wrong type indicates a programming error and panics.
func accFromThread(thread *gostarlark.Thread) *Accumulator {
	v := thread.Local("acc")
	if v == nil {
		return nil
	}
	acc, ok := v.(*Accumulator)
	if !ok {
		panic(fmt.Sprintf("accFromThread: unexpected type %T stored under \"acc\"", v))
	}
	return acc
}
