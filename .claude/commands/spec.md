---
description: Write or extend a component specification, then stop for review
argument-hint: <component or feature, e.g. "the rollback journal" or "deps.lock schema">
---

Write or extend the specification for: **$ARGUMENTS**

Load the `spec-driven` skill before doing anything else. It defines where specs
live, how requirement IDs are allocated, and what makes a statement normative.

## Do this

1. **Locate the authority.** Find what `docs/design/0.2.0-rust-rewrite.md`
   already decides about this component: its crate, its boundary, and the
   defect it exists to fix. A component spec elaborates those decisions and
   never contradicts them. Quote the relevant sections in your working notes.

2. **Locate the existing spec.** If `docs/spec/<component>.md` exists, you are
   extending it. Read all of it, and reuse its area prefix and allocation
   counter. If it does not, create it from the template in
   `docs/spec/README.md`.

3. **Read the Go implementation.** Parity is the contract, so for anything
   `v0.1.0` already does, the spec states what it does today and cites the file
   that does it. Specifying from memory is how a rewrite breaks a format
   nobody noticed it depended on. Where the spec deliberately changes
   behaviour, say so in its own sentence.

4. **Find the boundary.** Write down what is observable from outside this
   component: its API, the files it writes, the events it emits, the exit codes
   and errors it produces. Requirements constrain that boundary and nothing
   behind it.

5. **Write the requirements.** Testable, observable, singular, RFC 2119
   keywords. Specify the failure paths as precisely as the success path:
   a hook exiting non-zero mid-phase, a tarball failing its integrity check, a
   lock file from a newer schema version, a module bumped since the last apply.

6. **Write the open questions.** Anything you could not decide goes in an
   `## Open questions` section with the options and your recommendation. Do not
   resolve a genuine product decision by picking quietly.

## Then stop

**Do not implement. Do not create a branch. Do not open an issue.** This
command produces a document for a human to read and approve, and that approval
is the step the whole method exists to create.

Report:

- the file you wrote or extended
- the requirement IDs you allocated, as a range
- which design sections they elaborate
- any `v0.1.0` behaviour the spec deliberately changes
- the open questions, if any, with your recommendation on each
