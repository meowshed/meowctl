package ctx_test

import (
	"os"
	"path/filepath"
	"slices"
	"strings"
	"testing"

	"github.com/meowshed/meowctl/internal/ctx"
	gostarlark "go.starlark.net/starlark"
)

// expectedAttrNames is the complete sorted list of all 28 ctx attributes
// (6 properties + 22 methods).
var expectedAttrNames = []string{
	"append_file",
	"component_dir",
	"copy_file",
	"defaults_write",
	"delete_file",
	"download",
	"dry_run",
	"emit",
	"env",
	"file_exists",
	"git_clone",
	"home",
	"link_file",
	"list_dir",
	"log",
	"mkdir",
	"platform",
	"plist_set",
	"prompt",
	"read_file",
	"remove_symlink",
	"render",
	"render_file",
	"run",
	"shell",
	"state_dir",
	"symlink",
	"write_file",
}

func testCaps() *ctx.Capabilities {
	return &ctx.Capabilities{
		Home:         "/home/user",
		DryRun:       false,
		ComponentDir: "/home/user/.config/meowctl/components/git",
		StateDir:     "/home/user/.local/share/meowctl/_local/git",
		Shell:        "",
		Platform:     gostarlark.None,
		Env:          map[string]string{"HOME": "/home/user", "EDITOR": "vim"},
	}
}

// dryRunCaps returns Capabilities with DryRun set. Use for tests that exercise
// network or system-call methods where real execution is undesirable.
func dryRunCaps() *ctx.Capabilities {
	caps := testCaps()
	caps.DryRun = true
	return caps
}

// TestCtxValue_AttrNames asserts the full 28-name surface — the structural
// guard that catches missing method registrations at test time.
func TestCtxValue_AttrNames(t *testing.T) {
	c := ctx.New(testCaps())
	got := c.AttrNames()
	if !slices.Equal(got, expectedAttrNames) {
		t.Errorf("AttrNames() mismatch\ngot:  %v\nwant: %v", got, expectedAttrNames)
	}
}

// TestCtxValue_Attr_Method asserts that Attr returns a non-nil Builtin for
// each registered method name.
func TestCtxValue_Attr_Method(t *testing.T) {
	c := ctx.New(testCaps())
	methods := []string{
		"log", "env", "write_file", "append_file", "delete_file", "copy_file",
		"symlink", "remove_symlink", "mkdir", "read_file", "file_exists",
		"list_dir", "run", "git_clone", "download", "defaults_write",
		"plist_set", "prompt", "emit", "render", "render_file",
	}
	for _, name := range methods {
		v, err := c.Attr(name)
		if err != nil {
			t.Errorf("Attr(%q): unexpected error: %v", name, err)
		}
		if v == nil {
			t.Errorf("Attr(%q): got nil, want non-nil Builtin", name)
		}
		if _, ok := v.(*gostarlark.Builtin); !ok {
			t.Errorf("Attr(%q): got %T, want *starlark.Builtin", name, v)
		}
	}
}

// TestCtxValue_Attr_Props asserts that Attr returns the correct starlark
// Value type for each data property.
func TestCtxValue_Attr_Props(t *testing.T) {
	caps := testCaps()
	caps.Shell = "zsh"
	c := ctx.New(caps)

	cases := []struct {
		name string
		want gostarlark.Value
	}{
		{"home", gostarlark.String("/home/user")},
		{"dry_run", gostarlark.False},
		{"component_dir", gostarlark.String("/home/user/.config/meowctl/components/git")},
		{"state_dir", gostarlark.String("/home/user/.local/share/meowctl/_local/git")},
		{"shell", gostarlark.String("zsh")},
		{"platform", gostarlark.None},
	}
	for _, tc := range cases {
		v, err := c.Attr(tc.name)
		if err != nil {
			t.Errorf("Attr(%q): unexpected error: %v", tc.name, err)
		}
		if v != tc.want {
			t.Errorf("Attr(%q) = %v, want %v", tc.name, v, tc.want)
		}
	}
}

