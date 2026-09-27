---
id: REQ-1500
artifact: requirement
topic: fs
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1500

The `FileSystem` trait MUST NOT expand `~`, join a relative path against a
working directory, or consult the environment.

A dry-run implementation that resolved paths differently from the real one would
report a plan that doesn't match what runs (from docs/spec/fs.md, high). The
alternative was resolving paths inside the trait, which lost because a dry run
that resolved differently from the real run would plan against different files;
revisit it if path resolution becomes uniform enough to sink into the trait
(from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-FS-002` in `docs/spec/fs.md`, its second obligation.
