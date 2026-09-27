---
id: onboarding
artifact: onboarding
status: approved
revised: 2026-09-27
---

<!-- Written to the writing standard meow-prose ships: lead with what was found,
give each figure its source, and state a gap as plainly as a finding. -->

# Onboarding meowctl

meowctl kept a specification-driven record of its own: 337 requirements in 13
area specifications under `docs/spec/`, a trade-off row for each of them, the
argued decisions and the execution plan under `docs/design/`. Onboarding
migrated all of it into `project/`, which `.meowpaw/profile.toml` now declares
as the record's root, and removed the old documents, so `project/` is the single
source of truth. The removed files stay readable at revision `7e8cabe`, which is
where every `(from docs/...)` citation in the record points.

The record now holds a vision, 13 specifications, 556 requirements (554 approved
and 2 withdrawn), 50 decisions, one epic with 38 tasks that records the 0.2.0
work, and 23 defects (from `meow-method count`, high). Researching the questions
the first pass raised found 23 defects in the shipped code, three of them
serious; the Gaps section answers every question and names the record each
answer changed.

## Verbs

`meow-verbs status` resolves four of the five verbs from
`.meowpaw/profile.toml`:

| Verb | State | Command |
| --- | --- | --- |
| `fmt` | resolved | `mise run fmt-check` |
| `lint` | resolved | `mise run check` |
| `typecheck` | unresolved | none declared |
| `test` | resolved | `mise run test` |
| `build` | resolved | `mise run build` |

`typecheck` is unresolved because the repository has no type-check task for the
host: `mise run check` runs clippy, which type-checks as it lints, and
`mise run check-windows` checks only the Windows target (from mise.toml, high).
The gate `mise run all` also runs `doc`, `deny`, `unused-deps` and `lint-md`,
which map to no verb (from mise.toml, high).

## Conventions

Each convention below is what the repository does, counted on 2026-09-27.

- Commit subjects: 124 of 124 follow Conventional Commits, with types `feat` 62,
  `fix` 29, `test` 11, `chore` 8, `docs` 6, `refactor` 5, and `revert`, `ci` and
  `build` once each (from `git log`, high). The profile adopts these nine and
  `spec`, which the `scm` skill names.
- Subject length: 120 of 124 subjects are under 72 characters, three are exactly
  72 and one is 74 (from `git log`, high). The profile sets 72, the limit the
  `scm` skill states.
- Trailers: 124 of 124 commits carry `Signed-off-by` (from `git log`, high), and
  the profile requires it.
- Signatures: 124 of 124 commits are signed, 101 with the owner's SSH key and 23
  squash merges with GitHub's key (from `git log --format=%G?`, high). The
  profile requires signatures on push; `main` has no protection on GitHub (from
  `gh api`, high).
- Merges: all 85 pull requests in the forge history were merged, and the `scm`
  skill asks for a squash merge (from the forge history, high).
- Spelling: prose uses `behaviour` 65 times and `behavior` never (from `rg`,
  high). The profile declares `[prose] language = "en-GB"`, the default of the
  `meow-prose` standard, which has the same author as meowctl (from its SPDX
  headers, high), and CLAUDE.md now names that standard in place of the removed
  `technical-english` skill, which asked for American spelling.

## Documents

