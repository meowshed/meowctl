package ctx

import (
	"bufio"
	"crypto/rand"
	"fmt"
	"io"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"strings"

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

// dryLog emits a dry-run line for the given op and args.
func (c *CtxValue) dryLog(op string, args ...string) {
	c.logMsg(fmt.Sprintf("[dry-run] %-16s component=%-20s %s", op, c.caps.Component, strings.Join(args, " ")))
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
	if c.caps.DryRun {
		c.dryLog("write_file", "path="+path)
		return gostarlark.None, nil
	}
	// Read prior content for rollback.
	prior, err := os.ReadFile(path) // #nosec G304
	hadPrior := err == nil
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
func (c *CtxValue) starAppendFile(_ *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (gostarlark.Value, error) {
	var dst, content gostarlark.String
	var markerVal gostarlark.Value = gostarlark.None
	if err := gostarlark.UnpackArgs("append_file", args, kwargs,
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
		if cerr := f.Close(); cerr != nil && err == nil {
			err = fmt.Errorf("append_file: close: %w", cerr)
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
	if c.caps.DryRun {
		c.dryLog("delete_file", "path="+path)
		return gostarlark.None, nil
	}
	err := os.Remove(path)
	if err != nil && !os.IsNotExist(err) {
		return nil, fmt.Errorf("delete_file: %w", err)
	}
	return gostarlark.None, nil
}

// starCopyFile implements ctx.copy_file(src, dst).
func (c *CtxValue) starCopyFile(_ *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (gostarlark.Value, error) {
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
	if c.caps.DryRun {
		return gostarlark.None, nil
	}
	if c.caps.RollbackStack != nil {
		if err := c.caps.RollbackStack.AppendCopyFile(c.caps.Phase, c.caps.Component, dstPath); err != nil {
			return nil, fmt.Errorf("copy_file: record rollback: %w", err)
		}
	}
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
		if cerr := out.Close(); cerr != nil && err == nil {
			err = fmt.Errorf("copy_file: close dst: %w", cerr)
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
	srcPath, dstPath := string(src), string(dst)
	if err := requirePath("symlink", srcPath); err != nil {
		return nil, err
	}
	if err := requirePath("symlink", dstPath); err != nil {
		return nil, err
	}
	if c.caps.DryRun {
		c.dryLog("symlink", "src="+srcPath, "dst="+dstPath)
		return gostarlark.None, nil
	}
	if c.caps.RollbackStack != nil {
		if err := c.caps.RollbackStack.AppendSymlink(c.caps.Phase, c.caps.Component, dstPath); err != nil {
			return nil, fmt.Errorf("symlink: record rollback: %w", err)
		}
	}
	if err := os.MkdirAll(filepath.Dir(dstPath), 0o750); err != nil {
		return nil, fmt.Errorf("symlink: mkdir: %w", err)
	}
	// Remove existing symlink at dst if present so we can re-link idempotently.
	_ = os.Remove(dstPath)
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
	if c.caps.DryRun {
		c.dryLog("remove_symlink", "path="+path)
		return gostarlark.None, nil
	}
	if err := os.Remove(path); err != nil && !os.IsNotExist(err) {
		return nil, fmt.Errorf("remove_symlink: %w", err)
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
	if c.caps.DryRun {
		return gostarlark.None, nil
	}
	_, statErr := os.Stat(p)
	created := os.IsNotExist(statErr)
	if c.caps.RollbackStack != nil {
		if err := c.caps.RollbackStack.AppendMkdir(c.caps.Phase, c.caps.Component, p, created); err != nil {
			return nil, fmt.Errorf("mkdir: record rollback: %w", err)
		}
	}
	if err := os.MkdirAll(p, 0o750); err != nil {
		return nil, fmt.Errorf("mkdir: %w", err)
	}
	return gostarlark.None, nil
}

// starReadFile implements ctx.read_file(path) -> str.
func (c *CtxValue) starReadFile(_ *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (gostarlark.Value, error) {
	var path gostarlark.String
	if err := gostarlark.UnpackArgs("read_file", args, kwargs, "path", &path); err != nil {
		return nil, err
	}
	data, err := os.ReadFile(string(path)) // #nosec G304
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
	_, err := os.Stat(string(path))
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
	entries, err := os.ReadDir(string(path))
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
	if err := gostarlark.UnpackArgs("run", args, kwargs,
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
	// Build argument list.
	cmdArgs := make([]string, runArgs.Len())
	for i := range cmdArgs {
		s, ok := runArgs.Index(i).(gostarlark.String)
		if !ok {
			return nil, fmt.Errorf("run: args[%d] must be a string", i)
		}
		cmdArgs[i] = string(s)
	}
	//nolint:gosec,noctx // cmd and args come from user Starlark config — intentional.
	c2 := exec.Command(string(cmd), cmdArgs...)
	// Merge provided env over current env.
	c2.Env = os.Environ()
	for _, kv := range runEnv.Items() {
		k, ok1 := kv[0].(gostarlark.String)
		v, ok2 := kv[1].(gostarlark.String)
		if !ok1 || !ok2 {
			return nil, fmt.Errorf("run: env keys and values must be strings")
		}
		c2.Env = append(c2.Env, string(k)+"="+string(v))
	}
	if cwdStr, ok := cwd.(gostarlark.String); ok {
		c2.Dir = string(cwdStr)
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
			return nil, fmt.Errorf("run: %w", runErr)
		}
	}
	return starlarkRunResult(stdoutBuf.String(), stderrBuf.String(), exitCode), nil
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
	if err := gostarlark.UnpackArgs("git_clone", args, kwargs,
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
	// If destination exists, skip clone (idempotent).
	if _, err := os.Stat(dstPath); err == nil {
		return gostarlark.None, nil
	}
	cloneArgs := []string{"clone", string(rawURL), dstPath}
	if refStr, ok := ref.(gostarlark.String); ok && string(refStr) != "" {
		cloneArgs = append(cloneArgs, "--branch", string(refStr))
	}
	//nolint:gosec,noctx // args built from validated Starlark strings.
	cmd := exec.Command("git", cloneArgs...)
	if out, err := cmd.CombinedOutput(); err != nil {
		return nil, fmt.Errorf("git_clone: %w\n%s", err, out)
	}
	return gostarlark.None, nil
}

// starDownload implements ctx.download(url, dst, checksum=None).
func (c *CtxValue) starDownload(_ *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (gostarlark.Value, error) {
	var rawURL, dst gostarlark.String
	var checksum gostarlark.Value = gostarlark.None
	if err := gostarlark.UnpackArgs("download", args, kwargs,
		"url", &rawURL,
		"dst", &dst,
		"checksum?", &checksum,
	); err != nil {
		return nil, err
	}
	dstPath := string(dst)
	if c.caps.DryRun {
		c.dryLog("download", "url="+string(rawURL), "dst="+dstPath)
		return gostarlark.None, nil
	}
	if c.caps.RollbackStack != nil {
		if err := c.caps.RollbackStack.AppendDownload(c.caps.Phase, c.caps.Component, dstPath); err != nil {
			return nil, fmt.Errorf("download: record rollback: %w", err)
		}
	}
	if err := os.MkdirAll(filepath.Dir(dstPath), 0o750); err != nil {
		return nil, fmt.Errorf("download: mkdir: %w", err)
	}
	//nolint:gosec,noctx // URL comes from user Starlark config; no context available here.
	resp, err := http.Get(string(rawURL)) // #nosec G107
	if err != nil {
		return nil, fmt.Errorf("download: GET: %w", err)
	}
	defer func() { _ = resp.Body.Close() }()
	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("download: server returned %d", resp.StatusCode)
	}
	f, err := os.Create(dstPath) // #nosec G304
	if err != nil {
		return nil, fmt.Errorf("download: create: %w", err)
	}
	defer func() {
		if cerr := f.Close(); cerr != nil && err == nil {
			err = fmt.Errorf("download: close: %w", cerr)
		}
	}()
	if _, err = io.Copy(f, resp.Body); err != nil {
		return nil, fmt.Errorf("download: write: %w", err)
	}
	return gostarlark.None, nil
}

// starDefaultsWrite implements ctx.defaults_write(domain, key, type, value).
// macOS only; executes `defaults write` subprocess.
func (c *CtxValue) starDefaultsWrite(_ *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (gostarlark.Value, error) {
	var domain, key, valueType gostarlark.String
	var value gostarlark.Value
	if err := gostarlark.UnpackArgs("defaults_write", args, kwargs,
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
	//nolint:gosec,noctx // args from validated Starlark config.
	cmd := exec.Command("defaults", "write", string(domain), string(key), "-"+string(valueType), valStr)
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
	if err := gostarlark.UnpackArgs("plist_set", args, kwargs,
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
	// PlistBuddy expects booleans as "true"/"false" (lowercase); %v on gostarlark.Bool
	// produces "True"/"False" (Starlark casing), which PlistBuddy would reject.
	valStr := starlarkValueToShellArg(value)
	// PlistBuddy parses the -c string itself; keys or values containing spaces
	// must be quoted so PlistBuddy does not split them into extra tokens.
	cmdStr := fmt.Sprintf("Set :%s %s", shellQuote(string(key)), shellQuote(valStr))
	//nolint:gosec,noctx // args from validated Starlark config.
	cmd := exec.Command("/usr/libexec/PlistBuddy", "-c", cmdStr, string(file))
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
// Appends line to the shell init file currently being built.
func (c *CtxValue) starEmit(_ *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (gostarlark.Value, error) {
	var line gostarlark.String
	if err := gostarlark.UnpackArgs("emit", args, kwargs, "line", &line); err != nil {
		return nil, err
	}
	if c.caps.DryRun {
		c.dryLog("emit", "line="+string(line))
		return gostarlark.None, nil
	}
	// Emit is handled by the shell phase runner; for now write to stdout.
	fmt.Println(string(line))
	return gostarlark.None, nil
}

// starRender implements ctx.render(template_str, vars) -> str.
// Replaces {{VAR}} placeholders.
func (c *CtxValue) starRender(_ *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (gostarlark.Value, error) {
	var templateStr gostarlark.String
	var vars *gostarlark.Dict
	if err := gostarlark.UnpackArgs("render", args, kwargs,
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
	if err := gostarlark.UnpackArgs("render_file", args, kwargs,
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

// shellQuote wraps s in double-quotes if it contains any whitespace, ensuring
// it is parsed as a single token by PlistBuddy's internal command parser.
// Backslashes are escaped before double-quotes to avoid partial escape sequences.
func shellQuote(s string) string {
	if strings.ContainsAny(s, " \t\n\r") {
		s = strings.ReplaceAll(s, `\`, `\\`)
		s = strings.ReplaceAll(s, `"`, `\"`)
		return `"` + s + `"`
	}
	return s
}

// newUUID generates a UUID v4-like string using crypto/rand.
// crypto/rand is pure Go and CGO-free; it works correctly on all supported platforms.
func newUUID() string {
	b := make([]byte, 16)
	if _, err := rand.Read(b); err != nil {
		// crypto/rand failure is exceptional; fall back to a deterministic placeholder
		// rather than silently producing a zero UUID that could cause marker collisions.
		return fmt.Sprintf("fallback-%x", b)
	}
	b[6] = (b[6] & 0x0f) | 0x40
	b[8] = (b[8] & 0x3f) | 0x80
	return fmt.Sprintf("%08x-%04x-%04x-%04x-%012x",
		b[0:4], b[4:6], b[6:8], b[8:10], b[10:16])
}
