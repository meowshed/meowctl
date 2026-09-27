---
id: REQ-1108
artifact: requirement
topic: common
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: static
---

# REQ-1108

A crate below `meowctl-cli` MUST NOT choose the exit code for an error it
returns.

The alternative was letting crates choose codes, which lost because that is how
`v0.1.0` spreads `exitErrorf` through the CLI layer, and a crate that knows its
exit code knows about a process, and the record expects nothing to reverse it
(from docs/design/0.2.0-requirement-tradeoffs.md, high). What the rule forbids
is choosing a code where an error is raised, not holding the table:
`meowctl-common` holds the one table from `Severity` to a code, because it sits
below both the binary and the crates that classify, and only `meowctl-cli`
calls it (from crates/meowctl-common/src/error.rs:30-40,
crates/meowctl-cli/src/run.rs:36 and :71, high). The rewrite plan puts the
exit codes in `meowctl-common` on purpose (from
docs/design/0.2.0-rust-rewrite.md:323, high).

Onboarding narrowed the wording from forbidding such a crate to name an exit
code, which the shipped table breaks literally, to what the trade-off rows for
R-COMMON-031 and R-CLI-030 reject (from
docs/design/0.2.0-requirement-tradeoffs.md:46 and :406, high).

No check holds this yet, because crate dependencies let every crate reach
`std::process::exit` and `std::process::ExitCode`; the evidence still missing is
`exit = "deny"` in the workspace lints and a clippy `disallowed-types` entry for
`std::process::ExitCode`, allowed only in `meowctl-cli` (from Cargo.toml and
clippy.toml, high).

Migrated from `R-COMMON-031` in `docs/spec/common.md`, its third obligation.
