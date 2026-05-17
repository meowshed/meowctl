package starlark

import (
	"testing"

	gostarlark "go.starlark.net/starlark"
)

// newThread creates a test thread with an Accumulator set as a local.
func newThread(acc *Accumulator) *gostarlark.Thread {
	t := &gostarlark.Thread{Name: "test"}
	t.SetLocal("acc", acc)
	return t
}

// --- component() ---

func TestBuiltinComponent_Basic(t *testing.T) {
	acc := &Accumulator{}
	thread := newThread(acc)

	_, err := builtinComponent(
		thread, nil,
		gostarlark.Tuple{gostarlark.String("git")},
		nil,
	)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if len(acc.Components) != 1 {
		t.Fatalf("expected 1 component, got %d", len(acc.Components))
	}
	if acc.Components[0].Name != "git" {
		t.Errorf("expected name=%q, got %q", "git", acc.Components[0].Name)
	}
}

func TestBuiltinComponent_WithAfter(t *testing.T) {
	acc := &Accumulator{}
	thread := newThread(acc)

	list := gostarlark.NewList([]gostarlark.Value{gostarlark.String("mise"), gostarlark.String("shell")})
	kwargs := []gostarlark.Tuple{
		{gostarlark.String("after"), list},
	}
	_, err := builtinComponent(
		thread, nil,
		gostarlark.Tuple{gostarlark.String("node")},
		kwargs,
	)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	got := acc.Components[0].After
	if len(got) != 2 || got[0] != "mise" || got[1] != "shell" {
		t.Errorf("expected after=[mise shell], got %v", got)
	}
}

func TestBuiltinComponent_AfterNotList(t *testing.T) {
	acc := &Accumulator{}
	thread := newThread(acc)

	kwargs := []gostarlark.Tuple{
		{gostarlark.String("after"), gostarlark.String("mise")},
	}
	_, err := builtinComponent(thread, nil, gostarlark.Tuple{gostarlark.String("node")}, kwargs)
	if err == nil {
		t.Fatal("expected error when after is not a list")
	}
}

func TestBuiltinComponent_AfterNonStringElement(t *testing.T) {
	acc := &Accumulator{}
	thread := newThread(acc)

	list := gostarlark.NewList([]gostarlark.Value{gostarlark.MakeInt(42)})
	kwargs := []gostarlark.Tuple{
		{gostarlark.String("after"), list},
	}
	_, err := builtinComponent(thread, nil, gostarlark.Tuple{gostarlark.String("node")}, kwargs)
	if err == nil {
		t.Fatal("expected error when after element is not a string")
	}
}

func TestBuiltinComponent_WithKwargs(t *testing.T) {
	acc := &Accumulator{}
	thread := newThread(acc)

	kwargs := []gostarlark.Tuple{
		{gostarlark.String("provides"), gostarlark.String("pm")},
	}
	_, err := builtinComponent(
		thread, nil,
		gostarlark.Tuple{gostarlark.String("brew")},
		kwargs,
	)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	got := acc.Components[0].Kwargs["provides"]
	if got != gostarlark.String("pm") {
		t.Errorf("expected provides=%q, got %v", "pm", got)
	}
}

func TestBuiltinComponent_MissingName(t *testing.T) {
	acc := &Accumulator{}
	thread := newThread(acc)
	_, err := builtinComponent(thread, nil, gostarlark.Tuple{}, nil)
	if err == nil {
		t.Fatal("expected error for missing name argument")
	}
}

func TestBuiltinComponent_NoAccumulator(t *testing.T) {
	thread := &gostarlark.Thread{Name: "no-acc"}
	_, err := builtinComponent(thread, nil,
		gostarlark.Tuple{gostarlark.String("git")}, nil)
	if err == nil {
		t.Fatal("expected error when accumulator is not set")
	}
}

// --- pkg() ---

