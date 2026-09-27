---
id: REQ-3510
artifact: requirement
topic: cli
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: static
---

# REQ-3510

A command MUST NOT write to stdout or stderr directly; the workspace lint denies
it outside this crate, and `meowctl-tui` owns the exception.

The workspace lints `print_stdout` and `print_stderr` hold this, denied
everywhere outside `meowctl-tui` (from CLAUDE.md, high). The alternative was to
print directly where convenient, which lost because a stray print corrupts the
live region, and the trade-off record names no condition that reverses it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

The lints catch only the `print!` family, and `meowctl-cli` writes through
`std::io::stdout()` or `std::io::stderr()` at three sites (BUG-0040); the
evidence still missing is a clippy `disallowed-methods` entry for both, allowed
at each site that states its reason (from Cargo.toml and clippy.toml, high).

Migrated from `R-CLI-020` in `docs/spec/cli.md`, its second obligation.
