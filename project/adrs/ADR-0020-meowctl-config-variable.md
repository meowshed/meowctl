---
id: ADR-0020
artifact: adr
status: approved
revised: 2026-09-27
addresses: [REQ-1020]
supersedes: []
---

# 0020. Add $MEOWCTL_CONFIG ahead of the XDG config directory

## Decision

The config directory resolves to `$MEOWCTL_CONFIG` when set, then
`$XDG_CONFIG_HOME/meowctl`, then `~/.config/meowctl` (from docs/spec/common.md,
high). REQ-1020 states it, and it's a new variable `v0.1.0` doesn't consult
(from docs/design/0.2.0-decisions.md §3.1 and
crates/meowctl-common/src/paths.rs, high).

Once accepted, a user who never sets it sees `v0.1.0` behaviour exactly (from
docs/design/0.2.0-decisions.md §3.1, high).

## Why

The shell snippets run `meowctl hook shell` with no flags, so a user whose
configuration lives outside the XDG location can redirect that call only
through the environment, and `$XDG_CONFIG_HOME` would move every other tool's
configuration too; `$MEOWCTL_CONFIG` moves only meowctl (inferred from
crates/meowctl-cli/src/shell.rs:29,43,55,67, medium). The variable is additive,
so parity still holds for anyone who doesn't set it (from
docs/design/0.2.0-decisions.md §3.1, high).

The variable came from the compat corpus, which had to point two binaries at
the same configuration for every command it compared, where `--config` on
every invocation would have made the harness the thing being tested (from
docs/design/0.2.0-decisions.md §3.1, high). The corpus was deleted at the
cutover, and the shell-hook use is the reason it stays (from
https://github.com/meowshed/meowctl/pull/80, high).

## Alternatives

| Option | Better at | Why it lost |
| --- | --- | --- |
| Resolve the config directory exactly as `v0.1.0` does | No new surface, which is what parity exists to hold still | The flagless `meowctl hook shell` could reach a configuration outside the XDG location only by moving every tool's configuration with `$XDG_CONFIG_HOME` (inferred from crates/meowctl-cli/src/shell.rs, medium); the corpus would also have had to pass `--config` on every invocation (from docs/design/0.2.0-decisions.md §3.1, high) |

## What it costs

A new variable is new surface, and surface is what parity exists to hold still
(from docs/design/0.2.0-decisions.md §3.1, high).

## What would reverse it

- The shell snippets gain a way to pass `--config`. Removing it costs no
  released user until `v0.2.0` is tagged (from `git tag`, high).

## Consequences

- `--config` still overrides every source (from docs/spec/common.md, high).
- Users learn of the variable only from the changelog, which has to name it
  (from CHANGELOG.md and README.md, high).

## How I will know it was realised

1. `crates/meowctl-common/src/paths.rs` resolves `$MEOWCTL_CONFIG` first and
   holds REQ-1020 in its unit tests (from crates/meowctl-common/src/paths.rs,
   medium).

## What this does not settle

- The source recorded no alternative beyond the `v0.1.0` resolution (from
  docs/design/0.2.0-decisions.md §3.1, high).
