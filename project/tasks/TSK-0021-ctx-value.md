---
id: TSK-0021
artifact: task
status: approved
revised: 2026-09-27
epic: EPC-0001
closes: [REQ-1044, REQ-2801, REQ-2802, REQ-2803, REQ-2810, REQ-2811, REQ-2812, REQ-2813, REQ-2814, REQ-2820, REQ-2821, REQ-2822, REQ-2823, REQ-2824, REQ-2825, REQ-2826, REQ-2827, REQ-2828, REQ-2830, REQ-2831, REQ-2832, REQ-2840, REQ-2841, REQ-2842, REQ-2843, REQ-2844, REQ-2900, REQ-2901, REQ-2902, REQ-2903, REQ-2904, REQ-2905, REQ-2906, REQ-2907, REQ-2908, REQ-2909, REQ-2910, REQ-2911, REQ-2912, REQ-3263, REQ-3321, REQ-3322]
issue:
---

# The ctx value

This task records issue 23 of the 0.2.0 execution plan, which the plan marks
done, and closes 42 requirements (from docs/design/0.2.0-execution-plan.md,
high).

## Acceptance criteria

1. Given a hook given `ctx`, when it calls a mutating method, then the effect
   goes through an `Op` and the traits, never around them (inferred from
   docs/design/0.2.0-execution-plan.md, medium). Closed by: the tests that cite
   its requirements, in `crates/meowctl-common/src/event.rs`,
   `crates/meowctl-ctx/src/template.rs`, `crates/meowctl-ctx/tests/surface.rs`,
   `crates/meowctl-engine/tests/support/mod.rs`,
   `crates/meowctl-fs/tests/conformance.rs`,
   `crates/meowctl-tui/src/interaction.rs`, `tests/hook.rs` (from
   `rg REQ- crates tests`, medium).

## What to do

Add the `ctx` value with its twenty-four methods (from
docs/design/0.2.0-execution-plan.md, high). The issue is large on purpose: two
requirements are properties of the whole surface, and a review that sees half of
it cannot check that no method reached past the traits (from
docs/design/0.2.0-execution-plan.md, high).

It also delivers the free-text half of the `Interaction` trait, because
`ctx.prompt` returns a string; issue 27 delivered the confirmation (from
docs/design/0.2.0-execution-plan.md, high).

## Depends on

- TSK-0010, issue 11 in the plan: `ctx.run` and its relatives run through
  `Executor` (inferred from docs/design/0.2.0-execution-plan.md, medium).
- TSK-0011, issue 12 in the plan: every mutating method goes through an `Op`
  (inferred from docs/design/0.2.0-execution-plan.md, medium).
- TSK-0020, issue 22 in the plan: `ctx` dispatches package declarations to the
  registry (inferred from docs/design/0.2.0-execution-plan.md, medium).

## Evidence

The plan marks issue 23 done (from docs/design/0.2.0-execution-plan.md, high).

Pull request #70, "Give a hook the surface it is allowed, and nothing more",
carried the work, matched on "Issue 23" in its body:
https://github.com/meowshed/meowctl/pull/70 (from
https://github.com/meowshed/meowctl/pull/70, high).

## Left alone

Who builds the `ctx` and calls the hook. That is the engine, issues 28 to 31
(from https://github.com/meowshed/meowctl/pull/70, high).