| Document | Outcome | Where, or why |
| --- | --- | --- |
| `README.md` | cited | The user-facing introduction; `vision` cites it, and its documentation section now points at `project/` |
| `CHANGELOG.md` | cited | Release notes a user reads; two false claims corrected (see the decisions gaps) |
| `LICENSE` | cited | The licence text; outside the record's scope |
| `CLAUDE.md` | cited | The constitution; its principles now point at `project/` and the method |
| `docs/README.md` | discarded | An index of the old documents; `project/README.md` and the per-kind indexes list the record; removed |
| `docs/spec/README.md` | discarded | Its numbering and template gave way to the REQ blocks and the method's `spec` template; removed |
| `docs/spec/common.md` | migrated | SPC-1000 and REQ-1001 to REQ-1110; removed |
| `docs/spec/config.md` | migrated | SPC-1200 and REQ-1201 to REQ-1316; removed |
| `docs/spec/fs.md` | migrated | SPC-1400 and REQ-1401 to REQ-1508; removed |
| `docs/spec/exec.md` | migrated | SPC-1600 and REQ-1601 to REQ-1704; removed |
| `docs/spec/net.md` | migrated | SPC-1800 and REQ-1801 to REQ-1905; removed |
| `docs/spec/ops.md` | migrated | SPC-2000 and REQ-2001 to REQ-2120; removed |
| `docs/spec/starlark.md` | migrated | SPC-2200 and REQ-2201 to REQ-2307, without the M0 spike history; removed |
| `docs/spec/module.md` | migrated | SPC-2400 and REQ-2401 to REQ-2514; removed |
| `docs/spec/pm.md` | migrated | SPC-2600 and REQ-2601 to REQ-2704; removed |
| `docs/spec/ctx.md` | migrated | SPC-2800 and REQ-2801 to REQ-2912; removed |
| `docs/spec/engine.md` | migrated | SPC-3000 and REQ-3001 to REQ-3121; removed |
| `docs/spec/tui.md` | migrated | SPC-3200 and REQ-3201 to REQ-3323; removed |
| `docs/spec/cli.md` | migrated | SPC-3400 and REQ-3401 to REQ-3532, including `meowctl-release`; removed |
| `docs/design/0.2.0-requirement-tradeoffs.md` | migrated | REQ-1001 to REQ-3532, as the reason paragraph of each; removed |
| `docs/design/0.2.0-decisions.md` | migrated | ADR-0019 to ADR-0040, and merged into ADR-0004, 0005, 0008, 0009, 0011 to 0013 and 0016 to 0018; removed |
| `docs/design/0.2.0-rust-rewrite.md` | migrated | `vision` and ADR-0001 to ADR-0018, without the milestone history; removed |
| `docs/design/0.2.0-execution-plan.md` | migrated | EPC-0001 and TSK-0001 to TSK-0038; removed |
| `.claude/commands/spec.md` | discarded | The method's spec step does its work; removed |
| `.claude/commands/plan.md` | discarded | The method's epic step does its work; removed |
| `.claude/commands/implement.md` | discarded | The method's implement step does its work; removed |
| `.claude/commands/verify.md` | discarded | The method's verify step does its work; removed |
| `.claude/commands/review.md` | discarded | The method's review step does its work; removed |
| `.claude/commands/amend-spec.md` | discarded | A draft that supersedes the old record does its work; removed |
| `.claude/skills/spec-driven/SKILL.md` | discarded | The `meow-method:method` skill holds its method; removed |
| `.claude/skills/technical-english/SKILL.md` | discarded | The `meow-prose:writing` skill holds the writing standard; removed |
| `.claude/skills/scm/SKILL.md` | cited | Branch names, the pull request body and merge rules; its Go-tree and `rust.yml` passages are corrected |
| `.claude/skills/rust/SKILL.md` | cited | Code conventions; it now cites the decisions and names the dependencies that shipped |

A requirement `R-<AREA>-<n>` became `REQ-<base + n>`, with bases common 1000,
config 1200, fs 1400, exec 1600, net 1800, ops 2000, starlark 2200, module 2400,
pm 2600, ctx 2800, engine 3000, tui 3200 and cli 3400. A requirement that
carried more than one keyword was split, and each further obligation took a
number from `base + 100` upward; the last line of every migrated requirement
names its source `R-*` identifier and which obligation it was. The 1,112
citations in `crates/`, `tests/` and `src/` now name REQ identifiers, and a test
that held a split requirement cites every part of it.

The `v0.1.0` decisions ADR-006 to ADR-020 lived outside this repository, and
only issue and pull request text records them: issue #17 cites ADR-006, and pull
requests #24, #26, #35 and #48 cite ADR-011 to ADR-020 (from
https://github.com/meowshed/meowctl/issues/17, high). No record links them in
`supersedes`, because that key names records under `project/`; ADR-0011 lists
ADR-006 among its alternatives.

