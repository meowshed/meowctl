---
id: ADR-0023
artifact: adr
status: approved
revised: 2026-09-27
addresses: [REQ-1433]
supersedes: []
---

# 0023. Fail a dry run where the real run would fail for a reason it can see

## Decision

`DryRunFs` fails where `RealFs` would fail for a visible reason: a write into a
directory that doesn't exist and wasn't recorded as created, and a symlink over
a regular file with no backup requested (from docs/spec/fs.md and
docs/design/0.2.0-decisions.md §3.4, high). REQ-1433 states it (from
docs/design/0.2.0-decisions.md §3.4, high).

Once accepted, a dry run doesn't report success for a run that will fail on
those causes. Causes it can't see, such as permissions, stay unpredicted
(inferred from docs/design/0.2.0-decisions.md §3.4, low).

## Why

Not predicting failure guarantees a silent false success, and a dry run that
succeeds where the real run fails is worse than none (from
docs/design/0.2.0-decisions.md §3.4 and
https://github.com/meowshed/meowctl/pull/57, high).

## Alternatives

| Option | Better at | Why it lost |
| --- | --- | --- |
| Let a dry run succeed at everything | Never blocks a run that would have worked | It silently guarantees the opposite error (from docs/design/0.2.0-decisions.md §3.4, high) |

## What it costs

Predicting failure risks a false negative that blocks a run that would have
worked (from docs/design/0.2.0-decisions.md §3.4, high).

## What would reverse it

- The source names no reversal condition (from docs/design/0.2.0-decisions.md
  §3.4, high).

## Consequences

- Prediction is limited to causes the implementation can see (from
  docs/design/0.2.0-decisions.md §3.4, high).

## How I will know it was realised

1. `crates/meowctl-fs/tests/conformance.rs` and
   `crates/meowctl-ctx/tests/surface.rs` hold REQ-1433 (from those files, high).

## What this does not settle

- Which further failure causes a dry run should predict.
- The source recorded no alternative beyond always succeeding (from
  docs/design/0.2.0-decisions.md §3.4, high).
