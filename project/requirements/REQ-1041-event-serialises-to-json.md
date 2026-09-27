---
id: REQ-1041
artifact: requirement
topic: common
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1041

An `Event` MUST be serializable to JSON, because `JsonSink` emits one object per
event; see [REQ-3230].

`JsonSink` emits one object per event (from docs/spec/common.md, high). The
alternative was serialising only what `JsonSink` needs, which lost because two
representations of one event drift; revisit it if JSON output is dropped (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-COMMON-041` in `docs/spec/common.md`.
