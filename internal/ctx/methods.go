package ctx

import (
	gostarlark "go.starlark.net/starlark"
)

// starLog implements ctx.log(msg).
// Signature: log(msg)
// Always executes, even in dry-run.
func (c *CtxValue) starLog(_ *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (gostarlark.Value, error) {
	var msg gostarlark.String
	if err := gostarlark.UnpackArgs("log", args, kwargs, "msg", &msg); err != nil {
		return nil, err
	}
	// Real implementation deferred to M5; body stub returns None without panicking.
	return gostarlark.None, nil
}

// starEnv implements ctx.env(key).
// Signature: env(key)
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
// Signature: write_file(dst, content)
// Mutating; dry-run-aware; rollback-tracked. Body deferred to M5.
func (c *CtxValue) starWriteFile(_ *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (gostarlark.Value, error) {
	var dst, content gostarlark.String
	if err := gostarlark.UnpackArgs("write_file", args, kwargs, "dst", &dst, "content", &content); err != nil {
		return nil, err
	}
	return gostarlark.None, nil
}

// starAppendFile implements ctx.append_file(dst, content, marker=None).
// Signature: append_file(dst, content, marker=None)
// Mutating; dry-run-aware; rollback-tracked. Body deferred to M5.
func (c *CtxValue) starAppendFile(_ *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (gostarlark.Value, error) {
	var dst, content gostarlark.String
	var marker gostarlark.Value = gostarlark.None
	if err := gostarlark.UnpackArgs("append_file", args, kwargs,
		"dst", &dst,
		"content", &content,
		"marker?", &marker,
	); err != nil {
		return nil, err
	}
	return gostarlark.None, nil
}

// starDeleteFile implements ctx.delete_file(dst).
// Signature: delete_file(dst)
// Mutating; dry-run-aware. No-op if not present. Body deferred to M5.
func (c *CtxValue) starDeleteFile(_ *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (gostarlark.Value, error) {
	var dst gostarlark.String
	if err := gostarlark.UnpackArgs("delete_file", args, kwargs, "dst", &dst); err != nil {
		return nil, err
	}
	return gostarlark.None, nil
}

// starCopyFile implements ctx.copy_file(src, dst).
// Signature: copy_file(src, dst)
// Mutating; dry-run-aware; rollback-tracked. Body deferred to M5.
func (c *CtxValue) starCopyFile(_ *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (gostarlark.Value, error) {
	var src, dst gostarlark.String
	if err := gostarlark.UnpackArgs("copy_file", args, kwargs, "src", &src, "dst", &dst); err != nil {
		return nil, err
	}
	return gostarlark.None, nil
}

// starSymlink implements ctx.symlink(src, dst).
// Signature: symlink(src, dst)
// Mutating; dry-run-aware; rollback-tracked. Body deferred to M5.
func (c *CtxValue) starSymlink(_ *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (gostarlark.Value, error) {
	var src, dst gostarlark.String
	if err := gostarlark.UnpackArgs("symlink", args, kwargs, "src", &src, "dst", &dst); err != nil {
		return nil, err
	}
	return gostarlark.None, nil
}

// starRemoveSymlink implements ctx.remove_symlink(dst).
// Signature: remove_symlink(dst)
// Mutating; dry-run-aware. No-op if not present. Body deferred to M5.
func (c *CtxValue) starRemoveSymlink(_ *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (gostarlark.Value, error) {
	var dst gostarlark.String
	if err := gostarlark.UnpackArgs("remove_symlink", args, kwargs, "dst", &dst); err != nil {
		return nil, err
	}
	return gostarlark.None, nil
}

// starMkdir implements ctx.mkdir(path).
// Signature: mkdir(path)
// Mutating; dry-run-aware; rollback-tracked. No-op if exists. Body deferred to M5.
func (c *CtxValue) starMkdir(_ *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (gostarlark.Value, error) {
	var path gostarlark.String
	if err := gostarlark.UnpackArgs("mkdir", args, kwargs, "path", &path); err != nil {
		return nil, err
	}
	return gostarlark.None, nil
}

// starReadFile implements ctx.read_file(path).
// Signature: read_file(path) -> str
// Non-mutating. Body deferred to M5; returns empty string in stub form.
func (c *CtxValue) starReadFile(_ *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (gostarlark.Value, error) {
	var path gostarlark.String
	if err := gostarlark.UnpackArgs("read_file", args, kwargs, "path", &path); err != nil {
		return nil, err
	}
	return gostarlark.String(""), nil
}

// starFileExists implements ctx.file_exists(path).
// Signature: file_exists(path) -> bool
// Non-mutating. Body deferred to M5; returns False in stub form.
func (c *CtxValue) starFileExists(_ *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (gostarlark.Value, error) {
	var path gostarlark.String
	if err := gostarlark.UnpackArgs("file_exists", args, kwargs, "path", &path); err != nil {
		return nil, err
	}
	return gostarlark.False, nil
}

// starListDir implements ctx.list_dir(path).
// Signature: list_dir(path) -> list[str]
// Non-mutating. Body deferred to M5; returns empty list in stub form.
func (c *CtxValue) starListDir(_ *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (gostarlark.Value, error) {
	var path gostarlark.String
	if err := gostarlark.UnpackArgs("list_dir", args, kwargs, "path", &path); err != nil {
		return nil, err
	}
	return gostarlark.NewList(nil), nil
}

// starRun implements ctx.run(cmd, args=[], env={}, cwd=None).
// Signature: run(cmd, args=[], env={}, cwd=None) -> struct(stdout, stderr, exit_code)
// Mutating; dry-run-aware; not rollback-tracked. Body deferred to M5.
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
	return gostarlark.None, nil
}

// starGitClone implements ctx.git_clone(url, dst, ref=None).
// Signature: git_clone(url, dst, ref=None)
// Mutating; dry-run-aware; not rollback-tracked. Body deferred to M5.
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
	return gostarlark.None, nil
}

// starDownload implements ctx.download(url, dst, checksum=None).
// Signature: download(url, dst, checksum=None)
// Mutating; dry-run-aware; not rollback-tracked. Body deferred to M5.
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
	return gostarlark.None, nil
}

// starDefaultsWrite implements ctx.defaults_write(domain, key, type, value).
// Signature: defaults_write(domain, key, type, value)
// macOS only; mutating; dry-run-aware; not rollback-tracked. Body deferred to M5.
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
	return gostarlark.None, nil
}

// starPlistSet implements ctx.plist_set(file, key, type, value).
// Signature: plist_set(file, key, type, value)
// macOS only; mutating; dry-run-aware; not rollback-tracked. Body deferred to M5.
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
	return gostarlark.None, nil
}

// starPrompt implements ctx.prompt(question).
// Signature: prompt(question) -> str
// Interactive; only available during init phase. Body deferred to M5.
func (c *CtxValue) starPrompt(_ *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (gostarlark.Value, error) {
	var question gostarlark.String
	if err := gostarlark.UnpackArgs("prompt", args, kwargs, "question", &question); err != nil {
		return nil, err
	}
	return gostarlark.String(""), nil
}

// starEmit implements ctx.emit(line).
// Signature: emit(line)
// Only available during shell.star execution. Body deferred to M5.
func (c *CtxValue) starEmit(_ *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (gostarlark.Value, error) {
	var line gostarlark.String
	if err := gostarlark.UnpackArgs("emit", args, kwargs, "line", &line); err != nil {
		return nil, err
	}
	return gostarlark.None, nil
}

// starRender implements ctx.render(template_str, vars).
// Signature: render(template_str, vars) -> str
// Renders {{VAR}} placeholders in template_str. Body deferred to M5.
func (c *CtxValue) starRender(_ *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (gostarlark.Value, error) {
	var templateStr gostarlark.String
	var vars *gostarlark.Dict
	if err := gostarlark.UnpackArgs("render", args, kwargs,
		"template_str", &templateStr,
		"vars", &vars,
	); err != nil {
		return nil, err
	}
	return gostarlark.String(""), nil
}

// starRenderFile implements ctx.render_file(src, vars).
// Signature: render_file(src, vars) -> str
// Reads src from ctx.component_dir and renders {{VAR}} placeholders. Body deferred to M5.
func (c *CtxValue) starRenderFile(_ *gostarlark.Thread, _ *gostarlark.Builtin, args gostarlark.Tuple, kwargs []gostarlark.Tuple) (gostarlark.Value, error) {
	var src gostarlark.String
	var vars *gostarlark.Dict
	if err := gostarlark.UnpackArgs("render_file", args, kwargs,
		"src", &src,
		"vars", &vars,
	); err != nil {
		return nil, err
	}
	return gostarlark.String(""), nil
}
