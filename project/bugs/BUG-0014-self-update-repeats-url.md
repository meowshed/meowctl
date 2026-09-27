---
id: BUG-0014
artifact: bug
status: approved
severity: minor
found: 2026-09-27
revised: 2026-09-27
issue:
---

# `self-update` names the failing URL twice

Each network failure in `self-update` prefixes the URL to a `NetError` whose
text already carries it.

## Reproduction

The research ran the debug binary `target/debug/meowctl`, built from revision
`7e8cabe`, on macOS in a scratch directory, with `HOME`, `XDG_CACHE_HOME` and
`--config` pointed there. The research ran:

```bash
MEOWCTL_RELEASES=https://127.0.0.1:1/latest meowctl self-update
```

## What the system does

It printed `i checking for a newer release` and `meowctl:
https://127.0.0.1:1/latest: could not reach https://127.0.0.1:1/latest: io:
Connection refused (os error 61)`, and exited 4. The prefix is added at
`crates/meowctl-cli/src/update.rs:44`, `update.rs:68` and `update.rs:75`.

## What it should do, and why

The message should name the URL once, which the failure row in SPC-3400
describes. No requirement covers message wording, which is a gap for the
requirements step if the owner wants one.

## Triage

No requirement covers it, so it routes to the requirements step. It is minor
because the message is correct, only longer than it needs to be.

## Closed by

Open.