func TestBuiltinPkg_Basic(t *testing.T) {
	acc := &Accumulator{}
	thread := newThread(acc)

	_, err := builtinPkg(
		thread, nil,
		gostarlark.Tuple{gostarlark.String("brew"), gostarlark.String("ripgrep")},
		nil,
	)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if len(acc.Packages) != 1 {
		t.Fatalf("expected 1 package, got %d", len(acc.Packages))
	}
	p := acc.Packages[0]
	if p.Manager != "brew" || p.Name != "ripgrep" || p.Version != "" {
		t.Errorf("unexpected pkg: %+v", p)
	}
}

func TestBuiltinPkg_WithVersion(t *testing.T) {
	acc := &Accumulator{}
	thread := newThread(acc)

	kwargs := []gostarlark.Tuple{
		{gostarlark.String("version"), gostarlark.String("14.1.0")},
	}
	_, err := builtinPkg(
		thread, nil,
		gostarlark.Tuple{gostarlark.String("brew"), gostarlark.String("neovim")},
		kwargs,
	)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if acc.Packages[0].Version != "14.1.0" {
		t.Errorf("expected version=14.1.0, got %q", acc.Packages[0].Version)
	}
}

func TestBuiltinPkg_MissingArgs(t *testing.T) {
	acc := &Accumulator{}
	thread := newThread(acc)
	_, err := builtinPkg(thread, nil, gostarlark.Tuple{gostarlark.String("brew")}, nil)
	if err == nil {
		t.Fatal("expected error for missing name argument")
	}
}

// --- unpkg() ---

func TestBuiltinUnpkg_NoCtx(t *testing.T) {
	// unpkg() requires a ctx on the thread; no accumulator path exists.
	thread := &gostarlark.Thread{Name: "no-ctx"}
	_, err := builtinUnpkg(
		thread, nil,
		gostarlark.Tuple{gostarlark.String("brew"), gostarlark.String("ripgrep")},
		nil,
	)
	if err == nil {
		t.Fatal("expected error when ctx is not set on thread")
	}
}

func TestBuiltinUnpkg_MissingArgs(t *testing.T) {
	thread := &gostarlark.Thread{Name: "test"}
	_, err := builtinUnpkg(thread, nil, gostarlark.Tuple{gostarlark.String("brew")}, nil)
	if err == nil {
		t.Fatal("expected error for missing name argument")
	}
}

// --- uppkg() ---

func TestBuiltinUppkg_NoCtx(t *testing.T) {
	// uppkg() requires a ctx on the thread; no accumulator path exists.
	thread := &gostarlark.Thread{Name: "no-ctx"}
	_, err := builtinUppkg(
		thread, nil,
		gostarlark.Tuple{gostarlark.String("brew"), gostarlark.String("ripgrep")},
		nil,
	)
	if err == nil {
		t.Fatal("expected error when ctx is not set on thread")
	}
}

func TestBuiltinUppkg_MissingArgs(t *testing.T) {
	thread := &gostarlark.Thread{Name: "test"}
	_, err := builtinUppkg(thread, nil, gostarlark.Tuple{gostarlark.String("brew")}, nil)
	if err == nil {
		t.Fatal("expected error for missing name argument")
	}
}

// --- dep() ---

func TestBuiltinDep_Basic(t *testing.T) {
	acc := &Accumulator{}
	thread := newThread(acc)

	_, err := builtinDep(
		thread, nil,
		gostarlark.Tuple{gostarlark.String("github://meowshed/stdlib//base")},
		nil,
	)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if len(acc.Deps) != 1 || acc.Deps[0].URL != "github://meowshed/stdlib//base" {
		t.Errorf("unexpected deps: %+v", acc.Deps)
	}
}

func TestBuiltinDep_MissingURL(t *testing.T) {
	acc := &Accumulator{}
	thread := newThread(acc)
	_, err := builtinDep(thread, nil, gostarlark.Tuple{}, nil)
	if err == nil {
		t.Fatal("expected error for missing url argument")
	}
}

// --- module() ---

