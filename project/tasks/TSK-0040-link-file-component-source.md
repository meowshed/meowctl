---
id: TSK-0040
artifact: task
status: approved
revised: 2026-10-25
bug: BUG-0042
closes: [REQ-2913, REQ-2914]
---

# Read `link_file`'s `src` from `component_dir`

`link_file` links `dst` to `src` joined onto `component_dir`, and refuses an
absolute `src`, so the corpus components install again.

## Acceptance criteria

1. Given a component directory, when a hook calls
   `ctx.link_file("config", "~/.config/tool/config")`, then the link at
   `~/.config/tool/config` points at `config` inside `component_dir`.
   Closed by: a test in `crates/meowctl-ctx/tests/surface.rs` naming REQ-2913.
2. Given an absolute `src`, when a hook calls `ctx.link_file`, then the call
   fails and links nothing. Closed by: a test in
   `crates/meowctl-ctx/tests/surface.rs` naming REQ-2913.
3. Given a relative `dst`, when a hook calls `ctx.link_file`, then the call
   still fails. Closed by: a test in `crates/meowctl-ctx/tests/surface.rs`
   naming REQ-2914.

## What to do

Change `link_file` in `crates/meowctl-ctx/src/value.rs` to build the target
from `component_dir` and `src`, as `render_file` does, and keep resolving `dst`
and `backup` through `path`. Bring SPC-2800 and the citations of REQ-2812 in
code and tests to REQ-2914.

## Depends on

None.

## Left alone

`copy_file` keeps resolving both paths, because `v0.1.0` expanded its `src`
like any other path. `v0.1.0`'s `backup` was a boolean where meowctl takes a
path, and that difference is out of scope here.

## Evidence

Not yet.
