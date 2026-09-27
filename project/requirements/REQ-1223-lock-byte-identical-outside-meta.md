---
id: REQ-1223
artifact: requirement
topic: config
class: non-functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1223

A lock file written by `v0.2.0` from the same resolution as `v0.1.0` MUST be
byte-identical outside the `meta` table, including key order and table order.

A lock `v0.1.0` wrote is checked in beside the resolver test that reproduces it
(from docs/spec/config.md, high). `meta` is excluded because it is the one table
the two binaries are meant to disagree about, and everything a run depends on,
versions, sources, hashes, commits and packages, is inside the comparison (from
docs/spec/config.md, high). The alternative was comparing the whole file, `meta`
included, which lost because `meta` is the one table the two binaries are meant
to disagree about; revisit it if `meta` stops being written (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Its evidence is the fixtures in `crates/meowctl-module/tests/fixtures/`, because
nothing can run `v0.1.0` since the Go tree and the compatibility corpus were
deleted (from CLAUDE.md parity_is_the_contract, high). The byte-identity rule
covers `deps.lock` and not the package locks, which `v0.1.0` writes with
`toml.NewEncoder` and nothing reads back, so only their structure has to
round-trip (from docs/spec/config.md, high).

Migrated from `R-CONFIG-023` in `docs/spec/config.md`.
