---
id: REQ-1225
artifact: requirement
topic: config
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1225

`pkgs.lock` and `pkgs.local.lock` MUST record the packages declared by every
component whose `install` or `upgrade` ran, per manager.

`v0.1.0` writes the declared constraint as both `requested` and `installed`,
because nothing interrogates the manager afterwards (from docs/spec/config.md,
high). The file is a record of what is managed, not of what was resolved, and
stating otherwise invites somebody to trust it for reproducing an environment
(from docs/spec/config.md, high). The alternative was one `pkgs.lock`, which
lost because a machine-local component's packages would land in the shared,
committed file; revisit it if parity is abandoned (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CONFIG-025` in `docs/spec/config.md`, its first obligation.
