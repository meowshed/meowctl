---
id: REQ-1801
artifact: requirement
topic: net
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1801

`Http` MUST expose one method: fetch a URL and return its body as bytes.

A component that wants a document parses the bytes itself, because a trait that
decoded JSON would need a second method for TOML and a third for a tarball (from
docs/spec/net.md, high). The alternative was a method each for JSON, TOML and a
tarball, which lost because three decoders behind a trait means three test
doubles and a trait that grows with every format (from
docs/design/0.2.0-requirement-tradeoffs.md, high). Revisit it if a caller needs
streaming in place of bytes (from docs/design/0.2.0-requirement-tradeoffs.md,
high).

Migrated from `R-NET-001` in `docs/spec/net.md`.
