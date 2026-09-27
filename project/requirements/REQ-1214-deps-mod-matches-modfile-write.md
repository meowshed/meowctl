---
id: REQ-1214
artifact: requirement
topic: config
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1214

A written `deps.mod` MUST reproduce `modfile.Write` byte for byte: the header
comment, `module()` across four lines with indented keyword arguments, each
`dep()` on one line, a blank line after the dependency block, and each
`replace()` on one line.

The header `v0.1.0` writes names the file `meowctl.mod`, which the rename to
`deps.mod` left behind (from docs/spec/config.md, high). The file is compared
byte for byte, and correcting the header is a `0.3.0` change that costs a corpus
rebaseline and buys a comment nobody reads (from docs/spec/config.md, high). The
alternative was correcting the header, which names the pre-rename `meowctl.mod`,
which lost because the file is compared byte for byte, and fixing a stale
comment costs a corpus rebaseline and buys nothing a user reads; revisit it if
the Go tree is deleted (from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CONFIG-014` in `docs/spec/config.md`, its first obligation.
