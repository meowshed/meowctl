package ctx

import (
	"bufio"
	"context"
	"crypto/rand"
	"fmt"
	"io"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"sync/atomic"
	"time"

	gostarlark "go.starlark.net/starlark"
	"go.starlark.net/starlarkstruct"
)

// logMsg writes msg using caps.Log or falls back to fmt.Println.
func (c *CtxValue) logMsg(msg string) {
	if c.caps.Log != nil {
		c.caps.Log(msg)
		return
	}
	fmt.Println(msg)
}

// requirePath returns an error if path is empty, preventing file operations on
// an unintended path when the caller omits the argument (e.g. filepath.Dir("")
// resolves to "." which could silently act on the current working directory).
func requirePath(op, path string) error {
	if path == "" {
		return fmt.Errorf("%s: path must not be empty", op)
	}
	return nil
}

// expandPath expands a leading "~" to the user's home directory.
// Paths like "~/foo" become "/Users/user/foo". Paths without a leading "~"
// are returned unchanged. Uses os.UserHomeDir for portability.
func expandPath(path string) (string, error) {
	if !strings.HasPrefix(path, "~") {
		return path, nil
	}
	home, err := os.UserHomeDir()
	if err != nil {
		return "", fmt.Errorf("expand path: %w", err)
	}
	// "~" alone or "~/" prefix.
	if path == "~" {
		return home, nil
	}
	if len(path) > 1 && path[1] == '/' {
		return filepath.Join(home, path[2:]), nil
	}
	// "~username" form — not supported; return as-is.
	return path, nil
}

// resolvePath validates and expands a path. Returns an error if empty;
// expands leading "~" to home dir.
func resolvePath(op, path string) (string, error) {
	if err := requirePath(op, path); err != nil {
		return "", err
	}
	return expandPath(path)
}

// dryLog emits a dry-run line for the given op and args.
func (c *CtxValue) dryLog(op string, args ...string) {
	c.logMsg(fmt.Sprintf("[dry-run] %-16s component=%-20s %s", op, c.caps.Component, strings.Join(args, " ")))
}

// verboseLog emits a verbose line for the given op and args when --verbose is set.
func (c *CtxValue) verboseLog(op string, args ...string) {
	if c.caps.Verbose {
		c.logMsg(fmt.Sprintf("[verbose] %-16s component=%-20s %s", op, c.caps.Component, strings.Join(args, " ")))
	}
}

