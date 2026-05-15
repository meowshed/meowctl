// Package ctx implements the ctx object passed to lifecycle hook functions in
// user-authored Starlark component files. CtxValue satisfies starlark.HasAttrs
// and exposes a set of methods and data properties — see AttrNames for the
// authoritative list.
package ctx

import (
	"context"
	"fmt"
	"sort"

	"github.com/meowshed/meowctl/internal/pkg"
	"github.com/meowshed/meowctl/internal/rollback"
	gostarlark "go.starlark.net/starlark"
)

// Capabilities holds Go-side data the ctx methods operate on. The caller
// constructs a Capabilities value appropriate for the current phase and
// component before calling New or NewRestricted.
type Capabilities struct {
	// Home is the value of $HOME.
	Home string
	// DryRun indicates the --dry-run flag is set.
	DryRun bool
	// Verbose enables debug-level output: commands, output, path changes.
	Verbose bool
	// ComponentDir is the absolute path to the component's source directory.
	ComponentDir string
	// StateDir is the per-component persistent state directory.
	StateDir string
	// Shell is the target shell name ("zsh", "fish", "bash", "posix").
	// Empty in all phases except shell.star execution.
	Shell string
	// Platform is a pre-built starlark struct matching the platform() builtin output.
	Platform gostarlark.Value
	// Env is a copy of the process environment, used by ctx.env().
	Env map[string]string
	// Phase is the current lifecycle phase name (e.g. "install").
	Phase string
	// Component is the component ID being executed (e.g. "components/neovim").
	Component string
	// RollbackStack is the write-ahead log for reversible operations.
	// May be nil (dry-run mode or read-only phases like verify).
	RollbackStack *rollback.Stack
	// PMRegistry holds the registered package manager handlers, built during
	// pass-1 component evaluation. May be nil if no PM components were loaded.
	PMRegistry *pkg.PMRegistry
	// RuntimeHook is true when ctx is running inside meowctl hook <phase>
	// rather than during install-time lifecycle execution. When true, ctx.emit
	// writes to stdout so the shell can eval the output.
	RuntimeHook bool
	// EmitFile is the absolute path of the shell init file that ctx.emit
	// appends to during install-time execution. Empty string disables file
	// writing (emit becomes a no-op outside shell/login phases or under
	// dry-run). Ignored when RuntimeHook is true.
	EmitFile string
	// Log is the function used for ctx.log() output. Defaults to fmt.Println.
	Log func(msg string)
	// RunFunc, if non-nil, is called by ctx.run() instead of exec.CommandContext.
	// It receives the resolved command name, argument list, and merged environment
	// (os.Environ() plus any caller-supplied overrides). Returning a non-empty
	// stdout string and nil error is equivalent to a real subprocess exiting 0.
	// Returning a non-nil error maps to exit code 1 with empty stdout; the error
	// message is surfaced as the Starlark run result's stderr field. Partial stdout
	// alongside a non-zero exit is not supported — return an error for failure cases.
	// The context passed is always context.Background(); cancellation is not supported.
	// Used by tests to intercept subprocess calls without spawning real processes.
	RunFunc func(ctx context.Context, cmd string, args []string, env []string) (stdout string, err error)
}

// CtxValue is the ctx object passed to lifecycle hook functions. It implements
// starlark.Value and starlark.HasAttrs. Methods are stored in a map keyed by
// their Starlark name; data properties are stored in a separate props map so
// both appear in AttrNames.
//
//nolint:revive // CtxValue is intentional: ctx.Value would be ambiguous with starlark.Value.
type CtxValue struct {
	caps    *Capabilities
	methods map[string]*gostarlark.Builtin
	props   map[string]gostarlark.Value
}

// Compile-time interface checks.
var (
	_ gostarlark.Value    = (*CtxValue)(nil)
	_ gostarlark.HasAttrs = (*CtxValue)(nil)
)

