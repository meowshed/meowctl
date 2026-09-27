---
id: BUG-0013
artifact: bug
status: approved
severity: minor
violates: REQ-2540
found: 2026-09-27
revised: 2026-09-27
issue:
---

# A GitHub ref that doesn't exist is reported as a bare HTTP 422

The error for a ref GitHub doesn't know gives the status code and not its
meaning, so a typo in the ref reads like an outage.

## Reproduction

The research ran the debug binary `target/debug/meowctl`, built from revision
`7e8cabe`, on macOS in a scratch directory, with `HOME`, `XDG_CACHE_HOME` and
`--config` pointed there. The research put this line in a component and ran
`meowctl apply`:

```python
load("github://meowshed/meowctl@no-such-ref-xyz//init.star", "Y")
```

## What the system does

It printed `meowctl: a: error: loading
github://meowshed/meowctl@no-such-ref-xyz//init.star: fetching the commit behind
the ref for github:meowshed/meowctl@no-such-ref-xyz:
https://api.github.com/repos/meowshed/meowctl/commits/no-such-ref-xyz answered
HTTP 422` and exited 3 (`crates/meowctl-module/src/github.rs:78-103`). A 422 (no
such ref), a 404 (no such repository, or a private one) and a 403 (rate limited)
read alike (medium). The exit code is BUG-0003. The test at `github.rs:151`
covers an unscripted URL, not a 422.

## What it should do, and why

[REQ-2540] says a ref that resolves to no commit fails with the module code,
naming the repository and the ref and saying that the ref wasn't found.

## Triage

A requirement covers it, so the fix enters at implement. It is minor because the
message already names the ref and the URL, so a careful reader can work it out.

## Closed by

Open.