## Gaps

Every question the first pass raised is answered below from the code, the forge
history and probes of the debug binary, and each answer names the record it
changed. The research notes behind them aren't kept, because each finding now
lives in the record it changed.

The record and the process:

1. The execution plan became EPC-0001, realising ADR-0001, with 38 done tasks
   that each name the pull request that shipped it; nine plan issues that close
   no requirement are listed under its Not covered section.
2. The code's citations were rewritten to REQ identifiers in this change.
3. Coverage: every one of the 526 migrated requirements lands in exactly one
   task. The 30 requirements the research added, REQ-2872, REQ-3172, REQ-3572,
   REQ-1570, REQ-1770, REQ-2140, REQ-2141, REQ-2190, REQ-2340 to REQ-2343,
   REQ-2370, REQ-2540, REQ-2940, REQ-2941, REQ-3140 to REQ-3142, REQ-3340 to
   REQ-3342, REQ-3370, REQ-3540 to REQ-3543, REQ-3570, REQ-3571 and REQ-3590,
   land in no task yet, because most describe behaviour the defects below show
   isn't built.
4. The method writes one index block per file, and specifications, epics and
   defects share `project/README.md`, so its block holds the specifications and
   the epic and defect tables are written by hand. `meow-method check` reports
   the other two blocks as out of date; that is a defect in `meow-method` to
   raise upstream.
5. The research index finding stands: writing an empty `RES-0001-synthesis.md`
   would invent research nobody did.
6. CLAUDE.md's principles now point at `project/` and the method's steps.

Contradictions between a document and the code:

1. The code was right about exit codes: `meowctl-common` holds the one table and
   only `meowctl-cli` calls it, so REQ-1108 now forbids a lower crate from
   choosing an exit code.
2. SPC-2800 describes the shipped `alloc_simple` over a `Send + Sync` core, and
   SPC-2200 narrows the M0 finding to values that aren't `Send + Sync`.
3. `expect_used = "warn"` is intended; the `clippy.toml` comment is corrected.
4. REQ-3210's reversal condition is restated for the four sinks that exist.

Requirements whose reason or verification was thin:

1. Seven requirements got traced reasons; REQ-3518 kept only the command half,
   and the flag half became REQ-3590.
2. Three reasons in the starlark and module areas were replaced; the rest stand
   on their trade-off rows.
3. REQ-1107 and REQ-1108 are `static`, and each names the lint that would hold
   it. REQ-3505 and REQ-3506 are reworded so the engine's per-phase choice of an
   executor satisfies them.
4. REQ-1805, REQ-2252, REQ-3453 and REQ-3474 each record the test still missing.
5. A unit test inside a crate counts when it calls the code carrying the
   obligation.
6. REQ-3119 is withdrawn in favour of REQ-3051 and REQ-3117, and REQ-1227 in
   favour of REQ-1223.
7. 25 requirements moved from `non-functional` to `functional`, and REQ-1107,
   REQ-1108, REQ-1703 and REQ-2512 changed their verification kind.

Failure paths: all 22 cases are now in the Failure paths section of the
specification that owns them, with a draft requirement where a user relies on
the behaviour. Where the shipped code differs, a defect records it:

1. BUG-0001, critical: the rollback journal is never truncated, because
   `Journal::truncate` is called only from tests, so a failed run's rollback
   undoes files an earlier successful run installed.
2. BUG-0002, critical: `/etc/os-release` is never read, so on Linux every
   `distros`-guarded package manager in the standard library is dropped.
3. BUG-0003, major: the CLI ignores the severity lower crates compute, so a
   module that won't fetch exits 3 where REQ-2253 requires 4.
