---
id: REQ-3172
artifact: requirement
topic: engine
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3172

A dry run of a phase set MUST run each planned hook against the dry-run effects
after it renders the plan.

Four decisions already assume it: ADR-0022 has dry-run reads see the writes the
run recorded, ADR-0023 has a dry run predict failure, ADR-0024 chooses the
dry-run executor per phase, and REQ-1611 runs read-only commands so an
`install_check` hook can ask what is installed (from
project/adrs/ADR-0022-dry-run-reads-see-writes.md,
project/adrs/ADR-0023-dry-run-predicts-failure.md and
project/requirements/REQ-1611-dry-run-runs-read-only-commands.md, high). The
changelog stated the same intent before onboarding corrected it to the shipped
behaviour: "A dry run is a `FileSystem` and an `Executor` that cannot write, so
it predicts exactly what a real run would do" (from
`git show 7e8cabe:CHANGELOG.md`, high). REQ-3422 only requires the plan to be
rendered, so the two hold together (from
project/requirements/REQ-3422-dry-run-renders-plan.md, high).

`v0.1.0` split the commands: `apply`, `add`, `remove` and `update` printed the
plan and stopped, while `upgrade` and `verify` ran every hook with a dry-run
`ctx` (from `git show v0.1.0:internal/cli/apply.go` lines 492-493 and
`v0.1.0:internal/cli/lifecycle.go` lines 849-940, high). Running hooks in every
dry run keeps `upgrade` and `verify` at parity and changes only the terminal
output of `apply`, which the parity principle exempts (from CLAUDE.md, high).

Added during onboarding to answer whether a dry run runs hooks; the shipped
binary runs none (BUG-0020).
