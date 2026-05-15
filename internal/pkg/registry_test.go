package pkg_test

import (
	"testing"

	"github.com/meowshed/meowctl/internal/pkg"
	gostarlark "go.starlark.net/starlark"
)

// makeCallable returns a simple Starlark builtin that records call count.
func makeCallable(name string, called *int) gostarlark.Callable {
	return gostarlark.NewBuiltin(name, func(_ *gostarlark.Thread, _ *gostarlark.Builtin, _ gostarlark.Tuple, _ []gostarlark.Tuple) (gostarlark.Value, error) {
		*called++
		return gostarlark.None, nil
	})
}

func TestPMRegistry_RegisterAndDispatch(t *testing.T) {
	var installCalled int
	reg := pkg.NewPMRegistry()
	reg.Register("npm", &pkg.PMHandler{
		ComponentName: "node",
		InstallPkg:    makeCallable("install_pkg", &installCalled),
		UninstallPkg:  makeCallable("uninstall_pkg", new(int)),
		Interrogate:   makeCallable("interrogate", new(int)),
	})

	err := reg.Dispatch("install", "npm", "lodash", "4.17.21", nil, gostarlark.None)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if installCalled != 1 {
		t.Errorf("install_pkg called %d times, want 1", installCalled)
	}
}

func TestPMRegistry_Dispatch_UnknownManager(t *testing.T) {
	reg := pkg.NewPMRegistry()
	err := reg.Dispatch("install", "unknown", "pkg", "", nil, gostarlark.None)
	if err == nil {
		t.Fatal("expected error for unknown manager, got nil")
	}
}

func TestPMRegistry_Dispatch_NonInstallPhase(t *testing.T) {
	var installCalled int
	reg := pkg.NewPMRegistry()
	reg.Register("brew", &pkg.PMHandler{
		ComponentName: "homebrew",
		InstallPkg:    makeCallable("install_pkg", &installCalled),
		UninstallPkg:  makeCallable("uninstall_pkg", new(int)),
		Interrogate:   makeCallable("interrogate", new(int)),
	})

	// verify phase should be a no-op.
	err := reg.Dispatch("verify", "brew", "git", "", nil, gostarlark.None)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if installCalled != 0 {
		t.Errorf("install_pkg called %d times for verify phase, want 0", installCalled)
	}
}

