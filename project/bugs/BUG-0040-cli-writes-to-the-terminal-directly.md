---
id: BUG-0040
artifact: bug
status: approved
severity: minor
violates: REQ-3510
found: 2026-09-27
revised: 2026-09-27
issue:
---

# `meowctl-cli` writes to stdout and stderr directly at three sites

`meowctl-cli` writes through `std::io::stdout()` or `std::io::stderr()` outside
`meowctl-tui`, which [REQ-3510] forbids, and the workspace lints miss it
because `print_stdout` and `print_stderr` catch only the `print!` family.

## Reproduction

At revision `7e8cabe`, from the repository root, run:

```bash
git grep -n 'std::io::stdout()\|std::io::stderr()' 7e8cabe -- \
  crates/meowctl-cli/src/run/commands.rs crates/meowctl-cli/src/signals.rs \
  crates/meowctl-cli/src/run.rs
```

The onboarding research found the sites by reading the code and ran no
program that shows a corrupted live region; the search above is the probe.

## What the system does

Three functions write to the terminal themselves:

- `print_to_stdout` at `crates/meowctl-cli/src/run/commands.rs:992` writes the
  verbatim `hook` output.
- `report` at `crates/meowctl-cli/src/run.rs:82` writes the error report.
- `restore_cursor` at `crates/meowctl-cli/src/signals.rs:77` writes the escape
  sequence that shows the cursor again.

Each site has a stated reason in its comment, and `mise run check` passes,
because no clippy setting covers a write through the `std::io` handles.

## What it should do, and why

[REQ-3510] says a command must not write to stdout or stderr directly, and that
`meowctl-tui` owns the exception, because a stray write corrupts the live
region. Either the three writes move behind `meowctl-tui`, or the requirement
names them as allowed exceptions and a clippy `disallowed-methods` entry for
`std::io::stdout` and `std::io::stderr` holds the rest.

## Triage

A requirement in force covers it, so the fix enters at implement, unless the
owner prefers to amend [REQ-3510] to allow the three sites. It is minor
because each site has a stated reason and the research observed no corrupted
output; it becomes major if one of the writes is shown to land while the live
region is drawing.

## Closed by

Open.
