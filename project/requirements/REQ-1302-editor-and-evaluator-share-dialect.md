---
id: REQ-1302
artifact: requirement
topic: config
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1302

The parser this component uses to edit a declaration and `meowctl-starlark` MUST
use the same dialect.

A second implementation of the language is a second set of semantics, and the
two drift (from docs/spec/config.md, high). This component parses the files only
for [REQ-1250] and [REQ-1251], which change one declaration without disturbing
the rest of the file, and parsing to locate a statement isn't evaluating it
(from docs/spec/config.md, high). A file one accepts and the other rejects would
mean `meowctl add` succeeding on a configuration that then fails to apply, so
the dialect is named in both crates and a test runs one file through both (from
docs/spec/config.md, high). The alternative was parsing `init.star` with a small
parser, which lost because that makes two grammars for one language, and the
second is always slightly wrong, and the record expects nothing to reverse it
(from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CONFIG-010` in `docs/spec/config.md`, its second obligation.