4. BUG-0004 to BUG-0019 and BUG-0040, each minor or major: the legacy-layout
   message, unstaged `init`, `self-update` cleanup, unreadable Starlark files,
   narrow terminals, prompts with stderr closed, the `.hook-error` flag, GitHub
   refs, `platform()`, `query_pm` and `interrogate` messages, the journal's
   writability and torn records, and direct terminal writes in `meowctl-cli`.
5. BUG-0020: `apply --dry-run` runs no hook, which leaves REQ-3034, REQ-1611 and
   REQ-2621 unreachable from the binary; the changelog no longer claims a dry
   run predicts exactly what a real run does.
6. BUG-0021: the live sink counts characters where it needs columns.

The vision:

1. Nothing in the repository, its forge history or the neighbouring `dotfiles`
   and `dotmeow` configurations names a competing tool, so `vision` keeps
   `v0.1.0` as the only recorded alternative.
2. The quality goals keep their order, with compatibility first, because
   CLAUDE.md calls a break a regression.
3. Sentinel records live in the `completed_components` table of `state.toml`;
   SPC-3000 now says so at high confidence.

The decisions:

1. The engine's boolean is the allowed choice of an executor per phase, so
   R-ENGINE-034 wins; ADR-0004 records it.
2. ADR-0001, ADR-0007 and ADR-0010 now name what shipped: no `miette`, the
   stages `Discovered`, `Graph`, `Plan` and `Report`, and `terminal_size` with
   hand-written escapes.
3. The changelog now announces the duplicate-handler error and the other
   deliberate refusals, and no longer claims every `v0.1.0` configuration
   applies unchanged.
4. ADR-0016 records `confirm` and `ask`, with `select` among its alternatives.
5. Linux-only CI is ADR-0048, addressing REQ-3570; the hand-written `rfc3339`,
   `cargo-mutants` and the single branch stay working practice, because none
   changes what a requirement fixes.
6. REQ-1770 and REQ-1570 record #58 and #57; #67 and #59 map to REQ-2463 and to
   REQ-2002 and REQ-2102.
7. The trade-off table marks 42 rows decided; ADR-0049 and ADR-0050 cover the
   two that break `v0.1.0` behaviour, and ADR-0004 and ADR-0019 absorb two more.
8. ADR-0008 addresses REQ-3571 and REQ-1801; ADR-0029 addresses REQ-2251 and
   REQ-2370.
9. ADR-0003, ADR-0006 and ADR-0039 now state their costs and reversal
   conditions.
10. `$MEOWCTL_CONFIG` stays, for the shell hook that takes no `--config`;
    ADR-0020 says so and the changelog lists it under Added.
11. ADR-0011 lists `v0.1.0`'s ADR-006 among its alternatives.

The two questions the second pass left for the owner are answered too:

1. A dry run runs hooks against the dry-run effects, now REQ-3172. ADR-0022,
   ADR-0023, ADR-0024 and REQ-1611 already assume it, the changelog promised it,
   and `v0.1.0` ran hooks in `upgrade --dry-run` and `verify --dry-run` (from
   `git show v0.1.0:internal/cli/lifecycle.go`, high). BUG-0020 is now major and
   violates REQ-3172.
2. A download gets 5 minutes: REQ-2872 for `ctx.download`, the bound `v0.1.0`
   gave it, and REQ-3572 for `self-update`, which had none under `v0.1.0`.
   REQ-1802's 30 seconds now covers module and registry requests only, and
   BUG-0022 records that the shipped client applies 30 seconds to all three.

The owner approved every record on 2026-09-27. `meow-method check coverage` now
counts 524 of 554 requirements in force in a task: the 526 migrated ones less
the two withdrawn, with the 30 added during research waiting for the epics that
fix their defects.

## Adoption

The owner approved this report and every record on 2026-09-27, so `project/` is
the record. What comes next:

1. Fix BUG-0001, BUG-0002 and BUG-0003 first, each through an epic of its own,
   because the first loses user files and the second breaks every Linux machine.
2. Give each of the 30 requirements the research added a task, through the epics
   that fix the defects they describe.
3. Raise the index-block and research-index findings against `meow-method`.
