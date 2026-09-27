---
id: REQ-3471
artifact: requirement
topic: cli
class: non-functional
status: approved
revised: 2026-09-27
elaborates:
verification: static
---

# REQ-3471

A release MUST publish `checksums.sri`, one line per asset, each
`<sri>  <asset name>`.

A release with no checksums is a release that cannot be verified (from
docs/spec/cli.md, high). A tag matching `v*` runs
`.github/workflows/release.yml`, which builds four targets and publishes
`checksums.sri` (from CLAUDE.md, high). The alternative was to update from a
release with no checksums, unverified, which lost because a release nothing can
be verified against is not one that needs no verification, and that reading is
how the check gets skipped; the trade-off record names no condition that
reverses it (from docs/design/0.2.0-requirement-tradeoffs.md, high).

No test reads the release workflow; the evidence still missing is a test in
`crates/meowctl-release/tests/` asserting that `.github/workflows/release.yml`
uploads `checksums.sri` (from .github/workflows/release.yml, high).

Migrated from `R-CLI-071` in `docs/spec/cli.md`, its first obligation.
