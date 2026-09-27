---
id: BUG-0005
artifact: bug
status: approved
severity: major
violates: REQ-3541
found: 2026-09-27
revised: 2026-09-27
issue:
---

# `init <repo-url>` writes the archive straight into the configuration root, so a failure partway leaves files behind

`init <repo-url>` extracts into the configuration root with no staging, so a
write failure partway, or an archive with no `init.star`, leaves a partial tree.

## Reproduction

The onboarding research inferred this from the code at revision `7e8cabe` and
didn't run it. The research's probe covered only the fetch failure, which wrote
nothing. At revision `7e8cabe`, run `meowctl --config <empty dir> init
https://github.com/<owner>/<repo>` against a repository whose default branch
`main` has no `init.star` at its root.

## What the system does

After a successful fetch, `archive::write_into` writes straight into the
configuration root (`crates/meowctl-module/src/archive.rs:115-128`). The check
for `init.star` runs only after the files are written, so they stay
(`crates/meowctl-cli/src/run/writing.rs:282-292`). With `--force` over an
existing configuration, the old `init.star` makes that check pass, so a
repository without one is merged into the old tree and then applied.

## What it should do, and why

[REQ-3541] says a failed `init <repo-url>` leaves the configuration directory as
it was. The fix stages the extraction beside the root and renames it into place,
as `Cache::ensure` does.

## Triage

A requirement covers it, so the fix enters at implement. It is major because the
`--force` case merges an unrelated repository into a working configuration and
runs it.

## Closed by

Open.