// starLog implements ctx.log(msg).
// Always executes, even in dry-run mode.
func (c *CtxValue) starLog(_ *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (gostarlark.Value, error) {
	var msg gostarlark.String
	if err := gostarlark.UnpackArgs("log", args, kwargs, "msg", &msg); err != nil {
		return nil, err
	}
	c.logMsg(string(msg))
	return gostarlark.None, nil
}

// starEnv implements ctx.env(key).
// Returns the value of the named environment variable, or empty string if unset.
func (c *CtxValue) starEnv(_ *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (gostarlark.Value, error) {
	var key gostarlark.String
	if err := gostarlark.UnpackArgs("env", args, kwargs, "key", &key); err != nil {
		return nil, err
	}
	if v, ok := c.caps.Env[string(key)]; ok {
		return gostarlark.String(v), nil
	}
	return gostarlark.String(""), nil
}

// starWriteFile implements ctx.write_file(dst, content).
// Backs up prior content for rollback; creates parent dirs as needed.
func (c *CtxValue) starWriteFile(_ *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (gostarlark.Value, error) {
	var dst, content gostarlark.String
	if err := gostarlark.UnpackArgs("write_file", args, kwargs, "dst", &dst, "content", &content); err != nil {
		return nil, err
	}
	path := string(dst)
	if err := requirePath("write_file", path); err != nil {
		return nil, err
	}
	path, err := expandPath(path)
	if err != nil {
		return nil, fmt.Errorf("write_file: %w", err)
	}
	if c.caps.DryRun {
		c.dryLog("write_file", "path="+path)
		return gostarlark.None, nil
	}
	// Read prior content for rollback.
	prior, readErr := os.ReadFile(path) // #nosec G304
	if readErr != nil && !os.IsNotExist(readErr) {
		return nil, fmt.Errorf("write_file: read prior content: %w", readErr)
	}
	hadPrior := readErr == nil
	var priorStr string
	if hadPrior {
		priorStr = string(prior)
	}
	// Record rollback op before executing.
	if c.caps.RollbackStack != nil {
		if err := c.caps.RollbackStack.AppendWriteFile(c.caps.Phase, c.caps.Component, path, priorStr, hadPrior); err != nil {
			return nil, fmt.Errorf("write_file: record rollback: %w", err)
		}
	}
	// Note: parent directories created by MkdirAll below are NOT individually
	// rollback-recorded. Rollback removes the file but leaves any new parent dirs.
	if err := os.MkdirAll(filepath.Dir(path), 0o750); err != nil {
		return nil, fmt.Errorf("write_file: mkdir: %w", err)
	}
	if err := os.WriteFile(path, []byte(string(content)), 0o644); err != nil { // #nosec G306 — 0o644: user config files are intentionally world-readable
		return nil, fmt.Errorf("write_file: %w", err)
	}
	return gostarlark.None, nil
}

// starAppendFile implements ctx.append_file(dst, content, marker=None).
// Wraps the appended content in BEGIN/END meowctl markers for rollback.
func (c *CtxValue) starAppendFile(_ *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (_ gostarlark.Value, retErr error) {
	var dst, content gostarlark.String
	var markerVal gostarlark.Value = gostarlark.None
	if err := gostarlark.UnpackArgs(
		"append_file", args, kwargs,
		"dst", &dst,
		"content", &content,
		"marker?", &markerVal,
	); err != nil {
		return nil, err
	}
	path := string(dst)
	if err := requirePath("append_file", path); err != nil {
		return nil, err
	}
	path, err := expandPath(path)
	if err != nil {
		return nil, fmt.Errorf("append_file: %w", err)
	}
	if c.caps.DryRun {
		c.dryLog("append_file", "path="+path)
		return gostarlark.None, nil
	}
	// Determine the marker UUID.
	marker := newUUID()
	if ms, ok := markerVal.(gostarlark.String); ok && string(ms) != "" {
		marker = string(ms)
	}
	// Record rollback before executing.
	if c.caps.RollbackStack != nil {
		if err := c.caps.RollbackStack.AppendAppendFile(c.caps.Phase, c.caps.Component, path, marker); err != nil {
			return nil, fmt.Errorf("append_file: record rollback: %w", err)
		}
	}
	block := fmt.Sprintf("\n# BEGIN meowctl:%s\n%s\n# END meowctl:%s\n", marker, string(content), marker)
	f, err := os.OpenFile(path, os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0o600) // #nosec G304
	if err != nil {
		return nil, fmt.Errorf("append_file: %w", err)
	}
	defer func() {
		if cerr := f.Close(); cerr != nil && retErr == nil {
			retErr = fmt.Errorf("append_file: close: %w", cerr)
		}
	}()
	if _, err = f.WriteString(block); err != nil {
		return nil, fmt.Errorf("append_file: write: %w", err)
	}
	return gostarlark.None, nil
}

// starDeleteFile implements ctx.delete_file(dst).
// No-op if the file does not exist.
func (c *CtxValue) starDeleteFile(_ *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (gostarlark.Value, error) {
	var dst gostarlark.String
	if err := gostarlark.UnpackArgs("delete_file", args, kwargs, "dst", &dst); err != nil {
		return nil, err
	}
	path := string(dst)
	if err := requirePath("delete_file", path); err != nil {
		return nil, err
	}
	path, err := expandPath(path)
	if err != nil {
		return nil, fmt.Errorf("delete_file: %w", err)
	}
	if c.caps.DryRun {
		c.dryLog("delete_file", "path="+path)
		return gostarlark.None, nil
	}
	err = os.Remove(path)
	if err != nil && !os.IsNotExist(err) {
		return nil, fmt.Errorf("delete_file: %w", err)
	}
	return gostarlark.None, nil
}

// starCopyFile implements ctx.copy_file(src, dst).
func (c *CtxValue) starCopyFile(_ *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (_ gostarlark.Value, retErr error) {
	var src, dst gostarlark.String
	if err := gostarlark.UnpackArgs("copy_file", args, kwargs, "src", &src, "dst", &dst); err != nil {
		return nil, err
	}
	srcPath, dstPath := string(src), string(dst)
	if err := requirePath("copy_file", srcPath); err != nil {
		return nil, err
	}
	if err := requirePath("copy_file", dstPath); err != nil {
		return nil, err
	}
	srcPath, err := expandPath(srcPath)
	if err != nil {
		return nil, fmt.Errorf("copy_file: %w", err)
	}
	dstPath, err = expandPath(dstPath)
	if err != nil {
		return nil, fmt.Errorf("copy_file: %w", err)
	}
	if c.caps.DryRun {
		c.dryLog("copy_file", "src="+srcPath, "dst="+dstPath)
		return gostarlark.None, nil
	}
	if c.caps.RollbackStack != nil {
		if err := c.caps.RollbackStack.AppendCopyFile(c.caps.Phase, c.caps.Component, dstPath); err != nil {
			return nil, fmt.Errorf("copy_file: record rollback: %w", err)
		}
	}
	// Note: parent directories created by MkdirAll below are NOT individually
	// rollback-recorded. Rollback removes the file but leaves any new parent dirs.
	if err := os.MkdirAll(filepath.Dir(dstPath), 0o750); err != nil {
		return nil, fmt.Errorf("copy_file: mkdir: %w", err)
	}
	in, err := os.Open(srcPath) // #nosec G304
	if err != nil {
		return nil, fmt.Errorf("copy_file: open src: %w", err)
	}
	defer func() { _ = in.Close() }()
	out, err := os.Create(dstPath) // #nosec G304
	if err != nil {
		return nil, fmt.Errorf("copy_file: create dst: %w", err)
	}
	defer func() {
		if cerr := out.Close(); cerr != nil && retErr == nil {
			retErr = fmt.Errorf("copy_file: close dst: %w", cerr)
		}
	}()
	if _, err = io.Copy(out, in); err != nil {
		return nil, fmt.Errorf("copy_file: copy: %w", err)
	}
	return gostarlark.None, nil
}

// starSymlink implements ctx.symlink(src, dst).
func (c *CtxValue) starSymlink(_ *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (gostarlark.Value, error) {
	var src, dst gostarlark.String
	if err := gostarlark.UnpackArgs("symlink", args, kwargs, "src", &src, "dst", &dst); err != nil {
		return nil, err
	}
	srcPath, err := resolvePath("symlink", string(src))
	if err != nil {
		return nil, err
	}
	dstPath, err := resolvePath("symlink", string(dst))
	if err != nil {
		return nil, err
	}
	if c.caps.DryRun {
		c.dryLog("symlink", "src="+srcPath, "dst="+dstPath)
		return gostarlark.None, nil
	}
	if err := os.MkdirAll(filepath.Dir(dstPath), 0o750); err != nil {
		return nil, fmt.Errorf("symlink: mkdir: %w", err)
	}
	// Lstat-check before any destructive action. Refuse to clobber regular files.
	var priorTarget string
	var hadPrior bool
	if fi, err := os.Lstat(dstPath); err == nil {
		if fi.Mode()&os.ModeSymlink == 0 {
			return nil, fmt.Errorf("symlink: %s exists and is not a symlink; refusing to overwrite", dstPath)
		}
		// dst is an existing symlink — read its target for rollback.
		target, readErr := os.Readlink(dstPath)
		if readErr != nil {
			return nil, fmt.Errorf("symlink: readlink %s: %w", dstPath, readErr)
		}
		priorTarget = target
		hadPrior = true
		// Fall through to Remove below.
	} else if !os.IsNotExist(err) {
		return nil, fmt.Errorf("symlink: lstat %s: %w", dstPath, err)
	}
	// Record intent before any destructive operation so that a failed journal
	// write never leaves us with the old symlink already removed.
	if c.caps.RollbackStack != nil {
		if err := c.caps.RollbackStack.AppendSymlink(c.caps.Phase, c.caps.Component, dstPath, priorTarget, hadPrior); err != nil {
			return nil, fmt.Errorf("symlink: record rollback: %w", err)
		}
	}
	// Remove existing symlink now that the rollback record is safely written.
	if err := os.Remove(dstPath); err != nil && !os.IsNotExist(err) {
		return nil, fmt.Errorf("symlink: remove existing symlink %s: %w", dstPath, err)
	}
	if err := os.Symlink(srcPath, dstPath); err != nil {
		return nil, fmt.Errorf("symlink: %w", err)
	}
	return gostarlark.None, nil
}

// starRemoveSymlink implements ctx.remove_symlink(dst).
// No-op if absent.
func (c *CtxValue) starRemoveSymlink(_ *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (gostarlark.Value, error) {
	var dst gostarlark.String
	if err := gostarlark.UnpackArgs("remove_symlink", args, kwargs, "dst", &dst); err != nil {
		return nil, err
	}
	path := string(dst)
	if err := requirePath("remove_symlink", path); err != nil {
		return nil, err
	}
	path, err := expandPath(path)
	if err != nil {
		return nil, fmt.Errorf("remove_symlink: %w", err)
	}
	if c.caps.DryRun {
		c.dryLog("remove_symlink", "path="+path)
		return gostarlark.None, nil
	}
	if err := os.Remove(path); err != nil && !os.IsNotExist(err) {
		return nil, fmt.Errorf("remove_symlink: %w", err)
	}
	return gostarlark.None, nil
}

// prepareLinkFileDst inspects the current state of dstPath and returns the
// backup path and wasBackedUp flag. It creates any missing parent directories.
// Returns an error if dstPath exists as a non-symlink and backup is false.
func prepareLinkFileDst(dstPath string, backup bool) (backupPath string, wasBackedUp bool, err error) {
	if mkErr := os.MkdirAll(filepath.Dir(dstPath), 0o750); mkErr != nil {
		return "", false, fmt.Errorf("link_file: mkdir: %w", mkErr)
	}
	fi, lstatErr := os.Lstat(dstPath)
	if os.IsNotExist(lstatErr) {
		return "", false, nil
	}
	if lstatErr != nil {
		return "", false, fmt.Errorf("link_file: lstat %s: %w", dstPath, lstatErr)
	}
	if fi.Mode()&os.ModeSymlink != 0 {
		// Existing symlink pointing elsewhere — caller will remove it without backup.
		return "", false, nil
	}
	// Regular file (or other non-symlink type).
	if !backup {
		return "", false, fmt.Errorf("link_file: %s exists and is not a symlink; pass backup=True to allow backup", dstPath)
	}
	ts := fmt.Sprintf("%d", time.Now().UnixNano())
	return dstPath + ".meowctl-bak." + ts, true, nil
}

// starLinkFile implements ctx.link_file(src, dst, backup=True).
// src is resolved relative to caps.ComponentDir. dst must be an absolute path.
// If dst already points to the resolved src, this is a no-op. If dst exists
// as a regular file and backup=True, it is renamed to dst.meowctl-bak.<ts>
// before the symlink is created. If backup=False and dst is a regular file,
// the call returns an error rather than overwriting it.
func (c *CtxValue) starLinkFile(_ *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (gostarlark.Value, error) {
	var src, dst gostarlark.String
	backup := gostarlark.Bool(true)
	if err := gostarlark.UnpackArgs("link_file", args, kwargs, "src", &src, "dst", &dst, "backup?", &backup); err != nil {
		return nil, err
	}
	srcRel, dstPath := string(src), string(dst)
	if filepath.IsAbs(srcRel) {
		return nil, fmt.Errorf("link_file: src must be a relative path; got %q", srcRel)
	}
	dstPath, err := resolvePath("link_file", dstPath)
	if err != nil {
		return nil, err
	}
	srcPath := filepath.Join(c.caps.ComponentDir, srcRel)
	if c.caps.DryRun {
		c.dryLog("link_file", "src="+srcPath, "dst="+dstPath, fmt.Sprintf("backup=%v", bool(backup)))
		return gostarlark.None, nil
	}
	// Check if dst already points to srcPath — no-op.
	if target, err := os.Readlink(dstPath); err == nil && target == srcPath {
		return gostarlark.None, nil
	}
	backupPath, wasBackedUp, err := prepareLinkFileDst(dstPath, bool(backup))
	if err != nil {
		return nil, err
	}
	// Record intent before any destructive operation.
	if c.caps.RollbackStack != nil {
		if err := c.caps.RollbackStack.AppendLinkFile(c.caps.Phase, c.caps.Component, dstPath, backupPath, wasBackedUp); err != nil {
			return nil, fmt.Errorf("link_file: record rollback: %w", err)
		}
	}
	if wasBackedUp {
		if err := os.Rename(dstPath, backupPath); err != nil {
			return nil, fmt.Errorf("link_file: backup %s -> %s: %w", dstPath, backupPath, err)
		}
	} else {
		// Remove existing symlink if present (no-op if absent).
		if err := os.Remove(dstPath); err != nil && !os.IsNotExist(err) {
			return nil, fmt.Errorf("link_file: remove existing %s: %w", dstPath, err)
		}
	}
	if err := os.Symlink(srcPath, dstPath); err != nil {
		return nil, fmt.Errorf("link_file: %w", err)
	}
	return gostarlark.None, nil
}

// starMkdir implements ctx.mkdir(path).
// No-op if the directory already exists. Records rollback only when it creates.
func (c *CtxValue) starMkdir(_ *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (gostarlark.Value, error) {
	var path gostarlark.String
	if err := gostarlark.UnpackArgs("mkdir", args, kwargs, "path", &path); err != nil {
		return nil, err
	}
	p := string(path)
	if err := requirePath("mkdir", p); err != nil {
		return nil, err
	}
	p, err := expandPath(p)
	if err != nil {
		return nil, fmt.Errorf("mkdir: %w", err)
	}
	if c.caps.DryRun {
		c.dryLog("mkdir", "path="+p)
		return gostarlark.None, nil
	}
	// Use os.Mkdir to atomically determine whether we create the directory.
	// os.MkdirAll is used for the actual creation to handle missing parents,
	// but we first attempt os.Mkdir to get exclusive-create semantics: if it
	// succeeds we know we created the leaf; if it returns EEXIST the directory
	// already existed and we must not delete it on rollback.
	// Known limitation: when intermediate directories are missing, os.Mkdir
	// returns ENOENT, so we fall back to MkdirAll with created=false. This
	// means the leaf directory is NOT removed on rollback (conservative choice
	// to avoid deleting directories we may not own). Operators should check for
	// orphaned directories manually if a multi-level mkdir is rolled back.
	mkdirErr := os.Mkdir(p, 0o750)
	created := mkdirErr == nil
	if mkdirErr != nil && !os.IsExist(mkdirErr) {
		// Real error (e.g. missing parent). Fall back to MkdirAll which creates
		// intermediate directories. If MkdirAll succeeds we cannot tell whether
		// we created the leaf, so conservatively set created=false to avoid
		// destroying a directory we may not own during rollback.
		if err := os.MkdirAll(p, 0o750); err != nil {
			return nil, fmt.Errorf("mkdir: %w", err)
		}
		created = false
	}
	// Record rollback only after MkdirAll succeeds to avoid a phantom record
	// for a directory that was never actually created.
	if c.caps.RollbackStack != nil {
		if err := c.caps.RollbackStack.AppendMkdir(c.caps.Phase, c.caps.Component, p, created); err != nil {
			// Rollback record could not be written; undo the directory only if
			// *this call* created it (created=true). If it pre-existed, leave it.
			if created {
				if removeErr := os.Remove(p); removeErr != nil && !os.IsNotExist(removeErr) {
					return nil, fmt.Errorf("mkdir: record rollback: %w; also failed to undo directory: %v", err, removeErr)
				}
			}
			return nil, fmt.Errorf("mkdir: record rollback: %w", err)
		}
	}
	return gostarlark.None, nil
}

// starReadFile implements ctx.read_file(path) -> str.
func (c *CtxValue) starReadFile(_ *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (gostarlark.Value, error) {
	var path gostarlark.String
	if err := gostarlark.UnpackArgs("read_file", args, kwargs, "path", &path); err != nil {
		return nil, err
	}
	p, err := expandPath(string(path))
	if err != nil {
		return nil, fmt.Errorf("read_file: %w", err)
	}
	data, err := os.ReadFile(p) // #nosec G304
	if err != nil {
		return nil, fmt.Errorf("read_file: %w", err)
	}
	return gostarlark.String(data), nil
}

// starFileExists implements ctx.file_exists(path) -> bool.
func (c *CtxValue) starFileExists(_ *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (gostarlark.Value, error) {
	var path gostarlark.String
	if err := gostarlark.UnpackArgs("file_exists", args, kwargs, "path", &path); err != nil {
		return nil, err
	}
	p, err := expandPath(string(path))
	if err != nil {
		return nil, fmt.Errorf("file_exists: %w", err)
	}
	_, err = os.Lstat(p)
	if os.IsNotExist(err) {
		return gostarlark.False, nil
	}
	if err != nil {
		return nil, fmt.Errorf("file_exists: %w", err)
	}
	return gostarlark.True, nil
}

// starListDir implements ctx.list_dir(path) -> list[str].
func (c *CtxValue) starListDir(_ *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (gostarlark.Value, error) {
	var path gostarlark.String
	if err := gostarlark.UnpackArgs("list_dir", args, kwargs, "path", &path); err != nil {
		return nil, err
	}
	p, err := expandPath(string(path))
	if err != nil {
		return nil, fmt.Errorf("list_dir: %w", err)
	}
	entries, err := os.ReadDir(p)
	if err != nil {
		return nil, fmt.Errorf("list_dir: %w", err)
	}
	vals := make([]gostarlark.Value, len(entries))
	for i, e := range entries {
		vals[i] = gostarlark.String(e.Name())
	}
	return gostarlark.NewList(vals), nil
}

// starRun implements ctx.run(cmd, args=[], env={}, cwd=None).
// Returns struct(stdout, stderr, exit_code).
func (c *CtxValue) starRun(_ *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (gostarlark.Value, error) {
	var cmd gostarlark.String
	runArgs := &gostarlark.List{}
	runEnv := &gostarlark.Dict{}
	var cwd gostarlark.Value = gostarlark.None
	if err := gostarlark.UnpackArgs(
		"run", args, kwargs,
		"cmd", &cmd,
		"args?", &runArgs,
		"env?", &runEnv,
		"cwd?", &cwd,
	); err != nil {
		return nil, err
	}
	if c.caps.DryRun {
		c.dryLog("run", "cmd="+string(cmd))
		return starlarkRunResult("", "", 0), nil
	}
	cmdArgs, err := runArgsList(runArgs)
	if err != nil {
		return nil, err
	}
	if c.caps.Verbose {
		c.verboseLog("run", append([]string{"cmd=" + string(cmd)}, cmdArgs...)...)
	}
	mergedEnv, err := mergeRunEnv(runEnv)
	if err != nil {
		return nil, err
	}
	if c.caps.RunFunc != nil {
		stdout, err := c.caps.RunFunc(context.Background(), string(cmd), cmdArgs, mergedEnv)
		if err != nil {
			return starlarkRunResult("", err.Error(), 1), nil
		}
		return starlarkRunResult(stdout, "", 0), nil
	}
	stdout, stderr, exitCode, err := c.execRun(string(cmd), cmdArgs, mergedEnv, cwd)
	if err != nil {
		return nil, err
	}
	if c.caps.Verbose {
		c.verboseRunOutput(stdout, stderr, exitCode)
	}
	return starlarkRunResult(stdout, stderr, exitCode), nil
}

// runArgsList converts a Starlark list to a []string of command arguments.
func runArgsList(runArgs *gostarlark.List) ([]string, error) {
	cmdArgs := make([]string, runArgs.Len())
	for i := range cmdArgs {
		s, ok := runArgs.Index(i).(gostarlark.String)
		if !ok {
			return nil, fmt.Errorf("run: args[%d] must be a string", i)
		}
		cmdArgs[i] = string(s)
	}
	return cmdArgs, nil
}

// mergeRunEnv builds the merged environment (process env + caller overrides).
func mergeRunEnv(runEnv *gostarlark.Dict) ([]string, error) {
	mergedEnv := os.Environ()
	for _, kv := range runEnv.Items() {
		k, ok1 := kv[0].(gostarlark.String)
		v, ok2 := kv[1].(gostarlark.String)
		if !ok1 || !ok2 {
			return nil, fmt.Errorf("run: env keys and values must be strings")
		}
		mergedEnv = append(mergedEnv, string(k)+"="+string(v))
	}
	return mergedEnv, nil
}

// execRun runs a command and returns stdout, stderr, exit code, and any non-exit error.
func (c *CtxValue) execRun(cmd string, cmdArgs, mergedEnv []string, cwd gostarlark.Value) (string, string, int, error) {
	c2 := exec.CommandContext(context.Background(), cmd, cmdArgs...) // #nosec G204 -- cmd/args from validated Starlark config; no ctx available in hooks
	c2.Env = mergedEnv
	if cwdStr, ok := cwd.(gostarlark.String); ok {
		dir, err := expandPath(string(cwdStr))
		if err != nil {
			return "", "", 0, fmt.Errorf("run: %w", err)
		}
		c2.Dir = dir
	}
	var stdoutBuf, stderrBuf strings.Builder
	c2.Stdout = &stdoutBuf
	c2.Stderr = &stderrBuf
	runErr := c2.Run()
	exitCode := 0
	if runErr != nil {
		if exitErr, ok := runErr.(*exec.ExitError); ok {
			exitCode = exitErr.ExitCode()
		} else {
			return "", "", 0, fmt.Errorf("run: %w", runErr)
		}
	}
	return stdoutBuf.String(), stderrBuf.String(), exitCode, nil
}

// verboseRunOutput logs stdout, stderr, and exit code after a run.
func (c *CtxValue) verboseRunOutput(stdout, stderr string, exitCode int) {
	if out := strings.TrimRight(stdout, "\n"); out != "" {
		for _, line := range strings.Split(out, "\n") {
			c.logMsg(fmt.Sprintf("[verbose] stdout             component=%-20s %s", c.caps.Component, line))
		}
	}
	if out := strings.TrimRight(stderr, "\n"); out != "" {
		for _, line := range strings.Split(out, "\n") {
			c.logMsg(fmt.Sprintf("[verbose] stderr             component=%-20s %s", c.caps.Component, line))
		}
	}
	if exitCode != 0 {
		c.verboseLog("exit_code", fmt.Sprintf("code=%d", exitCode))
	}
}

func starlarkRunResult(stdout, stderr string, exitCode int) gostarlark.Value {
	return starlarkstruct.FromStringDict(starlarkstruct.Default, gostarlark.StringDict{
		"stdout":    gostarlark.String(stdout),
		"stderr":    gostarlark.String(stderr),
		"exit_code": gostarlark.MakeInt(exitCode),
	})
}

// starGitClone implements ctx.git_clone(url, dst, ref=None).
func (c *CtxValue) starGitClone(_ *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (gostarlark.Value, error) {
	var rawURL, dst gostarlark.String
	var ref gostarlark.Value = gostarlark.None
	if err := gostarlark.UnpackArgs(
		"git_clone", args, kwargs,
		"url", &rawURL,
		"dst", &dst,
		"ref?", &ref,
	); err != nil {
		return nil, err
	}
	if c.caps.DryRun {
		c.dryLog("git_clone", "url="+string(rawURL), "dst="+string(dst))
		return gostarlark.None, nil
	}
	dstPath := string(dst)
	dstPath, err := expandPath(dstPath)
	if err != nil {
		return nil, fmt.Errorf("git_clone: %w", err)
	}
	// If destination exists, skip clone (idempotent).
	if _, err := os.Stat(dstPath); err == nil {
		return gostarlark.None, nil
	}
	cloneArgs := []string{"clone", string(rawURL), dstPath}
	if refStr, ok := ref.(gostarlark.String); ok && string(refStr) != "" {
		cloneArgs = append(cloneArgs, "--branch", string(refStr))
	}
	cmd := exec.CommandContext(context.Background(), "git", cloneArgs...) // #nosec G204 -- args from validated Starlark strings; no ctx available in hooks
	if out, err := cmd.CombinedOutput(); err != nil {
		return nil, fmt.Errorf("git_clone: %w\n%s", err, out)
	}
	return gostarlark.None, nil
}

// starDownload implements ctx.download(url, dst, checksum=None).
func (c *CtxValue) starDownload(_ *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (_ gostarlark.Value, retErr error) {
	var rawURL, dst gostarlark.String
	var checksum gostarlark.Value = gostarlark.None
	if err := gostarlark.UnpackArgs(
		"download", args, kwargs,
		"url", &rawURL,
		"dst", &dst,
		"checksum?", &checksum,
	); err != nil {
		return nil, err
	}
	dstPath := string(dst)
	dstPath, err := expandPath(dstPath)
	if err != nil {
		return nil, fmt.Errorf("download: %w", err)
	}
	if c.caps.DryRun {
		c.dryLog("download", "url="+string(rawURL), "dst="+dstPath)
		return gostarlark.None, nil
	}
	if err := os.MkdirAll(filepath.Dir(dstPath), 0o750); err != nil {
		return nil, fmt.Errorf("download: mkdir: %w", err)
	}
	dlCtx, dlCancel := context.WithTimeout(context.Background(), 5*time.Minute)
	defer dlCancel()
	req, err := http.NewRequestWithContext(dlCtx, http.MethodGet, string(rawURL), nil) // #nosec G107
	if err != nil {
		return nil, fmt.Errorf("download: build request: %w", err)
	}
	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		return nil, fmt.Errorf("download: GET: %w", err)
	}
	defer func() { _ = resp.Body.Close() }()
	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("download: server returned %d", resp.StatusCode)
	}
	if err := c.downloadToFile(dstPath, resp.Body); err != nil {
		return nil, err
	}
	return gostarlark.None, nil
}

// downloadToFile writes body to dstPath, recording rollback and preserving any
// prior content. Extracted to keep starDownload below the cyclomatic complexity limit.
func (c *CtxValue) downloadToFile(dstPath string, body io.Reader) (retErr error) {
	// Check if dstPath already exists so we can record its prior content for
	// rollback. Without this, os.Create below would truncate a pre-existing file
	// before any rollback record is written, permanently destroying its content.
	var (
		priorContent string
		hadPrior     bool
	)
	if existing, readErr := os.ReadFile(dstPath); readErr == nil { // #nosec G304
		priorContent = string(existing)
		hadPrior = true
	} else if !os.IsNotExist(readErr) {
		return fmt.Errorf("download: read existing file: %w", readErr)
	}
	// Write the rollback record BEFORE os.Create so that prior content is
	// covered even if the journal write fails (we never reach the truncation).
	if c.caps.RollbackStack != nil {
		if err := c.caps.RollbackStack.AppendDownload(c.caps.Phase, c.caps.Component, dstPath, priorContent, hadPrior); err != nil {
			return fmt.Errorf("download: record rollback: %w", err)
		}
	}
	f, err := os.Create(dstPath) // #nosec G304
	if err != nil {
		return fmt.Errorf("download: create: %w", err)
	}
	var fClosed bool
	defer func() {
		if fClosed {
			return
		}
		if cerr := f.Close(); cerr != nil && retErr == nil {
			retErr = fmt.Errorf("download: close: %w", cerr)
			// On close failure the written data may not be flushed.
			// If there was no prior file, remove the corrupt artifact.
			// If there was a prior file, the rollback record will restore it;
			// do not remove so that rollback has something to overwrite.
			if !hadPrior {
				_ = os.Remove(dstPath)
			}
		}
	}()
	if _, err := io.Copy(f, body); err != nil {
		fClosed = true
		closeErr := f.Close()
		cleanErr := cleanupPartialDownload(dstPath, hadPrior)
		return combineDownloadErrors(err, closeErr, cleanErr)
	}
	return nil
}

// cleanupPartialDownload removes a partially-written download file.
// It only removes the file when hadPrior is false to avoid permanent data loss:
// when a prior file existed and rollback is disabled, the caller's rollback
// record (if present) will restore it; removing here would destroy the only copy.
func cleanupPartialDownload(dstPath string, hadPrior bool) error {
	if hadPrior {
		return nil
	}
	if err := os.Remove(dstPath); err != nil && !os.IsNotExist(err) {
		return err
	}
	return nil
}

// combineDownloadErrors assembles a single error from the three failure modes
// that can occur in the io.Copy error path of downloadToFile.
// writeErr is wrapped with %w to preserve chain inspection; closeErr and cleanErr
// are secondary and formatted with %w as well for completeness.
func combineDownloadErrors(writeErr, closeErr, cleanErr error) error {
	if cleanErr != nil && closeErr != nil {
		return fmt.Errorf("download: write: %w; close: %w; also failed to remove partial file: %w", writeErr, closeErr, cleanErr)
	}
	if cleanErr != nil {
		return fmt.Errorf("download: write: %w; also failed to remove partial file: %w", writeErr, cleanErr)
	}
	if closeErr != nil {
		return fmt.Errorf("download: write: %w; close: %w", writeErr, closeErr)
	}
	return fmt.Errorf("download: write: %w", writeErr)
}

// starDefaultsWrite implements ctx.defaults_write(domain, key, type, value).
// macOS only; executes `defaults write` subprocess.
func (c *CtxValue) starDefaultsWrite(_ *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (gostarlark.Value, error) {
	var domain, key, valueType gostarlark.String
	var value gostarlark.Value
	if err := gostarlark.UnpackArgs(
		"defaults_write", args, kwargs,
		"domain", &domain,
		"key", &key,
		"type", &valueType,
		"value", &value,
	); err != nil {
		return nil, err
	}
	if c.caps.DryRun {
		c.dryLog("defaults_write", "domain="+string(domain), "key="+string(key))
		return gostarlark.None, nil
	}
	// defaults write expects booleans as "YES"/"NO"; %v on gostarlark.Bool produces
	// "True"/"False" (Starlark casing), which the command would reject.
	valStr := starlarkValueToShellArg(value)
	cmd := exec.CommandContext(context.Background(), "defaults", "write", string(domain), string(key), "-"+string(valueType), valStr) // #nosec G204 -- args from validated Starlark config; no ctx available in hooks
	if out, err := cmd.CombinedOutput(); err != nil {
		return nil, fmt.Errorf("defaults_write: %w\n%s", err, out)
	}
	return gostarlark.None, nil
}

// starPlistSet implements ctx.plist_set(file, key, type, value).
// macOS only; uses PlistBuddy.
func (c *CtxValue) starPlistSet(_ *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (gostarlark.Value, error) {
	var file, key, valueType gostarlark.String
	var value gostarlark.Value
	if err := gostarlark.UnpackArgs(
		"plist_set", args, kwargs,
		"file", &file,
		"key", &key,
		"type", &valueType,
		"value", &value,
	); err != nil {
		return nil, err
	}
	if c.caps.DryRun {
		c.dryLog("plist_set", "file="+string(file), "key="+string(key))
		return gostarlark.None, nil
	}
	filePath, err := expandPath(string(file))
	if err != nil {
		return nil, fmt.Errorf("plist_set: %w", err)
	}
	// PlistBuddy expects booleans as "true"/"false" (lowercase); %v on gostarlark.Bool
	// produces "True"/"False" (Starlark casing), which PlistBuddy would reject.
	valStr := starlarkValueToShellArg(value)
	// PlistBuddy parses the -c string itself; keys or values containing spaces
	// must be quoted so PlistBuddy does not split them into extra tokens.
	cmdStr := fmt.Sprintf("Set :%s %s", shellQuote(string(key)), shellQuote(valStr))
	cmd := exec.CommandContext(context.Background(), "/usr/libexec/PlistBuddy", "-c", cmdStr, filePath) // #nosec G204 -- args from validated Starlark config; no ctx available in hooks
	if out, err := cmd.CombinedOutput(); err != nil {
		return nil, fmt.Errorf("plist_set: %w\n%s", err, out)
	}
	return gostarlark.None, nil
}

// starPrompt implements ctx.prompt(question) -> str.
// Reads a line from stdin interactively.
func (c *CtxValue) starPrompt(_ *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (gostarlark.Value, error) {
	var question gostarlark.String
	if err := gostarlark.UnpackArgs("prompt", args, kwargs, "question", &question); err != nil {
		return nil, err
	}
	if c.caps.DryRun {
		c.dryLog("prompt", "question="+string(question))
		return gostarlark.String(""), nil
	}
	fmt.Print(string(question) + " ")
	sc := bufio.NewScanner(os.Stdin)
	if sc.Scan() {
		return gostarlark.String(sc.Text()), nil
	}
	if err := sc.Err(); err != nil {
		return nil, fmt.Errorf("prompt: %w", err)
	}
	return gostarlark.String(""), nil
}

// starEmit implements ctx.emit(line).
// Writes line to stdout so the calling shell can eval it.
// Only meaningful inside shell or login hook phases (meowctl hook shell/login).
// Outside those phases RuntimeHook is false and emit is a no-op, ensuring
// components that define shell(ctx) can be safely included in any phase set
// without producing spurious output.
func (c *CtxValue) starEmit(_ *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (gostarlark.Value, error) {
	var line gostarlark.String
	if err := gostarlark.UnpackArgs("emit", args, kwargs, "line", &line); err != nil {
		return nil, err
	}
	if c.caps.DryRun {
		c.dryLog("emit", "line="+string(line))
		return gostarlark.None, nil
	}
	if !c.caps.RuntimeHook {
		// Not running inside meowctl hook — silently ignore.
		return gostarlark.None, nil
	}
	fmt.Println(string(line))
	return gostarlark.None, nil
}

// starAddPath implements ctx.add_path(dir) -> None.
// Prepends dir to the current process PATH so that subsequent ctx.run calls
// (which snapshot os.Environ()) can find executables installed there.
// This is intentionally a live mutation of the process environment — it is
// the correct mechanism for components like mise that install tools whose
// binaries are not yet on PATH in the current session.
func (c *CtxValue) starAddPath(_ *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (gostarlark.Value, error) {
	var dir gostarlark.String
	if err := gostarlark.UnpackArgs("add_path", args, kwargs, "dir", &dir); err != nil {
		return nil, err
	}
	d := string(dir)
	if d == "" {
		return gostarlark.None, nil
	}
	if c.caps.DryRun {
		c.dryLog("add_path", "dir="+d)
		return gostarlark.None, nil
	}
	current := os.Getenv("PATH")
	// Avoid duplicates: skip if dir is already the first entry.
	if !strings.HasPrefix(current, d+string(os.PathListSeparator)) && current != d {
		var newPath string
		if current == "" {
			newPath = d
		} else {
			newPath = d + string(os.PathListSeparator) + current
		}
		if err := os.Setenv("PATH", newPath); err != nil {
			return nil, fmt.Errorf("add_path: setenv PATH: %w", err)
		}
		c.verboseLog("add_path", "dir="+d)
	}
	return gostarlark.None, nil
}

// starRender implements ctx.render(template_str, vars) -> str.
// Replaces {{VAR}} placeholders.
func (c *CtxValue) starRender(_ *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (gostarlark.Value, error) {
	var templateStr gostarlark.String
	var vars *gostarlark.Dict
	if err := gostarlark.UnpackArgs(
		"render", args, kwargs,
		"template_str", &templateStr,
		"vars", &vars,
	); err != nil {
		return nil, err
	}
	result, err := renderTemplate(string(templateStr), vars)
	if err != nil {
		return nil, err
	}
	return gostarlark.String(result), nil
}

// starRenderFile implements ctx.render_file(src, vars) -> str.
// Reads src from ctx.component_dir and renders {{VAR}} placeholders.
func (c *CtxValue) starRenderFile(_ *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (gostarlark.Value, error) {
	var src gostarlark.String
	var vars *gostarlark.Dict
	if err := gostarlark.UnpackArgs(
		"render_file", args, kwargs,
		"src", &src,
		"vars", &vars,
	); err != nil {
		return nil, err
	}
	full := filepath.Join(c.caps.ComponentDir, string(src))
	data, err := os.ReadFile(full) // #nosec G304
	if err != nil {
		return nil, fmt.Errorf("render_file: %w", err)
	}
	result, err := renderTemplate(string(data), vars)
	if err != nil {
		return nil, err
	}
	return gostarlark.String(result), nil
}

// renderTemplate replaces {{KEY}} placeholders in tmpl using vars dict.
func renderTemplate(tmpl string, vars *gostarlark.Dict) (string, error) {
	if vars == nil {
		return tmpl, nil
	}
	result := tmpl
	for _, kv := range vars.Items() {
		k, ok1 := kv[0].(gostarlark.String)
		v, ok2 := kv[1].(gostarlark.String)
		if !ok1 || !ok2 {
			return "", fmt.Errorf("render: vars keys and values must be strings")
		}
		result = strings.ReplaceAll(result, "{{"+string(k)+"}}", string(v))
	}
	return result, nil
}

// starlarkValueToShellArg converts a Starlark value to a shell argument string.
// Booleans are converted to "true"/"false" (lowercase). Both `defaults write`
// and PlistBuddy accept lowercase boolean strings, avoiding the Starlark
// default of "True"/"False" which both commands would reject.
func starlarkValueToShellArg(v gostarlark.Value) string {
	if b, ok := v.(gostarlark.Bool); ok {
		if b {
			return "true"
		}
		return "false"
	}
	return fmt.Sprintf("%v", v)
}

// shellQuote wraps s in double-quotes if it contains any whitespace, double-quote,
// or backslash characters, ensuring it is parsed as a single unambiguous token
// by PlistBuddy's internal command parser.
// Backslashes are escaped before double-quotes to avoid partial escape sequences.
func shellQuote(s string) string {
	if strings.ContainsAny(s, " \t\n\r\"\\") {
		s = strings.ReplaceAll(s, `\`, `\\`)
		s = strings.ReplaceAll(s, `"`, `\"`)
		return `"` + s + `"`
	}
	return s
}

// uuidFallbackCounter provides unique values when crypto/rand is unavailable.
var uuidFallbackCounter atomic.Uint64

// newUUID generates a UUID v4-like string using crypto/rand.
// crypto/rand is pure Go and CGO-free; it works correctly on all supported platforms.
func newUUID() string {
	b := make([]byte, 16)
	if _, err := rand.Read(b); err != nil {
		// crypto/rand failure is exceptional; use a time+counter pair to ensure
		// the fallback is unique across concurrent calls in the same process.
		n := uuidFallbackCounter.Add(1)
		return fmt.Sprintf("fallback-%016x%016x", time.Now().UnixNano(), n)
	}
	b[6] = (b[6] & 0x0f) | 0x40
	b[8] = (b[8] & 0x3f) | 0x80
	return fmt.Sprintf("%08x-%04x-%04x-%04x-%012x",
		b[0:4], b[4:6], b[6:8], b[8:10], b[10:16])
}

// starWhich implements ctx.which(name) -> bool.
// Returns True if the named executable is found in PATH (respecting any
// paths added via ctx.add_path), False otherwise.
func (c *CtxValue) starWhich(_ *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (gostarlark.Value, error) {
	var name gostarlark.String
	if err := gostarlark.UnpackArgs("which", args, kwargs, "name", &name); err != nil {
		return nil, err
	}
	_, err := exec.LookPath(string(name))
	if err != nil {
		return gostarlark.False, nil
	}
	return gostarlark.True, nil
}