func TestBuiltinModule_Basic(t *testing.T) {
	acc := &Accumulator{}
	thread := newThread(acc)

	_, err := builtinModule(
		thread, nil,
		gostarlark.Tuple{gostarlark.String("mymod"), gostarlark.String("1.0.0")},
		nil,
	)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if acc.Module == nil {
		t.Fatal("expected module to be set")
	}
	if acc.Module.Name != "mymod" || acc.Module.Version != "1.0.0" {
		t.Errorf("unexpected module: %+v", acc.Module)
	}
}

func TestBuiltinModule_DuplicateDeclaration(t *testing.T) {
	acc := &Accumulator{}
	thread := newThread(acc)

	if _, err := builtinModule(thread, nil,
		gostarlark.Tuple{gostarlark.String("a"), gostarlark.String("1.0.0")}, nil); err != nil {
		t.Fatalf("first module: %v", err)
	}
	_, err := builtinModule(thread, nil,
		gostarlark.Tuple{gostarlark.String("b"), gostarlark.String("2.0.0")}, nil)
	if err == nil {
		t.Fatal("expected error on duplicate module declaration")
	}
}

// --- select() ---

func TestBuiltinSelect_MatchesPlatform(t *testing.T) {
	platform := PlatformInfo{OS: "macos"}
	fn := makeBuiltinSelect(platform)
	acc := &Accumulator{}
	thread := newThread(acc)

	cases, _ := gostarlark.Call(thread, fn, gostarlark.Tuple{
		buildSelectDict(t, []selectEntry{
			{"//platform:macos", gostarlark.String("mac-value")},
			{"//platform:linux", gostarlark.String("linux-value")},
			{"//conditions:default", gostarlark.String("default-value")},
		}),
	}, nil)

	if cases != gostarlark.String("mac-value") {
		t.Errorf("expected mac-value, got %v", cases)
	}
	if len(acc.SelectCases) != 1 || acc.SelectCases[0].Condition != "//platform:macos" {
		t.Errorf("unexpected SelectCases: %+v", acc.SelectCases)
	}
}

func TestBuiltinSelect_FallsBackToDefault(t *testing.T) {
	platform := PlatformInfo{OS: "windows"}
	fn := makeBuiltinSelect(platform)
	acc := &Accumulator{}
	thread := newThread(acc)

	result, _ := gostarlark.Call(thread, fn, gostarlark.Tuple{
		buildSelectDict(t, []selectEntry{
			{"//platform:macos", gostarlark.String("mac-value")},
			{"//conditions:default", gostarlark.String("default-value")},
		}),
	}, nil)

	if result != gostarlark.String("default-value") {
		t.Errorf("expected default-value, got %v", result)
	}
}

func TestBuiltinSelect_NoMatchNoDefault(t *testing.T) {
	platform := PlatformInfo{OS: "linux"}
	fn := makeBuiltinSelect(platform)
	acc := &Accumulator{}
	thread := newThread(acc)

	_, err := gostarlark.Call(thread, fn, gostarlark.Tuple{
		buildSelectDict(t, []selectEntry{
			{"//platform:macos", gostarlark.String("mac-value")},
		}),
	}, nil)

	if err == nil {
		t.Error("expected error when no condition matched and no default provided")
	}
}

func TestBuiltinSelect_MacOSArm64NeverMatches(t *testing.T) {
	// //platform:macos-arm64 must not silently match all macOS hosts until arch detection ships.
	// With no default provided, select() must return an error.
	platform := PlatformInfo{OS: "macos"}
	fn := makeBuiltinSelect(platform)
	thread := newThread(&Accumulator{})

	_, err := gostarlark.Call(thread, fn, gostarlark.Tuple{
		buildSelectDict(t, []selectEntry{
			{"//platform:macos-arm64", gostarlark.String("arm64-value")},
		}),
	}, nil)

	if err == nil {
		t.Error("expected error: //platform:macos-arm64 should not match until arch detection is implemented")
	}
}

// --- platform() ---