// New constructs a CtxValue with all methods registered and all data
// properties populated from caps. The returned value is ready to pass to
// Evaluator.CallHook as the ctx argument.
func New(caps *Capabilities) *CtxValue {
	c := &CtxValue{
		caps:    caps,
		methods: make(map[string]*gostarlark.Builtin, 22),
		props:   make(map[string]gostarlark.Value, 6),
	}

	// Register data properties.
	c.props["home"] = gostarlark.String(caps.Home)
	c.props["dry_run"] = gostarlark.Bool(caps.DryRun)
	c.props["component_dir"] = gostarlark.String(caps.ComponentDir)
	c.props["state_dir"] = gostarlark.String(caps.StateDir)
	if caps.Shell != "" {
		c.props["shell"] = gostarlark.String(caps.Shell)
	} else {
		c.props["shell"] = gostarlark.None
	}
	if caps.Platform != nil {
		c.props["platform"] = caps.Platform
	} else {
		c.props["platform"] = gostarlark.None
	}

	// Register methods.
	c.methods["log"] = gostarlark.NewBuiltin("log", c.starLog)
	c.methods["env"] = gostarlark.NewBuiltin("env", c.starEnv)
	c.methods["write_file"] = gostarlark.NewBuiltin("write_file", c.starWriteFile)
	c.methods["append_file"] = gostarlark.NewBuiltin("append_file", c.starAppendFile)
	c.methods["delete_file"] = gostarlark.NewBuiltin("delete_file", c.starDeleteFile)
	c.methods["copy_file"] = gostarlark.NewBuiltin("copy_file", c.starCopyFile)
	c.methods["symlink"] = gostarlark.NewBuiltin("symlink", c.starSymlink)
	c.methods["remove_symlink"] = gostarlark.NewBuiltin("remove_symlink", c.starRemoveSymlink)
	c.methods["link_file"] = gostarlark.NewBuiltin("link_file", c.starLinkFile)
	c.methods["mkdir"] = gostarlark.NewBuiltin("mkdir", c.starMkdir)
	c.methods["read_file"] = gostarlark.NewBuiltin("read_file", c.starReadFile)
	c.methods["file_exists"] = gostarlark.NewBuiltin("file_exists", c.starFileExists)
	c.methods["list_dir"] = gostarlark.NewBuiltin("list_dir", c.starListDir)
	c.methods["run"] = gostarlark.NewBuiltin("run", c.starRun)
	c.methods["git_clone"] = gostarlark.NewBuiltin("git_clone", c.starGitClone)
	c.methods["download"] = gostarlark.NewBuiltin("download", c.starDownload)
	c.methods["defaults_write"] = gostarlark.NewBuiltin("defaults_write", c.starDefaultsWrite)
	c.methods["plist_set"] = gostarlark.NewBuiltin("plist_set", c.starPlistSet)
	c.methods["prompt"] = gostarlark.NewBuiltin("prompt", c.starPrompt)
	c.methods["emit"] = gostarlark.NewBuiltin("emit", c.starEmit)
	c.methods["add_path"] = gostarlark.NewBuiltin("add_path", c.starAddPath)
	c.methods["render"] = gostarlark.NewBuiltin("render", c.starRender)
	c.methods["render_file"] = gostarlark.NewBuiltin("render_file", c.starRenderFile)

	return c
}

// Caps returns the Capabilities backing this ctx value.
// Intended for use by predeclared builtins (e.g. pkg, unpkg, query_pm) that
// need to dispatch to the PMRegistry during hook execution.
func (c *CtxValue) Caps() *Capabilities { return c.caps }

// String implements starlark.Value.
func (c *CtxValue) String() string { return "<ctx>" }

// Type implements starlark.Value.
func (c *CtxValue) Type() string { return "ctx" }

// Freeze implements starlark.Value. CtxValue is intentionally not frozen — it
// carries mutable Go-side state during hook execution.
func (c *CtxValue) Freeze() {}

// Truth implements starlark.Value.
func (c *CtxValue) Truth() gostarlark.Bool { return gostarlark.True }

// Hash implements starlark.Value. ctx objects are not hashable.
func (c *CtxValue) Hash() (uint32, error) {
	return 0, fmt.Errorf("unhashable type: ctx")
}

// Attr implements starlark.HasAttrs. Returns nil, nil for unknown names so
// Starlark reports attribute-not-found rather than a Go error.
func (c *CtxValue) Attr(name string) (gostarlark.Value, error) {
	if v, ok := c.props[name]; ok {
		return v, nil
	}
	if v, ok := c.methods[name]; ok {
		return v, nil
	}
	return nil, nil
}

// AttrNames implements starlark.HasAttrs. Returns a sorted list of all
// property and method names registered on this CtxValue.
func (c *CtxValue) AttrNames() []string {
	names := make([]string, 0, len(c.props)+len(c.methods))
	for k := range c.props {
		names = append(names, k)
	}
	for k := range c.methods {
		names = append(names, k)
	}
	sort.Strings(names)
	return names
}
