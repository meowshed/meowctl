---
description: Implement one issue against its spec, on a feature branch, ending in a pull request
argument-hint: <issue number or requirement IDs, e.g. #42 or R-OPS-014..018>
---

Implement: **$ARGUMENTS**

Load the `spec-driven`, `scm`, and `technical-english` skills first. Load
`rust` if the work touches Rust code.

## Before writing code

1. **Read the issue and every requirement it cites.** Quote the requirement
   text in your working notes. You are building to that text, not to your
   memory of a conversation.

2. **Refuse to start if the spec does not cover the work.** If the issue asks
   for behaviour that no requirement describes, stop and say so. Run `/spec`
   first. Implementing uncovered behaviour is the failure the whole method
   exists to prevent.

3. **Check the dependencies.** If the issue names blockers that have not
   closed, or sits in a milestone whose predecessor has not landed, say so and
   stop.

4. **Read what `v0.1.0` does.** For anything with an existing behaviour, open
   the Go code and read it before writing the Rust. The Go binary is the
   oracle, and a parity defect found by reading costs minutes where one found
   by the compat corpus costs a day.

5. **Branch off `rust-rewrite`.** Name it for the change, not for the issue
   number.

## While implementing

Build to the requirements and nothing more. Behaviour the spec does not ask for
does not go in, however obvious it seems; open an issue instead.

Write the tests against the requirement IDs as you go, not afterwards. Each
test names the requirement it checks in a doc comment. A requirement with no
test is not implemented, whatever the code does.

Cover the failure paths the spec specifies. A hook exiting non-zero mid-phase,
a tarball failing its integrity check, a lock file from a newer schema version,
a module bumped since the last apply: these are where the Go tree accumulated
its defects, and they are invisible to a coverage number.

Watch the three boundaries. `meowctl-common` imports no workspace crate,
`meowctl-tui` does not know the engine exists, and nothing below `meowctl-cli`
constructs an effect or reads a dry-run flag. A change that crosses one is a
defect however small it looks.

**When the implementation contradicts the spec, stop.** Do not write code the
spec forbids and reword the spec afterwards. Run `/amend-spec`, get the
amendment approved, then resume. This is the single rule that decides whether
the spec stays worth reading.

## Before opening the pull request

Run the full local gate and report what it actually said:

```bash
mise run fmt-check && mise run check && mise run test
```

If the change touches a format, a builtin, or a command, run the compat corpus
as well and report its result.

Then run `/verify` against the requirement IDs. Fix what it finds.

## Output

Open the pull request as described in the `scm` skill, then report:

- the branch and the pull request number
- the requirement IDs closed, and the test that covers each
- the gate results, with real numbers
- the parity result, if the change could affect it
- anything deferred, and the issue you opened for it
- any spec amendment this work required

Do not merge. Merging needs explicit approval.
