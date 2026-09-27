---
id: REQ-1570
artifact: requirement
topic: fs
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1570

Creating a symlink on Windows MUST fail with an unsupported-operation error
naming the link path.

Windows distinguishes a file link from a directory link and needs a privilege
for either, so refusing is better than creating the wrong kind of link (from
https://github.com/meowshed/meowctl/pull/57 and
crates/meowctl-fs/src/real.rs:294-308, high). It is the sibling of REQ-1501,
which makes the executable bit a no-op on a platform without mode bits.

Added during onboarding from the research on decision 6; the shipped `RealFs`
returns `ErrorKind::Unsupported` inside an error that names the link path (from
crates/meowctl-fs/src/real.rs:294-308, high).
