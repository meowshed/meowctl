---
id: REQ-2914
artifact: requirement
topic: ctx
class: functional
status: approved
revised: 2026-10-25
elaborates:
verification: behavioural
supersedes: [REQ-2812]
---

# REQ-2914

Every method MUST reject a relative path, except the `src` of `link_file` and
`render_file`, which REQ-2913 and REQ-2907 read relative to `component_dir`.

A hook has no defined working directory, so any other relative path has
nothing to resolve against. REQ-2812 rejected every relative path because it
took resolving against the component directory to differ from `v0.1.0`, but
`v0.1.0` resolves exactly these two sources that way, in `starLinkFile` and
`starRenderFile`.
