---
id: REQ-2913
artifact: requirement
topic: ctx
class: functional
status: approved
revised: 2026-10-25
elaborates:
verification: behavioural
---

# REQ-2913

`link_file(src, dst)` MUST read `src` as a path relative to `component_dir`,
and refuse an absolute `src`.

A component ships the file it links beside its `.star` file, so `src` names a
file in the component and `dst` names a place on the machine. `v0.1.0` joins
`src` onto `ComponentDir` and refuses an absolute one in `starLinkFile`, and
the corpus depends on it: dotmeow and meowctl-stdlib make 48 `ctx.link_file`
calls with a relative `src`, such as `ctx.link_file("config", d + "/config")`
in `@dotmeow//components/git-config`.
