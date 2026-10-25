---
id: BUG-0042
artifact: bug
status: approved
severity: major
enters: requirements
found: 2026-10-25
revised: 2026-10-25
---

# `link_file` refuses the relative `src` every corpus component passes

`ctx.link_file("config", dst)` fails with "the path is relative, and a hook
has no defined working directory", because `link_file` resolves `src` like any
other path. `v0.1.0` read `src` relative to the component directory, so every
dotmeow component that links a config file fails to install.

## Reproduction

meowctl 0.2.0 at commit `9feca9c`, aarch64-apple-darwin, with dotmeow 0.3.33
and meowctl-stdlib 0.2.17. In a config that includes `@dotmeow` and was last
applied with dotmeow 0.3.32, run:

```bash
meowctl apply --format plain
```

## What the system does

```text
error: ctx.link_file: config is not usable: the path is relative, and a hook has no defined working directory
  --> @dotmeow//components/git-config:10:5
   |
10 |     ctx.link_file("config", d + "/config")
```

dotmeow and meowctl-stdlib make 48 `ctx.link_file` calls with a relative
`src`, and none with an absolute one.

## What it should do, and why

`v0.1.0`'s `starLinkFile` in `internal/ctx/methods.go` says "src is resolved
relative to caps.ComponentDir", joins it onto `ComponentDir` and refuses an
absolute `src`. The parity principle in `CLAUDE.md` keeps that behaviour.
No requirement in force says so: REQ-2812 makes every method reject a relative
path, on the stated ground that resolving against the component directory
differs from `v0.1.0`, which is wrong for this argument.

## Triage

The requirement is wrong, so this enters at requirements: REQ-2913 states the
behaviour and REQ-2914 supersedes REQ-2812 with the exception. It is major
because it stops every dotmeow component that links a file, `git-config`,
`fish-config` and `tmux-config` among them, and the corpus can't work round it
without dropping `v0.1.0` compatibility.

## Closed by

Open.

## Tasks

- [ ] T-001 TSK-0040 read `link_file`'s `src` from `component_dir` in `crates/meowctl-ctx/src/value.rs`
