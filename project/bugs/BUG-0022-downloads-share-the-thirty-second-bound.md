---
id: BUG-0022
artifact: bug
status: approved
severity: major
violates: REQ-2872
found: 2026-09-27
revised: 2026-09-27
issue:
---

<!-- Written to the writing standard meow-prose ships: lead with the answer,
give each rule its reason in the same sentence, and show the failing case. -->

# `ctx.download` and `self-update` share the 30-second bound meant for module requests

## Reproduction

Found by reading the code at revision `7e8cabe`, not by running it: the binary
builds one `RealHttp::new()` for the session and hands it to every caller (from
crates/meowctl-cli/src/run.rs:262, high), and `RealHttp::new` sets a 30-second
global timeout on the `ureq` agent (from crates/meowctl-net/src/real.rs:33-45,
high). To see it, call `ctx.download` on a file that takes more than 30 seconds
to arrive, or run `meowctl self-update` over a link slower than about 2.5
Mbit/s, since a release binary is 8 to 9 MB (from `gh release view v0.1.0`,
high).

## What the system does

The transfer fails with a timeout after 30 seconds, whatever its progress.

## What it should do, and why

`ctx.download` allows 5 minutes, as `v0.1.0` did (REQ-2872), and `self-update`
allows 5 minutes (REQ-3572), because a tool archive and a release binary are
larger than a module and `v0.1.0` gave neither the loaders' 30 seconds (from
`git show v0.1.0:internal/ctx/methods.go` line 840, high).

## Triage

It enters at `meowctl-cli`, which builds one client for three kinds of request.
Major, because a component whose download succeeded under `v0.1.0` fails under
`v0.2.0`, which breaks the compatibility CLAUDE.md calls the contract, and
`self-update` fails on a slow link.

## Closed by

Not yet: a test that downloads through a client slower than 30 seconds and
succeeds, beside the module-request test that still times out at 30.
