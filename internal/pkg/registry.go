// Package pkg implements the PM (package manager) handler registry.
// Components that export pm_name + install_pkg + uninstall_pkg + interrogate
// globals are auto-registered as PM handlers during pass-1 evaluation.
// pkg() declarations accumulated during hook execution are dispatched to the
// registered handler for the named manager.
package pkg

import (
	"fmt"

	gostarlark "go.starlark.net/starlark"
)

// PMHandler holds the Starlark callables exported by a PM component.
type PMHandler struct {
	// ComponentName is the logical name of the component that registered this handler.
	ComponentName string
	// InstallPkg is the install_pkg(ctx, name, version, **kwargs) function.
	InstallPkg gostarlark.Callable
	// UninstallPkg is the uninstall_pkg(ctx, name, version, **kwargs) function.
	UninstallPkg gostarlark.Callable
	// Interrogate is the interrogate(ctx) function.
	Interrogate gostarlark.Callable
	// UpdatePkg is the optional update_pkg(ctx, name, **kwargs) function.
	// If nil, DispatchUpdate falls back to InstallPkg(ctx, name, "latest", **kwargs).
	UpdatePkg gostarlark.Callable
}

// PMRegistry maps manager names to their registered handlers.
type PMRegistry struct {
	handlers map[string]*PMHandler
}

// NewPMRegistry returns an empty, ready-to-use PMRegistry.
func NewPMRegistry() *PMRegistry {
	return &PMRegistry{handlers: make(map[string]*PMHandler)}
}

// Register adds a handler for managerName. Overwrites any existing entry.
func (r *PMRegistry) Register(managerName string, h *PMHandler) {
	r.handlers[managerName] = h
}

// Handler returns the PMHandler for managerName, or nil if not found.
func (r *PMRegistry) Handler(managerName string) *PMHandler {
	if r == nil {
		return nil
	}
	return r.handlers[managerName]
}

// Dispatch calls the install_pkg handler for manager with the given arguments.
// phase selects which handler function to call: "install" → InstallPkg,
// "uninstall" → UninstallPkg. All other phase names are no-ops.
// ctx is the Starlark ctx value passed as the first argument to the handler.
// name and version are the package name and version string.
// kwargs contains additional keyword arguments forwarded from the pkg() declaration.
func (r *PMRegistry) Dispatch(phase string, manager, name, version string, kwargs map[string]any, ctx gostarlark.Value) error {
	h := r.Handler(manager)
	if h == nil {
		return fmt.Errorf("pkg: no handler registered for manager %q", manager)
	}

	var fn gostarlark.Callable
	switch phase {
	case "install":
		fn = h.InstallPkg
	case "uninstall":
		fn = h.UninstallPkg
	default:
		// Phases like verify, update do not trigger pkg dispatch.
		return nil
	}

	// Build positional args: (ctx, name, version)
	posArgs := gostarlark.Tuple{ctx, gostarlark.String(name), gostarlark.String(version)}

	// Convert kwargs map to starlark kwargs slice.
	var starKwargs []gostarlark.Tuple
	for k, v := range kwargs {
		sv, ok := toStarlarkValue(v)
		if !ok {
			return fmt.Errorf("pkg: kwarg %q has unsupported type %T", k, v)
		}
		starKwargs = append(starKwargs, gostarlark.Tuple{gostarlark.String(k), sv})
	}

	thread := &gostarlark.Thread{Name: "pkg/" + manager}
	// Set ctx on the thread so that nested pkg() calls inside install_pkg/uninstall_pkg
	// (e.g. github_release's install_pkg calling pkg(manager="mise", ...)) can
	// dispatch immediately via the hook path rather than failing with "no accumulator".
	thread.SetLocal("ctx", ctx)
	_, err := gostarlark.Call(thread, fn, posArgs, starKwargs)
	if err != nil {
		return fmt.Errorf("pkg %s/%s: %w", manager, name, err)
	}
	return nil
}

