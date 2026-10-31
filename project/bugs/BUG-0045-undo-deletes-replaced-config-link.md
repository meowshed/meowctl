---
id: BUG-0045
artifact: bug
status: approved
severity: critical
enters: requirements
found: 2026-10-31
revised: 2026-10-31
---

# Undoing a `link_file` that replaced a symlink deletes the link

When a run fails, the replay of a `link_file` record whose link replaced an
existing symlink removes the link instead of putting the old one back, so the
user loses the config file the link served.

## Reproduction

meowctl 0.2.0 at commit `fb70085`, aarch64-apple-darwin. A configuration last
applied with dotmeow 0.3.32, so `~/.config/fish/config.fish` and the other
config files are symlinks into that module's cache, is moved to dotmeow 0.3.34
and applied from a session with no terminal:

```bash
meowctl apply --format plain
```

## What the system does

`fish-config`, `ssh-config` and `claude-code-config` re-point their links at
the 0.3.34 cache, then `claude-code-config` stops at a `ctx.prompt` that has no
answer. The run is undone, and the 24 journal records, each
`{"dst": ..., "was_backed_up": false}`, replay as removals. `~/.ssh/config`,
`~/.claude/settings.json` and every file under `~/.config/fish` are gone after
the run.

## What it should do, and why

`Op::inverse` computes `Op::Symlink` with the prior target for exactly this
case, as REQ-1506 and REQ-2003 require, and `Symlink`'s own inverse restores a
prior link (REQ-2014). The journal record for `link_file` has no field for the
prior target, so the replay can't. REQ-2015 says the inverse removes the link,
which is the behaviour that deleted the files.

## Triage

The requirement is wrong, so this enters at requirements: REQ-2033 supersedes
REQ-2015, and ADR-0052 records the break from `v0.1.0`. It is critical because
it deletes user configuration with no backup, and any failure after a module
upgrade triggers it.

## Closed by

`a_replaced_link_comes_back_on_replay` in `crates/meowctl-ops/tests/journal.rs`
holds the replay as a regression check.

## Tasks

- [x] T-001 TSK-0044 journal and replay the prior target of a replaced link, in
  `crates/meowctl-ops/src/journal.rs` (the replay tests pass, and every stage
  exits 0)