// TestCtxValue_Attr_ShellNone asserts that ctx.shell is None when Shell is empty.
func TestCtxValue_Attr_ShellNone(t *testing.T) {
	caps := testCaps()
	caps.Shell = ""
	c := ctx.New(caps)
	v, err := c.Attr("shell")
	if err != nil {
		t.Fatalf("Attr(shell): %v", err)
	}
	if v != gostarlark.None {
		t.Errorf("Attr(shell) = %v, want None", v)
	}
}

// TestCtxValue_Attr_Unknown asserts that Attr returns (nil, nil) for unknown
// names — Starlark interprets this as attribute-not-found.
func TestCtxValue_Attr_Unknown(t *testing.T) {
	c := ctx.New(testCaps())
	v, err := c.Attr("does_not_exist")
	if err != nil {
		t.Errorf("Attr(unknown): unexpected error: %v", err)
	}
	if v != nil {
		t.Errorf("Attr(unknown) = %v, want nil", v)
	}
}

// TestCtxValue_Hash asserts that ctx is not hashable.
func TestCtxValue_Hash(t *testing.T) {
	c := ctx.New(testCaps())
	_, err := c.Hash()
	if err == nil {
		t.Error("Hash(): expected error for unhashable type, got nil")
	}
}

// ---- RestrictedCtxValue tests ----

// TestRestrictedCtxValue_AttrNames asserts that AttrNames only includes
// names from the allow-list.
func TestRestrictedCtxValue_AttrNames(t *testing.T) {
	r := ctx.NewRestricted(testCaps(), ctx.ShellCtxAllowList)
	got := r.AttrNames()
	want := slices.Clone(ctx.ShellCtxAllowList)
	slices.Sort(want)
	if !slices.Equal(got, want) {
		t.Errorf("AttrNames() mismatch\ngot:  %v\nwant: %v", got, want)
	}
}

// TestRestrictedCtxValue_Attr_Allowed asserts that allowed names return a
// non-nil value.
func TestRestrictedCtxValue_Attr_Allowed(t *testing.T) {
	r := ctx.NewRestricted(testCaps(), ctx.ShellCtxAllowList)
	for _, name := range ctx.ShellCtxAllowList {
		v, err := r.Attr(name)
		if err != nil {
			t.Errorf("Attr(%q): unexpected error: %v", name, err)
		}
		if v == nil {
			t.Errorf("Attr(%q): got nil, want non-nil for allowed name", name)
		}
	}
}

// TestRestrictedCtxValue_Attr_Blocked asserts that names not in the
// allow-list return (nil, nil).
func TestRestrictedCtxValue_Attr_Blocked(t *testing.T) {
	r := ctx.NewRestricted(testCaps(), ctx.ShellCtxAllowList)
	blocked := []string{"write_file", "symlink", "mkdir", "delete_file", "copy_file", "prompt"}
	for _, name := range blocked {
		v, err := r.Attr(name)
		if err != nil {
			t.Errorf("Attr(%q): unexpected error: %v", name, err)
		}
		if v != nil {
			t.Errorf("Attr(%q) = %v, want nil for blocked name", name, v)
		}
	}
}

// ---- Method arg-parsing tests ----
// Each test calls the method via a Starlark thread to exercise UnpackArgs
// contracts. Missing required args or wrong types should produce errors.

