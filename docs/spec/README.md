# Component specifications

This tree holds component-level normative behaviour, one file per crate or
feature. `docs/design/0.2.0-rust-rewrite.md` decides why the system is shaped
the way it is; a spec here says what a component must do inside those
decisions. When the two disagree, the design document wins.

The method is in the `spec-driven` skill. Read it before writing a spec.

## Area prefixes

A requirement ID is `R-<AREA>-<n>`, where the area matches the file it lives
in. Numbers are allocated once per area and never reused.

| Area | File | Covers |
| --- | --- | --- |
| `COMMON` | `common.md` | Domain types, XDG paths, the error and exit-code taxonomy, the `Event` vocabulary |
| `CONFIG` | `config.md` | On-disk schemas, atomic writes, schema versioning, the Starlark editor |
| `FS` | `fs.md` | The `FileSystem` trait and its implementations |
| `EXEC` | `exec.md` | Process execution, env merging, terminal hand-off |
| `NET` | `net.md` | The `Http` trait and its implementations |
| `OPS` | `ops.md` | The `Op` enum, inverses, the write-ahead journal |
| `STAR` | `starlark.md` | Evaluator, builtins, accumulator, `load()` resolution, diagnostics |
| `MODULE` | `module.md` | Module graph, MVS, loaders, cache, integrity |
| `PM` | `pm.md` | Package-manager registry and dispatch |
| `CTX` | `ctx.md` | The Starlark `ctx` value and its methods |
| `ENGINE` | `engine.md` | Phases, the component graph, `Plan`, the runner, sentinel state |
| `TUI` | `tui.md` | Event sinks, theme, capability degradation, prompts |
| `CLI` | `cli.md` | The command surface, flags, exit-code mapping |

## Template

A spec is prose with normative statements embedded in it. Copy this shape:

```markdown
# <Component>

**Crate:** `meowctl-<name>`
**Design:** `docs/design/0.2.0-rust-rewrite.md` §<n>
**v0.1.0 equivalent:** `internal/<pkg>/`

## Scope

What this component owns, and what it explicitly does not. Two or three
paragraphs. A reader who stops here knows whether their question belongs in
this file.

## Boundary

What is observable from outside: the API, the files written, the events
emitted, the errors produced. Requirements constrain this and nothing behind
it.

## Behaviour

Prose, with requirements embedded where they belong:

**[R-<AREA>-001]** The loader MUST verify the SRI hash of every file it
extracts before any of them becomes visible to an evaluation.

## Failure paths

Prose and requirements covering what happens when things go wrong. Specify
these as precisely as the success path.

## Parity with v0.1.0

What the Go implementation does today, cited to the file that does it, and any
behaviour this spec deliberately changes.

## Open questions

Anything undecided, with the options and a recommendation. Delete the section
when it is empty.

## Withdrawn requirements

**[R-<AREA>-00n]** *Withdrawn in #<pr> - superseded by [R-<AREA>-0nn].*
```

## Writing rules, in short

- Testable, observable, singular. One obligation per statement.
- RFC 2119 keywords. `SHOULD` means a documented exception is permitted; if you
  cannot imagine the exception, write `MUST`.
- Specify the failure as precisely as the success.
- For anything `v0.1.0` already does, read the Go code and cite it. Parity is
  the contract, and a parity requirement written from memory will be trusted
  and will be wrong.
