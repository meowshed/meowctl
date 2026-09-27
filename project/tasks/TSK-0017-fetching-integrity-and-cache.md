---
id: TSK-0017
artifact: task
status: approved
revised: 2026-09-27
epic: EPC-0001
closes: [REQ-1005, REQ-1405, REQ-1406, REQ-1501, REQ-1502, REQ-1503, REQ-1801, REQ-1802, REQ-1803, REQ-1804, REQ-1805, REQ-1806, REQ-1810, REQ-1811, REQ-1812, REQ-1813, REQ-1814, REQ-1900, REQ-1901, REQ-1902, REQ-1903, REQ-1904, REQ-1905, REQ-2220, REQ-2221, REQ-2222, REQ-2304, REQ-2410, REQ-2411, REQ-2412, REQ-2413, REQ-2430, REQ-2431, REQ-2432, REQ-2433, REQ-2434, REQ-2440, REQ-2441, REQ-2442, REQ-2443, REQ-2444, REQ-2445, REQ-2460, REQ-2463, REQ-2501, REQ-2502, REQ-2503, REQ-2506, REQ-2507, REQ-2508, REQ-2509, REQ-2510, REQ-2511, REQ-2512, REQ-2514]
issue:
---

# Fetching, integrity, and the cache

This task records issue 19 of the 0.2.0 execution plan, which the plan marks
done, and closes 55 requirements (from docs/design/0.2.0-execution-plan.md,
high).

## Acceptance criteria

1. Given a module whose tarball does not match its recorded hash, when it is
   fetched, then resolution fails with the module exit code and no files are
   extracted (inferred from docs/design/0.2.0-execution-plan.md, medium). Closed
   by: the tests that cite its requirements, in
   `crates/meowctl-common/src/id.rs`, `crates/meowctl-common/src/paths.rs`,
   `crates/meowctl-config/tests/parity.rs`,
   `crates/meowctl-ctx/tests/surface.rs`,
   `crates/meowctl-fs/tests/conformance.rs`, `crates/meowctl-module/src/mvs.rs`,
   `crates/meowctl-module/src/registry.rs`,
   `crates/meowctl-module/tests/fetching.rs`,
   `crates/meowctl-module/tests/registry_live.rs`,
   `crates/meowctl-module/tests/support/mod.rs`,
   `crates/meowctl-module/tests/syncing.rs`,
   `crates/meowctl-module/tests/urls.rs`, `crates/meowctl-net/src/error.rs`,
   `crates/meowctl-net/src/offline.rs`, `crates/meowctl-net/src/real.rs`,
   `crates/meowctl-net/src/script.rs`,
   `crates/meowctl-starlark/tests/evaluation.rs`, `tests/errors.rs` (from
   `rg REQ- crates tests`, medium).

## What to do

Fetch modules from the registry and GitHub, verify them, extract them and cache
them (from docs/design/0.2.0-execution-plan.md, medium). The issue is large on
purpose: integrity, extraction and caching are one path, and a slice that
fetches without verifying must not ship (from
docs/design/0.2.0-execution-plan.md, high).

It carries three things the plan did not first put here. `meowctl-net` is one:
the network is the third effect behind a trait and has to exist before anything
fetches (from docs/design/0.2.0-execution-plan.md, high). The composite loader
is the second, because the schemes it dispatches to exist only here (from
docs/design/0.2.0-execution-plan.md, high). The executable bit on extraction is
the third, because setting it outside `FileSystem` is an effect a dry run cannot
see (from docs/design/0.2.0-execution-plan.md, high).

## Depends on

- TSK-0003, issue 4 in the plan: a fetched module is checked against the lock
  issue 4 reads (inferred from docs/design/0.2.0-execution-plan.md, medium).
- TSK-0009, issue 10 in the plan: extraction and the cache write through
  `FileSystem` (from docs/design/0.2.0-execution-plan.md, high).

## Evidence

The plan marks issue 19 done (from docs/design/0.2.0-execution-plan.md, high).

Pull request #67, "Fetch a module, verify it, and know when the cache is lying",
carried the work, matched on "Issue 19" in its body:
https://github.com/meowshed/meowctl/pull/67 (from
https://github.com/meowshed/meowctl/pull/67, high).

## Left alone

Locking and syncing, which decide which version a module resolves to; they are
issue 21. This issue takes the version as given (from
https://github.com/meowshed/meowctl/pull/67, high).