func callBuiltin(t *testing.T, c *ctx.CtxValue, name string, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (gostarlark.Value, error) {
	t.Helper()
	v, err := c.Attr(name)
	if err != nil || v == nil {
		t.Fatalf("Attr(%q) returned nil or error: %v", name, err)
	}
	b := v.(*gostarlark.Builtin)
	thread := &gostarlark.Thread{Name: "test"}
	return gostarlark.Call(thread, b, args, kwargs)
}

func TestStarLog_RequiresMsg(t *testing.T) {
	c := ctx.New(testCaps())
	// Missing required arg.
	_, err := callBuiltin(t, c, "log", nil, nil)
	if err == nil {
		t.Error("log(): expected error for missing msg, got nil")
	}
}

func TestStarLog_ValidCall(t *testing.T) {
	c := ctx.New(testCaps())
	_, err := callBuiltin(t, c, "log", gostarlark.Tuple{gostarlark.String("hello")}, nil)
	if err != nil {
		t.Errorf("log(msg): unexpected error: %v", err)
	}
}

func TestStarEnv_ReturnsValue(t *testing.T) {
	c := ctx.New(testCaps())
	v, err := callBuiltin(t, c, "env", gostarlark.Tuple{gostarlark.String("EDITOR")}, nil)
	if err != nil {
		t.Fatalf("env(key): %v", err)
	}
	if got := string(v.(gostarlark.String)); got != "vim" {
		t.Errorf("env(EDITOR) = %q, want %q", got, "vim")
	}
}

func TestStarEnv_MissingKeyReturnsEmpty(t *testing.T) {
	c := ctx.New(testCaps())
	v, err := callBuiltin(t, c, "env", gostarlark.Tuple{gostarlark.String("DOES_NOT_EXIST")}, nil)
	if err != nil {
		t.Fatalf("env(key): %v", err)
	}
	if got := string(v.(gostarlark.String)); got != "" {
		t.Errorf("env(missing) = %q, want empty", got)
	}
}

func TestStarWriteFile_RequiresBothArgs(t *testing.T) {
	c := ctx.New(testCaps())
	// Missing content.
	_, err := callBuiltin(t, c, "write_file", gostarlark.Tuple{gostarlark.String("/tmp/x")}, nil)
	if err == nil {
		t.Error("write_file(dst): expected error for missing content, got nil")
	}
}

func TestStarAppendFile_MarkerOptional(t *testing.T) {
	c := ctx.New(testCaps())
	_, err := callBuiltin(t, c, "append_file",
		gostarlark.Tuple{gostarlark.String("/tmp/x"), gostarlark.String("data")}, nil)
	if err != nil {
		t.Errorf("append_file(dst, content): unexpected error: %v", err)
	}
	// With marker kwarg.
	_, err = callBuiltin(t, c, "append_file",
		gostarlark.Tuple{gostarlark.String("/tmp/x"), gostarlark.String("data")},
		[]gostarlark.Tuple{{gostarlark.String("marker"), gostarlark.String("# meowctl")}})
	if err != nil {
		t.Errorf("append_file(dst, content, marker=): unexpected error: %v", err)
	}
}

func TestStarRun_ArgsOptional(t *testing.T) {
	c := ctx.New(testCaps())
	_, err := callBuiltin(t, c, "run", gostarlark.Tuple{gostarlark.String("ls")}, nil)
	if err != nil {
		t.Errorf("run(cmd): unexpected error: %v", err)
	}
}

func TestStarGitClone_RefOptional(t *testing.T) {
	c := ctx.New(dryRunCaps())
	_, err := callBuiltin(t, c, "git_clone",
		gostarlark.Tuple{gostarlark.String("https://github.com/x/y"), gostarlark.String("/tmp/y")}, nil)
	if err != nil {
		t.Errorf("git_clone(url, dst): unexpected error: %v", err)
	}
}

func TestStarDownload_ChecksumOptional(t *testing.T) {
	c := ctx.New(dryRunCaps())
	_, err := callBuiltin(t, c, "download",
		gostarlark.Tuple{gostarlark.String("https://example.com/file"), gostarlark.String("/tmp/file")}, nil)
	if err != nil {
		t.Errorf("download(url, dst): unexpected error: %v", err)
	}
}

func TestStarDefaultsWrite_RequiresAllArgs(t *testing.T) {
	// Missing-value check uses a real ctx; the call fails on argument
	// validation before any subprocess is spawned.
	c := ctx.New(testCaps())
	_, err := callBuiltin(t, c, "defaults_write",
		gostarlark.Tuple{
			gostarlark.String("com.apple.finder"),
			gostarlark.String("AppleShowAllFiles"),
			gostarlark.String("bool"),
		}, nil)
	if err == nil {
		t.Error("defaults_write(domain, key, type): expected error for missing value")
	}
	// All-args check must use dry-run to avoid exec'ing the macOS-only
	// `defaults` binary, which is absent on Linux CI runners.
	cd := ctx.New(dryRunCaps())
	_, err = callBuiltin(t, cd, "defaults_write",
		gostarlark.Tuple{
			gostarlark.String("com.apple.finder"),
			gostarlark.String("AppleShowAllFiles"),
			gostarlark.String("bool"),
			gostarlark.True,
		}, nil)
	if err != nil {
		t.Errorf("defaults_write(all args): unexpected error: %v", err)
	}
}

func TestStarPlistSet_RequiresAllArgs(t *testing.T) {
	c := ctx.New(dryRunCaps())
	_, err := callBuiltin(t, c, "plist_set",
		gostarlark.Tuple{
			gostarlark.String("/Library/Preferences/foo.plist"),
			gostarlark.String("Key"),
			gostarlark.String("string"),
			gostarlark.String("val"),
		}, nil)
	if err != nil {
		t.Errorf("plist_set(all args): unexpected error: %v", err)
	}
}

func TestStarPrompt_RequiresQuestion(t *testing.T) {
	c := ctx.New(testCaps())
	_, err := callBuiltin(t, c, "prompt", nil, nil)
	if err == nil {
		t.Error("prompt(): expected error for missing question")
	}
	_, err = callBuiltin(t, c, "prompt", gostarlark.Tuple{gostarlark.String("Your name?")}, nil)
	if err != nil {
		t.Errorf("prompt(question): unexpected error: %v", err)
	}
}

func TestStarEmit_RequiresLine(t *testing.T) {
	c := ctx.New(testCaps())
	_, err := callBuiltin(t, c, "emit", nil, nil)
	if err == nil {
		t.Error("emit(): expected error for missing line")
	}
}

// TestStarEmit_NoopWithoutEmitFile verifies that emit is a no-op when
// RuntimeHook is false and EmitFile is empty.
func TestStarEmit_NoopWithoutEmitFile(t *testing.T) {
	caps := testCaps()
	caps.RuntimeHook = false
	caps.EmitFile = ""
	c := ctx.New(caps)
	_, err := callBuiltin(t, c, "emit", gostarlark.Tuple{gostarlark.String("export FOO=bar")}, nil)
	if err != nil {
		t.Fatalf("emit no-op: unexpected error: %v", err)
	}
}

// TestStarEmit_WritesToEmitFile verifies that emit appends to EmitFile during
// install-time (RuntimeHook=false).
func TestStarEmit_WritesToEmitFile(t *testing.T) {
	dir := t.TempDir()
	emitFile := filepath.Join(dir, ".zshrc")

	caps := testCaps()
	caps.RuntimeHook = false
	caps.EmitFile = emitFile
	c := ctx.New(caps)
	_, err := callBuiltin(t, c, "emit", gostarlark.Tuple{gostarlark.String("export FOO=bar")}, nil)
	if err != nil {
		t.Fatalf("emit: %v", err)
	}
	content, readErr := os.ReadFile(emitFile) // #nosec G304
	if readErr != nil {
		t.Fatalf("read emit file: %v", readErr)
	}
	if !strings.Contains(string(content), "export FOO=bar") {
		t.Errorf("emit file content = %q; want to contain %q", string(content), "export FOO=bar")
	}
}

func TestStarRun_MissingCmdErrors(t *testing.T) {
	c := ctx.New(testCaps())
	_, err := callBuiltin(t, c, "run", nil, nil)
	if err == nil {
		t.Error("run(): expected error for missing cmd, got nil")
	}
}

func TestStarDeleteFile_RequiresDst(t *testing.T) {
	c := ctx.New(testCaps())
	_, err := callBuiltin(t, c, "delete_file", nil, nil)
	if err == nil {
		t.Error("delete_file(): expected error for missing dst, got nil")
	}
}

func TestStarCopyFile_RequiresBothArgs(t *testing.T) {
	c := ctx.New(testCaps())
	_, err := callBuiltin(t, c, "copy_file", gostarlark.Tuple{gostarlark.String("/tmp/src")}, nil)
	if err == nil {
		t.Error("copy_file(src): expected error for missing dst, got nil")
	}
}

func TestStarSymlink_RequiresBothArgs(t *testing.T) {
	c := ctx.New(testCaps())
	_, err := callBuiltin(t, c, "symlink", gostarlark.Tuple{gostarlark.String("/tmp/src")}, nil)
	if err == nil {
		t.Error("symlink(src): expected error for missing dst, got nil")
	}
}

func TestStarRemoveSymlink_RequiresDst(t *testing.T) {
	c := ctx.New(testCaps())
	_, err := callBuiltin(t, c, "remove_symlink", nil, nil)
	if err == nil {
		t.Error("remove_symlink(): expected error for missing dst, got nil")
	}
}

func TestStarMkdir_RequiresPath(t *testing.T) {
	c := ctx.New(testCaps())
	_, err := callBuiltin(t, c, "mkdir", nil, nil)
	if err == nil {
		t.Error("mkdir(): expected error for missing path, got nil")
	}
}

func TestStarRender_ValidCall(t *testing.T) {
	c := ctx.New(testCaps())
	_, err := callBuiltin(t, c, "render",
		gostarlark.Tuple{gostarlark.String("hello {{NAME}}"), &gostarlark.Dict{}}, nil)
	if err != nil {
		t.Errorf("render(template_str, vars): unexpected error: %v", err)
	}
}

func TestStarRender_MissingVarsErrors(t *testing.T) {
	c := ctx.New(testCaps())
	_, err := callBuiltin(t, c, "render", gostarlark.Tuple{gostarlark.String("hello")}, nil)
	if err == nil {
		t.Error("render(template_str): expected error for missing vars, got nil")
	}
}

func TestStarRenderFile_ValidCall(t *testing.T) {
	tmp := t.TempDir()
	if err := os.WriteFile(filepath.Join(tmp, "tmpl.txt"), []byte("hello {{NAME}}"), 0o600); err != nil {
		t.Fatal(err)
	}
	caps := testCaps()
	caps.ComponentDir = tmp
	c := ctx.New(caps)
	_, err := callBuiltin(t, c, "render_file",
		gostarlark.Tuple{gostarlark.String("tmpl.txt"), &gostarlark.Dict{}}, nil)
	if err != nil {
		t.Errorf("render_file(src, vars): unexpected error: %v", err)
	}
}

func TestStarFileExists_ReturnsBool(t *testing.T) {
	c := ctx.New(testCaps())
	v, err := callBuiltin(t, c, "file_exists", gostarlark.Tuple{gostarlark.String("/tmp")}, nil)
	if err != nil {
		t.Fatalf("file_exists: %v", err)
	}
	if _, ok := v.(gostarlark.Bool); !ok {
		t.Errorf("file_exists: got %T, want Bool", v)
	}
}

func TestStarListDir_ReturnsList(t *testing.T) {
	c := ctx.New(testCaps())
	v, err := callBuiltin(t, c, "list_dir", gostarlark.Tuple{gostarlark.String("/tmp")}, nil)
	if err != nil {
		t.Fatalf("list_dir: %v", err)
	}
	if _, ok := v.(*gostarlark.List); !ok {
		t.Errorf("list_dir: got %T, want *starlark.List", v)
	}
}

func TestStarReadFile_ReturnsString(t *testing.T) {
	c := ctx.New(testCaps())
	v, err := callBuiltin(t, c, "read_file", gostarlark.Tuple{gostarlark.String("/tmp/x")}, nil)
	if err != nil {
		t.Fatalf("read_file: %v", err)
	}
	if _, ok := v.(gostarlark.String); !ok {
		t.Errorf("read_file: got %T, want String", v)
	}
}

// TestStarLinkFile_RequiresBothArgs verifies that link_file errors when src or
// dst is missing.
func TestStarLinkFile_RequiresBothArgs(t *testing.T) {
	c := ctx.New(testCaps())
	_, err := callBuiltin(t, c, "link_file", gostarlark.Tuple{gostarlark.String("file.zsh")}, nil)
	if err == nil {
		t.Fatal("link_file with only src: expected error, got nil")
	}
}

// TestStarLinkFile_AbsSrcErrors verifies that an absolute src path is rejected.
func TestStarLinkFile_AbsSrcErrors(t *testing.T) {
	c := ctx.New(testCaps())
	_, err := callBuiltin(t, c, "link_file",
		gostarlark.Tuple{gostarlark.String("/abs/src"), gostarlark.String("/tmp/dst")},
		nil,
	)
	if err == nil {
		t.Fatal("link_file with absolute src: expected error, got nil")
	}
}

// TestStarLinkFile_DryRun verifies that link_file is a no-op under dry-run.
func TestStarLinkFile_DryRun(t *testing.T) {
	caps := dryRunCaps()
	caps.ComponentDir = "/some/component"
	c := ctx.New(caps)
	_, err := callBuiltin(t, c, "link_file",
		gostarlark.Tuple{gostarlark.String("file.zsh"), gostarlark.String("/tmp/dst")},
		nil,
	)
	if err != nil {
		t.Fatalf("link_file dry-run: unexpected error: %v", err)
	}
}

// TestStarLinkFile_CreatesSymlink verifies that link_file creates a symlink
// from dst -> resolved src when dst does not exist.
func TestStarLinkFile_CreatesSymlink(t *testing.T) {
	dir := t.TempDir()
	srcFile := filepath.Join(dir, "file.zsh")
	if err := os.WriteFile(srcFile, []byte("# zsh"), 0o600); err != nil {
		t.Fatal(err)
	}
	dst := filepath.Join(dir, "link.zsh")
	if err := os.Symlink(srcFile, dst); err != nil {
		t.Fatal(err)
	}

	caps := testCaps()
	caps.ComponentDir = dir
	c := ctx.New(caps)
	_, err := callBuiltin(t, c, "link_file",
		gostarlark.Tuple{gostarlark.String("file.zsh"), gostarlark.String(dst)},
		nil,
	)
	if err != nil {
		t.Fatalf("link_file no-op: %v", err)
	}
	// dst should still point to srcFile unchanged.
	target, _ := os.Readlink(dst)
	if target != srcFile {
		t.Errorf("symlink target after no-op = %q; want %q", target, srcFile)
	}
}

// TestStarLinkFile_BackupsRegularFile verifies that a regular file at dst is
// backed up when backup=True (the default).
func TestStarLinkFile_BackupsRegularFile(t *testing.T) {
	dir := t.TempDir()
	srcFile := filepath.Join(dir, "file.zsh")
	if err := os.WriteFile(srcFile, []byte("# new"), 0o600); err != nil {
		t.Fatal(err)
	}
	dst := filepath.Join(dir, "link.zsh")
	if err := os.WriteFile(dst, []byte("# old"), 0o600); err != nil {
		t.Fatal(err)
	}

	caps := testCaps()
	caps.ComponentDir = dir
	c := ctx.New(caps)
	_, err := callBuiltin(t, c, "link_file",
		gostarlark.Tuple{gostarlark.String("file.zsh"), gostarlark.String(dst)},
		nil,
	)
	if err != nil {
		t.Fatalf("link_file backup: %v", err)
	}
	target, readErr := os.Readlink(dst)
	if readErr != nil {
		t.Fatalf("readlink after backup: %v", readErr)
	}
	if target != srcFile {
		t.Errorf("symlink target = %q; want %q", target, srcFile)
	}
	// Backup file must exist and contain original content.
	entries, _ := os.ReadDir(dir)
	var backupFound bool
	for _, e := range entries {
		if strings.HasPrefix(e.Name(), "link.zsh.meowctl-bak.") {
			backupFound = true
			content, _ := os.ReadFile(filepath.Join(dir, e.Name())) // #nosec G304
			if string(content) != "# old" {
				t.Errorf("backup content = %q; want \"# old\"", string(content))
			}
		}
	}
	if !backupFound {
		t.Error("no backup file found")
	}
}

// TestStarLinkFile_NoBackupErrors verifies that link_file errors when dst is a
// regular file and backup=False.
func TestStarLinkFile_NoBackupErrors(t *testing.T) {
	dir := t.TempDir()
	srcFile := filepath.Join(dir, "file.zsh")
	if err := os.WriteFile(srcFile, []byte("# new"), 0o600); err != nil {
		t.Fatal(err)
	}
	dst := filepath.Join(dir, "link.zsh")
	if err := os.WriteFile(dst, []byte("# old"), 0o600); err != nil {
		t.Fatal(err)
	}

	caps := testCaps()
	caps.ComponentDir = dir
	c := ctx.New(caps)
	_, err := callBuiltin(t, c, "link_file",
		gostarlark.Tuple{gostarlark.String("file.zsh"), gostarlark.String(dst)},
		[]gostarlark.Tuple{{gostarlark.String("backup"), gostarlark.Bool(false)}},
	)
	if err == nil {
		t.Fatal("link_file backup=False with regular file: expected error, got nil")
	}
}
