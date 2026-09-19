---
description: Decompose an approved spec into GitHub issues with explicit dependencies
argument-hint: <spec file or component, e.g. docs/spec/ops.md>
---

Decompose the approved specification into implementable issues: **$ARGUMENTS**

Load the `spec-driven` and `scm` skills first.

## Before planning

Confirm the spec is approved. Planning against a draft produces issues that get
rewritten. If you cannot tell whether it is approved, ask rather than assume.

Check the milestone order in `docs/design/0.2.0-rust-rewrite.md` §6 and §7. An
issue that depends on a milestone which has not landed is blocked, and saying
so now is cheaper than discovering it during implementation. M0 gates
everything: until the Starlark spike lands, no `meowctl-*` issue is ready.

## Do this

1. **Enumerate the requirements.** List every `R-*` ID in the spec. Each must
   end up in exactly one issue, or be explicitly deferred with a reason. A
   requirement that lands in no issue is the failure this step exists to catch.

2. **Cut by vertical slice, not by layer.** An issue that adds a struct to
   `meowctl-common` and nothing else cannot be reviewed on its merits and
   cannot be tested. Prefer issues that carry a behaviour from its boundary to
   its store and back, even when that touches three crates.

3. **Make dependencies explicit.** For each issue, name the issues that must
   close first and why. A dependency that is only about convenience is not a
   dependency; say so and let them proceed in parallel.

4. **Size for one PR each.** If an issue cannot be reviewed in one sitting,
   split it. If splitting it produces a piece with no observable behaviour, the
   cut is in the wrong place, so recut.

5. **Write the acceptance criteria from the spec.** Each issue states the
   requirement IDs it closes, verbatim. Do not paraphrase the requirement into
   the issue; cite it. The spec stays the single source.

6. **Name the parity check.** For an issue that touches a format, a builtin, or
   a command, say how parity against `v0.1.0` gets verified: which corpus
   config, which command, and what is compared.

7. **Identify what is not covered.** Anything the spec leaves open, anything
   that needs a decision before work starts, anything that depends on a spec
   that does not exist yet.

## Output

Present the plan for approval **before creating anything on GitHub**:

- the ordered issue list, each with its title, requirement IDs, dependencies,
  milestone, and whether it is small, medium, or large to review
- a dependency graph as text
- requirements not covered by any issue, with the reason
- what must be decided before the first issue can start

Create the issues only after the plan is approved, then report their numbers
and the order they should be taken in.
