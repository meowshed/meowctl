---
id: REQ-2940
artifact: requirement
topic: ctx
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2940

`ctx.download` MUST NOT write any bytes whose hash differs from the given
checksum.

A check after the write has already put untrusted bytes on disk, so the hash is
compared before anything is journalled or written.

Added during onboarding from the failure-path research; the shipped method holds
this, and a test checks it (from crates/meowctl-ctx/src/value.rs:664-704 and
crates/meowctl-ctx/tests/surface.rs:503, high).
