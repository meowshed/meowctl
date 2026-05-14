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
