---
id: ADR-0034
artifact: adr
status: approved
revised: 2026-09-27
addresses: [REQ-3413]
supersedes: []
---

# 0034. Take the version from build metadata, not linker flags

## Decision

The version string comes from the build, `CARGO_PKG_VERSION` plus build
metadata, rather than from variables patched by `-ldflags` (from
docs/design/0.2.0-rust-rewrite.md §2 defect #8 and
docs/design/0.2.0-decisions.md §3.18, high). REQ-3413 states it (from
docs/spec/cli.md, high).

Once accepted, a release built outside the release task still reports its
version.

## Why

`mise.toml` computed three `-X` flags in a shell string, and a release built
outside that task silently reports `dev` (from docs/design/0.2.0-decisions.md
§3.18, high).

## Alternatives

| Option | Better at | Why it lost |
| --- | --- | --- |
| Reproduce the `-ldflags` variable patching | It's what the release workflow already knew how to set | A build outside that task reports `dev` (from docs/design/0.2.0-decisions.md §3.18, high) |

## What it costs

A build script to own; `crates/meowctl-cli/build.rs` computes the build date
(from crates/meowctl-cli/build.rs, medium).

## What would reverse it

- The source names no reversal condition (from docs/design/0.2.0-decisions.md
  §3.18, high).

## Consequences

- `MEOWCTL_BUILD_DATE` can override the date the build script records (from
  crates/meowctl-cli/build.rs, high).

## How I will know it was realised

1. `crates/meowctl-cli/src/version.rs` reads `env!("CARGO_PKG_VERSION")`, and
   `crates/meowctl-cli/tests/surface.rs` holds REQ-3413 (from those files,
   high).

## What this does not settle

- The source recorded no alternative beyond linker flags (from
  docs/design/0.2.0-decisions.md §3.18, high).
