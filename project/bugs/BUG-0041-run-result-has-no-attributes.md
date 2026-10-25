---
id: BUG-0041
artifact: bug
status: approved
severity: major
violates: REQ-2820
enters: implement
found: 2026-10-25
revised: 2026-10-25
---

# `ctx.run` returns a dict, so a component reading `result.stdout` fails

`run_result` at `crates/meowctl-ctx/src/value.rs:266` builds a Starlark `dict`,
and a dict carries no attributes, so every component that reads `r.stdout`,
`r.stderr` or `r.exit_code` stops with an error. `v0.1.0` returned a struct,
and the `meowctl-stdlib` and `dotmeow` corpus reads the fields as attributes.

## Reproduction

meowctl 0.2.0 at commit `71f4762`, aarch64-apple-darwin, with dotmeow 0.3.32
and meowctl-stdlib 0.2.17, on a machine where `capslock-control` is installed.
From a config that includes `@dotmeow`, run:

```bash
meowctl verify --format plain
```

The test `run_returns_three_fields_and_does_not_fail_on_a_non_zero_exit` in
`crates/meowctl-ctx/tests/surface.rs` reads the result by index, so it passes
on the defect and shows nothing.

## What the system does

`verify` fails at the first component that reads an attribute:

```text
error: Object of type `dict` has no attribute `stdout`
  --> @dotmeow//components/capslock-control:48:42
   |
48 |     if "org.dotmeow.capslock-control" in r.stdout:
```

24 components in meowctl-stdlib and dotmeow read the fields as attributes,
among them `brew`, `mas`, `mise`, `go`, `python`, `rust` and `flatpak`, and
none reads them by index.

## What it should do, and why

[REQ-2820] says `run` returns a value carrying `stdout`, `stderr` and
`exit_code` under those names, and the parity principle in `CLAUDE.md` keeps
the Starlark API of `v0.1.0`. `v0.1.0` built the value with
`starlarkstruct.FromStringDict` in `internal/ctx/methods.go`, so a component
reads `result.stdout`. The value must be a struct again.

## Triage

[REQ-2820] covers the behaviour and `v0.1.0` shows the shape, so the fix enters
at implement. It is major because it breaks the package-manager components every
install, upgrade and verify goes through, and the corpus can't work round it
without dropping `v0.1.0` compatibility.

## Closed by

`run_returns_three_fields_and_does_not_fail_on_a_non_zero_exit` in
`crates/meowctl-ctx/tests/surface.rs` reads the result by attribute and holds
the shape as a regression check. Against the reproduction config, `verify`
now reports `capslock-control: OK`.

## Tasks

- [x] T-001 TSK-0039 return a struct from `ctx.run` in `crates/meowctl-ctx/src/value.rs`
  (the three attribute tests pass, and every stage exits 0)
