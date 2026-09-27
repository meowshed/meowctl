---
id: REQ-1401
artifact: requirement
topic: fs
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1401

`FileSystem` MUST cover exactly the operations the rest of the workspace needs:
reading a file, writing a file, appending to a file, removing a file, copying a
file, creating and removing a symlink, reading a symlink's target, creating a
directory, listing a directory, renaming, and querying metadata including
whether a path exists and whether it is a symlink.

The alternative was a broader trait covering everything `std::fs` offers, which
lost because each method needs three implementations and an inverse story, so
unused breadth is cost with no user; revisit it if a component needs an
operation the set omits (from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-FS-001` in `docs/spec/fs.md`.