// DispatchUpdate calls the update_pkg handler for manager with the given arguments.
// If the handler has no update_pkg function, it falls back to
// install_pkg(ctx, name, "latest", **kwargs).
// ctx is the Starlark ctx value passed as the first argument to the handler.
func (r *PMRegistry) DispatchUpdate(manager, name string, kwargs map[string]any, ctx gostarlark.Value) error {
	h := r.Handler(manager)
	if h == nil {
		return fmt.Errorf("pkg: no handler registered for manager %q", manager)
	}

	// Convert kwargs map to starlark kwargs slice.
	var starKwargs []gostarlark.Tuple
	for k, v := range kwargs {
		sv, ok := toStarlarkValue(v)
		if !ok {
			return fmt.Errorf("pkg: kwarg %q has unsupported type %T", k, v)
		}
		starKwargs = append(starKwargs, gostarlark.Tuple{gostarlark.String(k), sv})
	}

	thread := &gostarlark.Thread{Name: "pkg/" + manager}
	thread.SetLocal("ctx", ctx)
	if h.UpdatePkg != nil {
		// update_pkg(ctx, name, **kwargs) — no version arg.
		posArgs := gostarlark.Tuple{ctx, gostarlark.String(name)}
		if _, err := gostarlark.Call(thread, h.UpdatePkg, posArgs, starKwargs); err != nil {
			return fmt.Errorf("pkg update %s/%s: %w", manager, name, err)
		}
		return nil
	}
	// Fallback: install_pkg(ctx, name, "latest", **kwargs).
	posArgs := gostarlark.Tuple{ctx, gostarlark.String(name), gostarlark.String("latest")}
	if _, err := gostarlark.Call(thread, h.InstallPkg, posArgs, starKwargs); err != nil {
		return fmt.Errorf("pkg update (fallback) %s/%s: %w", manager, name, err)
	}
	return nil
}

// toStarlarkValue converts a Go value stored in PkgDecl.Kwargs to a Starlark value.
// Supported: gostarlark.Value (pass-through), string, bool, int, int64, float64.
func toStarlarkValue(v any) (gostarlark.Value, bool) {
	switch val := v.(type) {
	case gostarlark.Value:
		return val, true
	case string:
		return gostarlark.String(val), true
	case bool:
		return gostarlark.Bool(val), true
	case int:
		return gostarlark.MakeInt(val), true
	case int64:
		return gostarlark.MakeInt64(val), true
	case float64:
		return gostarlark.Float(val), true
	}
	return nil, false
}

// ScanGlobals scans a Starlark StringDict for PM registration globals.
// If pm_name, install_pkg, uninstall_pkg, and interrogate are all present,
// a PMHandler is returned. If pm_name is present but functions are missing,
// warnings are written to warn (may be nil). Returns nil if pm_name is absent.
func ScanGlobals(componentName string, globals gostarlark.StringDict, warn func(string)) *PMHandler {
	pmNameVal, hasPMName := globals["pm_name"]
	if !hasPMName {
		return nil
	}
	pmNameStr, ok := pmNameVal.(gostarlark.String)
	if !ok {
		if warn != nil {
			warn(fmt.Sprintf("component %s: pm_name must be a string, got %s; skipping PM registration", componentName, pmNameVal.Type()))
		}
		return nil
	}

	missing := []string{}
	installPkg, _ := globals["install_pkg"].(gostarlark.Callable)
	if installPkg == nil {
		missing = append(missing, "install_pkg")
	}
	uninstallPkg, _ := globals["uninstall_pkg"].(gostarlark.Callable)
	if uninstallPkg == nil {
		missing = append(missing, "uninstall_pkg")
	}
	interrogate, _ := globals["interrogate"].(gostarlark.Callable)
	if interrogate == nil {
		missing = append(missing, "interrogate")
	}

	if len(missing) > 0 {
		if warn != nil {
			warn(fmt.Sprintf("component %s declares pm_name=%q but is missing required functions: %v; skipping PM registration",
				componentName, string(pmNameStr), missing))
		}
		return nil
	}

	updatePkg, _ := globals["update_pkg"].(gostarlark.Callable) // optional

	return &PMHandler{
		ComponentName: componentName,
		InstallPkg:    installPkg,
		UninstallPkg:  uninstallPkg,
		Interrogate:   interrogate,
		UpdatePkg:     updatePkg,
	}
}
