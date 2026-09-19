---
description: Check an implementation against its spec in both directions and report divergences
argument-hint: <spec file, requirement IDs, or a pull request number>
---

Verify the implementation against the specification: **$ARGUMENTS**

Load the `spec-driven` skill first.

This command reports. It does not fix anything, because deciding whether the
code or the spec is wrong is a judgement someone has to make deliberately.

## Do this

1. **Collect the requirement IDs from the spec.** Every `R-*` in scope,
   including the tombstones, so you can tell a withdrawn requirement from a
   typo.

2. **Collect the requirement IDs from the tree.** Search `crates/` and `tests/`
   for `R-*` references.

3. **Report the two differences.**
   - **Specified, not referenced by any test.** Either unimplemented or
     untested. Say which, by reading the code.
   - **Referenced, not in the spec.** Either a withdrawn requirement whose test
     survived, or a typo in the ID.

4. **Read each covered requirement against its test.** An ID in a doc comment
   proves somebody typed the ID, not that the test checks the obligation. For
   each one, say whether the test would actually fail if the requirement were
   violated. This is the slow part and it is the part that matters.

5. **Read the code for behaviour the spec does not describe.** Anything
   observable at a boundary that no requirement covers is either a missing
   requirement or scope that should not have shipped.

6. **Check the failure paths.** For every error the spec specifies, find the
   test that produces it.

7. **Check parity where the spec claims it.** For a requirement that cites
   `v0.1.0` behaviour, confirm the Go code still does what the requirement says
   it does. A parity requirement written from memory is worse than none,
   because it will be trusted.

## Output

Report as prose with a table, in this order:

- **Divergences**, worst first. For each: the requirement ID, what the spec
  demands, what the code does, and which one you think is wrong.
- **Uncovered requirements**: specified, no test.
- **Untethered tests**: reference an ID that no longer exists.
- **Unspecified behaviour**: observable, no requirement.
- **Weak tests**: the ID is cited but the test would pass even if the
  requirement were violated.
- **Parity claims that do not hold**: the requirement describes `v0.1.0` and
  `v0.1.0` does something else.

If everything matches, say so in one sentence and list the requirement IDs you
checked. Do not pad a clean result.
