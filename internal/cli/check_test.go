package cli_test

import (
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/meowshed/meowctl/internal/cli"
)

func TestCheckCmd_EmptyDir(t *testing.T) {
	dir := t.TempDir()
	root := cli.NewRootCmdForTest()
	root.SetArgs([]string{"check", dir})
	if err := root.Execute(); err != nil {
		t.Fatalf("expected no error on empty dir, got: %v", err)
	}
}

func TestCheckCmd_ValidComponent(t *testing.T) {
	dir := t.TempDir()
	star := filepath.Join(dir, "hello.star")
	if err := os.WriteFile(star, []byte(`
name = "hello"
version = "1.0.0"

def install(ctx):
    pass
`), 0o600); err != nil {
		t.Fatal(err)
	}

	root := cli.NewRootCmdForTest()
	root.SetArgs([]string{"check", dir})
	if err := root.Execute(); err != nil {
		t.Fatalf("expected no error for valid component, got: %v", err)
	}
}

func TestCheckCmd_InvalidStar(t *testing.T) {
	dir := t.TempDir()
	star := filepath.Join(dir, "broken.star")
	if err := os.WriteFile(star, []byte(`this is not valid starlark !!!@@@`), 0o600); err != nil {
		t.Fatal(err)
	}

	root := cli.NewRootCmdForTest()
	var errOut strings.Builder
	root.SetErr(&errOut)
	root.SetArgs([]string{"check", dir})
	err := root.Execute()
	if err == nil {
		t.Fatal("expected error for invalid .star file, got nil")
	}
}

func TestCheckCmd_MissingDir(t *testing.T) {
	root := cli.NewRootCmdForTest()
	root.SetArgs([]string{"check", "/nonexistent/path/to/dir"})
	if err := root.Execute(); err == nil {
		t.Fatal("expected error for missing dir, got nil")
	}
}

func TestCheckCmd_NestedComponents(t *testing.T) {
	dir := t.TempDir()
	comp := filepath.Join(dir, "yazi")
	if err := os.MkdirAll(comp, 0o750); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(filepath.Join(comp, "init.star"), []byte(`
def install(ctx):
    pass
`), 0o600); err != nil {
		t.Fatal(err)
	}

	root := cli.NewRootCmdForTest()
	root.SetArgs([]string{"check", dir})
	if err := root.Execute(); err != nil {
		t.Fatalf("expected no error for nested component, got: %v", err)
	}
}

func TestCheckCmd_NestedInvalidStar(t *testing.T) {
	dir := t.TempDir()
	comp := filepath.Join(dir, "broken")
	if err := os.MkdirAll(comp, 0o750); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(filepath.Join(comp, "init.star"), []byte(`!!! not starlark @@@`), 0o600); err != nil {
		t.Fatal(err)
	}

	root := cli.NewRootCmdForTest()
	var errOut strings.Builder
	root.SetErr(&errOut)
	root.SetArgs([]string{"check", dir})
	if err := root.Execute(); err == nil {
		t.Fatal("expected error for invalid nested .star file, got nil")
	}
}

func TestCheckCmd_SkipsDottedDirs(t *testing.T) {
	dir := t.TempDir()
	hidden := filepath.Join(dir, ".github")
	if err := os.MkdirAll(hidden, 0o750); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(filepath.Join(hidden, "broken.star"), []byte(`!!! not starlark @@@`), 0o600); err != nil {
		t.Fatal(err)
	}

	root := cli.NewRootCmdForTest()
	root.SetArgs([]string{"check", dir})
	if err := root.Execute(); err != nil {
		t.Fatalf("expected dotted dirs to be skipped, got: %v", err)
	}
}
