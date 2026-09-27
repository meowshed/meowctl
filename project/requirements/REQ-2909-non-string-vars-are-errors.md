---
id: REQ-2909
artifact: requirement
topic: ctx
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2909

Anything other than a mapping of strings to strings passed as `vars` MUST be an
error rather than being formatted.

Neither writes anything: a component that wants the rendered text on disk passes
it to `write_file` (from docs/spec/ctx.md, high). `renderTemplate` refuses a
non-string value, and a component that passed an integer would otherwise get a
substitution that depends on Starlark's formatting (from docs/spec/ctx.md,
high). The alternatives were a full template language, or a `render_file` that
writes (from docs/design/0.2.0-requirement-tradeoffs.md, high). Substitution is
what components use, writing is the job of `write_file`, and `v0.1.0` returns a
string that the standard library passes on (from
docs/design/0.2.0-requirement-tradeoffs.md, high). Revisit it if a component
needs conditionals in a template (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CTX-027` in `docs/spec/ctx.md`, its fourth obligation.