func TestBuiltinPlatform_ReturnsStruct(t *testing.T) {
	platform := PlatformInfo{OS: "linux", Distro: "ubuntu", VersionID: "22.04", WSL: true}
	pStruct := makePlatformStruct(platform)
	fn := makeBuiltinPlatform(pStruct)
	thread := &gostarlark.Thread{Name: "test"}

	result, err := gostarlark.Call(thread, fn, nil, nil)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	checkField := func(name, want string) {
		t.Helper()
		val, err := result.(interface {
			Attr(string) (gostarlark.Value, error)
		}).Attr(name)
		if err != nil {
			t.Fatalf("Attr(%q): %v", name, err)
		}
		if got := string(val.(gostarlark.String)); got != want {
			t.Errorf("platform.%s = %q, want %q", name, got, want)
		}
	}
	checkField("os", "linux")
	checkField("distro", "ubuntu")
	checkField("version_id", "22.04")

	wslVal, _ := result.(interface {
		Attr(string) (gostarlark.Value, error)
	}).Attr("wsl")
	if wslVal != gostarlark.True {
		t.Errorf("platform.wsl = %v, want True", wslVal)
	}
}

func TestBuiltinPlatform_RejectsArgs(t *testing.T) {
	fn := makeBuiltinPlatform(makePlatformStruct(PlatformInfo{}))
	thread := &gostarlark.Thread{Name: "test"}
	_, err := gostarlark.Call(thread, fn, gostarlark.Tuple{gostarlark.String("extra")}, nil)
	if err == nil {
		t.Fatal("expected error when platform() called with arguments")
	}
}

// --- matchesPlatform ---

func TestMatchesPlatform(t *testing.T) {
	tests := []struct {
		cond     string
		platform PlatformInfo
		want     bool
	}{
		{"//platform:macos", PlatformInfo{OS: "macos"}, true},
		{"//platform:macos", PlatformInfo{OS: "linux"}, false},
		{"//platform:macos-arm64", PlatformInfo{OS: "macos"}, false}, // arch detection not yet implemented
		{"//platform:linux", PlatformInfo{OS: "linux"}, true},
		{"//platform:linux-debian", PlatformInfo{OS: "linux", Distro: "ubuntu", DistroLike: "debian"}, true},
		{"//platform:linux-debian", PlatformInfo{OS: "linux", Distro: "arch"}, false},
		{"//platform:linux-arch", PlatformInfo{OS: "linux", Distro: "arch"}, true},
		{"//platform:linux-fedora", PlatformInfo{OS: "linux", Distro: "fedora"}, true},
		{"//platform:linux-fedora", PlatformInfo{OS: "linux", Distro: "rhel"}, true},
		{"//platform:wsl", PlatformInfo{OS: "linux", WSL: true}, true},
		{"//platform:wsl", PlatformInfo{OS: "linux", WSL: false}, false},
		{"//platform:windows", PlatformInfo{OS: "windows"}, true},
		{"//platform:unknown", PlatformInfo{OS: "macos"}, false},
	}
	for _, tt := range tests {
		t.Run(tt.cond, func(t *testing.T) {
			got := matchesPlatform(tt.cond, tt.platform)
			if got != tt.want {
				t.Errorf("matchesPlatform(%q, %+v) = %v, want %v", tt.cond, tt.platform, got, tt.want)
			}
		})
	}
}

// --- helpers ---

// selectEntry is a key-value pair for buildSelectDict, preserving order for
// select() first-match semantics.
type selectEntry struct {
	k string
	v gostarlark.Value
}

// buildSelectDict builds a *starlark.Dict from an ordered slice of entries.
// Order matters for select() first-match semantics; use this instead of a Go
// map to avoid non-deterministic iteration.
func buildSelectDict(t *testing.T, entries []selectEntry) *gostarlark.Dict {
	t.Helper()
	d := new(gostarlark.Dict)
	for _, e := range entries {
		if err := d.SetKey(gostarlark.String(e.k), e.v); err != nil {
			t.Fatalf("SetKey: %v", err)
		}
	}
	return d
}