func TestPMRegistry_DispatchUpdate_WithUpdatePkg(t *testing.T) {
	var gotName string
	var gotKwargs []gostarlark.Tuple
	reg := pkg.NewPMRegistry()
	reg.Register("brew", &pkg.PMHandler{
		ComponentName: "homebrew",
		InstallPkg:    makeCallable("install_pkg", new(int)),
		UninstallPkg:  makeCallable("uninstall_pkg", new(int)),
		Interrogate:   makeCallable("interrogate", new(int)),
		UpdatePkg: gostarlark.NewBuiltin("update_pkg", func(_ *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (gostarlark.Value, error) {
			if len(args) >= 2 {
				gotName = string(args[1].(gostarlark.String))
			}
			gotKwargs = kwargs
			return gostarlark.None, nil
		}),
	})

	err := reg.DispatchUpdate("brew", "git", map[string]any{"cask": true}, gostarlark.None)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if gotName != "git" {
		t.Errorf("name = %q, want %q", gotName, "git")
	}
	if len(gotKwargs) != 1 {
		t.Errorf("expected 1 kwarg, got %d", len(gotKwargs))
	}
}

func TestPMRegistry_DispatchUpdate_Fallback(t *testing.T) {
	var installCalled int
	var gotVersion string
	reg := pkg.NewPMRegistry()
	reg.Register("brew", &pkg.PMHandler{
		ComponentName: "homebrew",
		InstallPkg: gostarlark.NewBuiltin("install_pkg", func(_ *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, _ []gostarlark.Tuple) (gostarlark.Value, error) {
			installCalled++
			if len(args) >= 3 {
				gotVersion = string(args[2].(gostarlark.String))
			}
			return gostarlark.None, nil
		}),
		UninstallPkg: makeCallable("uninstall_pkg", new(int)),
		Interrogate:  makeCallable("interrogate", new(int)),
		// UpdatePkg intentionally nil — should fall back to install_pkg
	})

	err := reg.DispatchUpdate("brew", "git", nil, gostarlark.None)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if installCalled != 1 {
		t.Errorf("install_pkg called %d times, want 1", installCalled)
	}
	if gotVersion != "latest" {
		t.Errorf("fallback version = %q, want %q", gotVersion, "latest")
	}
}

func TestPMRegistry_DispatchUpdate_UnknownManager(t *testing.T) {
	reg := pkg.NewPMRegistry()
	err := reg.DispatchUpdate("unknown", "pkg", nil, gostarlark.None)
	if err == nil {
		t.Fatal("expected error for unknown manager")
	}
}

func TestScanGlobals_AllPresent(t *testing.T) {
	called := 0
	globals := gostarlark.StringDict{
		"pm_name":       gostarlark.String("npm"),
		"install_pkg":   makeCallable("install_pkg", &called),
		"uninstall_pkg": makeCallable("uninstall_pkg", &called),
		"interrogate":   makeCallable("interrogate", &called),
	}

	h := pkg.ScanGlobals("node", globals, nil)
	if h == nil {
		t.Fatal("expected handler, got nil")
	}
	if h.ComponentName != "node" {
		t.Errorf("ComponentName = %q, want %q", h.ComponentName, "node")
	}
	if h.UpdatePkg != nil {
		t.Error("UpdatePkg should be nil when update_pkg not in globals")
	}
}

func TestScanGlobals_WithUpdatePkg(t *testing.T) {
	called := 0
	globals := gostarlark.StringDict{
		"pm_name":       gostarlark.String("npm"),
		"install_pkg":   makeCallable("install_pkg", &called),
		"uninstall_pkg": makeCallable("uninstall_pkg", &called),
		"interrogate":   makeCallable("interrogate", &called),
		"update_pkg":    makeCallable("update_pkg", &called),
	}

	h := pkg.ScanGlobals("node", globals, nil)
	if h == nil {
		t.Fatal("expected handler, got nil")
	}
	if h.UpdatePkg == nil {
		t.Error("UpdatePkg should be set when update_pkg is in globals")
	}
}

func TestPMRegistry_Dispatch_KwargsForwarded(t *testing.T) {
	var receivedKwargs []gostarlark.Tuple
	reg := pkg.NewPMRegistry()
	reg.Register("brew", &pkg.PMHandler{
		ComponentName: "homebrew",
		InstallPkg: gostarlark.NewBuiltin("install_pkg", func(_ *gostarlark.Thread, _ *gostarlark.Builtin, _ gostarlark.Tuple, kwargs []gostarlark.Tuple) (gostarlark.Value, error) {
			receivedKwargs = kwargs
			return gostarlark.None, nil
		}),
		UninstallPkg: makeCallable("uninstall_pkg", new(int)),
		Interrogate:  makeCallable("interrogate", new(int)),
	})

	err := reg.Dispatch("install", "brew", "git", "2.0", map[string]any{
		"with_docs": true,
		"formula":   "git",
	}, gostarlark.None)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if len(receivedKwargs) != 2 {
		t.Errorf("expected 2 kwargs, got %d", len(receivedKwargs))
	}
	kwmap := make(map[string]gostarlark.Value)
	for _, kv := range receivedKwargs {
		kwmap[string(kv[0].(gostarlark.String))] = kv[1]
	}
	if kwmap["formula"] != gostarlark.String("git") {
		t.Errorf("formula kwarg = %v, want %q", kwmap["formula"], "git")
	}
	if kwmap["with_docs"] != gostarlark.Bool(true) {
		t.Errorf("with_docs kwarg = %v, want true", kwmap["with_docs"])
	}
}

func TestScanGlobals_NoPMName(t *testing.T) {
	globals := gostarlark.StringDict{
		"install_pkg": makeCallable("install_pkg", new(int)),
	}
	h := pkg.ScanGlobals("node", globals, nil)
	if h != nil {
		t.Fatal("expected nil handler when pm_name absent")
	}
}

func TestScanGlobals_MissingFunctions(t *testing.T) {
	var warnMsg string
	globals := gostarlark.StringDict{
		"pm_name": gostarlark.String("npm"),
		// install_pkg and uninstall_pkg and interrogate all missing
	}
	h := pkg.ScanGlobals("node", globals, func(msg string) { warnMsg = msg })
	if h != nil {
		t.Fatal("expected nil handler when functions missing")
	}
	if warnMsg == "" {
		t.Error("expected a warning message, got empty string")
	}
}
