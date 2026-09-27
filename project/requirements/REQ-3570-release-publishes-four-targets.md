---
id: REQ-3570
artifact: requirement
topic: cli
class: non-functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3570

Each release MUST publish a binary for `x86_64-unknown-linux-gnu`,
`aarch64-apple-darwin`, `x86_64-apple-darwin` and `x86_64-pc-windows-msvc`.

These are the platforms meowctl ships to, and no other requirement names them;
only per-platform behaviour such as REQ-1023, REQ-1501 and REQ-3473 does (from
.github/workflows/release.yml:23-30, high). `self-update` picks the asset for
the running platform, so a release missing one of these fails `self-update`
there; see REQ-3532 (from crates/meowctl-release, medium).

Added during onboarding from the research on decision 5; the shipped release
workflow builds the four targets, and CI tests only the first (from
.github/workflows/ci.yml:70, high).
