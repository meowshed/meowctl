---
id: REQ-1107
artifact: requirement
topic: common
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: static
---

# REQ-1107

The conversion from an error enum to an exit code MUST be the only place a code
is chosen.

The alternative was letting crates choose codes, which lost because that is how
`v0.1.0` spreads `exitErrorf` through the CLI layer, and a crate that knows its
exit code knows about a process, and the record expects nothing to reverse it
(from docs/design/0.2.0-requirement-tradeoffs.md, high).

No check holds this yet, because crate dependencies let every crate reach
`std::process::exit` and `std::process::ExitCode`; the evidence still missing is
`exit = "deny"` in the workspace lints and a clippy `disallowed-types` entry for
`std::process::ExitCode`, allowed only in `meowctl-cli` (from Cargo.toml and
clippy.toml, high).

Migrated from `R-COMMON-031` in `docs/spec/common.md`, its second obligation.
