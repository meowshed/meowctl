---
name: spec-driven
description: How specification-driven development works in this repository - where specs live, how requirements are identified and traced to tests, when a spec must be written or amended, and what to do when code and spec disagree. Load before writing or changing a spec, before implementing against one, and whenever an implementation reveals the spec is wrong.
---

# Specification-driven development

## The rule

**No behaviour ships that a specification does not describe.** If you are about
to write code whose behaviour no normative statement in `docs/spec/` covers,
stop and write the statement first.

This is not process for its own sake. `v0.2.0` is a rewrite whose whole
justification is that the Go tree accumulated behaviour nobody decided on:
dry-run implemented as a branch that one code path forgot, a renderer passed
into the engine and then threaded back out as a callback, config edits done
with a regular expression that only matches canonical keyword order. Each of
those exists because someone wrote code, not because someone decided. The spec
is how that stops happening again.

The rewrite has a second source of truth that most projects lack: `v0.1.0`
still runs. When a spec and the Go binary disagree about a format or a builtin,
the Go binary is right and the spec is wrong, because parity is the contract.

## The two trees

| Tree | Holds | Changes when |
| --- | --- | --- |
| `docs/design/` | Architecture-level decisions for `v0.2.0`: the crate layout, the effect model, the event stream, the milestone order | A decision changes. Rare, and always a deliberate act |
| `docs/spec/` | Component-level normative behaviour, one file per crate or feature | Continuously, as components are specified ahead of being built |

`docs/design/0.2.0-rust-rewrite.md` answers why the system is shaped this way.
`docs/spec/` answers what this component must do. When they disagree, the
design document wins and the component spec is wrong.

## Requirements

A spec is prose with **normative statements** embedded in it. Each one carries
a stable identifier:

```markdown
**[R-OPS-014]** Applying an `Op` MUST append its inverse to the journal before
the effect reaches the filesystem. A crash between the two MUST leave a journal
that replays cleanly, even though the effect never landed.
```

- `R-<AREA>-<n>`, where `<AREA>` matches the spec file: `COMMON`, `CONFIG`,
  `FS`, `EXEC`, `OPS`, `STAR`, `MODULE`, `PM`, `CTX`, `ENGINE`, `TUI`, `CLI`.
- Numbers are **allocated, never reused**. A deleted requirement leaves a
  tombstone, so an old pull request or test referencing it still resolves:

  ```markdown
  **[R-OPS-009]** *Withdrawn in #142 - superseded by [R-OPS-014].*
  ```

- Numbers are not ordered by importance and do not renumber when the document
  is reorganised.

### Writing one well

A normative statement is testable, observable, and singular.

- **Testable.** Someone must be able to write a test that fails if it is
  violated. "The engine SHOULD be efficient" is not a requirement.
- **Observable.** It constrains behaviour visible at a boundary: a return
  value, a file on disk, an exit code, an emitted event, a rendered line. It
  does not constrain how a crate is laid out internally. `meowctl-engine` can
  be restructured freely as long as every `R-ENGINE-*` still holds.
- **Singular.** One obligation per statement. Two obligations means two
  requirements, because they will be tested separately and one may be withdrawn
  without the other.

Use MUST, MUST NOT, SHOULD, SHOULD NOT, and MAY in the RFC 2119 sense. `SHOULD`
means a deliberate, documented exception is permitted; if you cannot imagine
the exception, write `MUST`.

Specify the failure as precisely as the success. A spec that only describes the
happy path has not done its job, and the Go tree shows why: a module fetch that
fails integrity checking, a hook that exits non-zero halfway through a phase, a
lock file written by a newer schema version, a component whose module was
bumped since the last apply.

### Parity requirements

For anything `v0.1.0` already does, the requirement states the existing
behaviour and cites where it lives:

```markdown
**[R-CONFIG-021]** `deps.lock` MUST serialize module entries in the key order
`version`, `source`, `integrity`, `files`, `commit-sha`, `replaced`, `path`,
matching `internal/lock/write.go`. A file written by `v0.2.0` and a file written
by `v0.1.0` from the same resolution MUST be byte-identical.
```

Read the Go implementation before writing such a requirement. Specifying from
memory is how a rewrite breaks a format nobody noticed it depended on.

## Traceability

Every requirement has at least one test, and the test names the requirement:

```rust
/// [R-OPS-014] the inverse is journaled before the effect lands
#[test]
fn journal_records_inverse_before_write() { ... }
```

This is what makes `/verify` mechanical rather than a matter of opinion. The
requirement IDs in `docs/spec/` and the ones referenced under `crates/` and
`tests/` are two sets, and the interesting cases are the differences:

- **In the spec, not in any test.** Unimplemented or untested. Both are
  findings.
- **In a test, not in the spec.** The requirement was withdrawn and the test
  was not updated, or the ID is a typo.

## The loop

```text
  /spec --> review & approve --> /plan --> /implement --> /verify --> PR
              ^                               |
              +------- /amend-spec <----------+
                      (divergence found)
```

1. **`/spec`** writes or extends a component spec. It produces requirements and
   stops: no implementation, no branch, no code.
2. **Approval.** A human reads it. This is the step the whole method exists to
   create, and skipping it makes the rest ceremony.
3. **`/plan`** decomposes approved requirements into issues with explicit
   dependencies. Each issue cites the requirement IDs it closes.
4. **`/implement`** takes one issue, one branch, one pull request. The spec is
   the acceptance criteria.
5. **`/verify`** checks the implementation against the spec in both directions
   before you ask for review.

## Divergence

Implementation reveals that specs are wrong. That is expected and is the method
working, because the discovery happens against a written claim instead of
against a vague memory.

**When implementation contradicts the spec, stop.** Do not write code the spec
forbids and fix the wording afterwards. The spec stops being trustworthy the
first time that is allowed, and an untrustworthy spec is worse than none
because people still cite it.

The protocol:

1. Stop implementing. Leave the work in place, uncommitted or on its branch.
2. State the contradiction precisely: the requirement ID, what it demands, what
   the implementation found, and why the requirement cannot hold.
3. Run `/amend-spec`. It proposes the change, the blast radius, and the
   migration for anything already built.
4. Get the amendment approved as its own reviewable change.
5. Resume.

The step people skip is the third, and the failure is subtle: a requirement is
quietly reworded to match what was built, the reasoning behind the original is
lost, and six months later the same mistake is made again because the record of
why it was a mistake is gone.

One divergence is expected ahead of the rest. M0 is a spike against
`starlark-rust`, and its findings may contradict the design document rather
than a component spec. That is an amendment to `docs/design/`, which is a
bigger conversation than `/amend-spec` handles; say so and stop.

## What a spec is not

- **Not a design document.** `docs/design/` holds the reasoning and the
  rejected alternatives. A component spec states obligations and can be read by
  someone who does not care why.
- **Not documentation.** User-facing docs explain how to accomplish a task and
  may omit, simplify, and recommend. A spec is exhaustive about a boundary and
  omits nothing normative.
- **Not a test plan.** The spec says what must be true; the tests say how it is
  checked. One requirement may need six tests, and six requirements may share
  one.
- **Not a backlog.** Requirements describe the system as it is specified to be,
  in present tense. Sequencing lives in issues.

## Scope discipline

Specify what is being built now, to the depth needed to build it. A speculative
spec for a component nobody is implementing is worse than no spec: it will be
wrong by the time it matters, and it will be cited as authority meanwhile.

Equally, do not write a requirement you are unwilling to test. If a statement
cannot be checked, it is a design note. Put it in `docs/design/` and leave it
out of the normative tree.
