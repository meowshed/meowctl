---
id: BUG-0002
artifact: bug
status: approved
severity: critical
violates: REQ-2341
found: 2026-09-27
revised: 2026-09-27
issue:
---

# meowctl never reads the Linux distribution, so every `distros` guard and distribution case fails on Linux

Nothing in the binary reads `/etc/os-release` or detects WSL, so on every Linux
machine `platform().distro`, `distro_like` and `version_id` are empty and `wsl`
is `False`.

## Reproduction

The onboarding research inferred this from the code at revision `7e8cabe` and
didn't run it. The probe machine runs macOS, so the research couldn't run it on
Linux. At revision `7e8cabe`, from the repository root:

```bash
grep -rn "os-release\|WSL" crates src
```

The search finds no reader. `Platform::current()` fills only `os` and says "the
distribution fields are filled by the caller"
(`crates/meowctl-starlark/src/platform.rs:28-44`). Its only callers,
`crates/meowctl-cli/src/run.rs:273` and `crates/meowctl-module/src/sync.rs:359`,
don't fill them. On a Linux machine, a component declaring `distros =
["debian"]` then reproduces it: `meowctl apply` drops the component.

## What the system does

`//platform:linux-debian`, `-arch`, `-fedora` and `//platform:wsl` never match
(`platform.rs:60-65`). `Discovered::runs_on` drops every component that declares
`distros = [...]`, because an empty string equals no name
(`crates/meowctl-engine/src/discovery.rs:64-77`). Tests build `Platform` values
by hand (`platform.rs:138-300`,
`crates/meowctl-starlark/tests/evaluation.rs:681`), so none of them sees the
missing detection.

## What it should do, and why

[REQ-2341] says `platform()` reports the `ID`, `ID_LIKE` and `VERSION_ID` values
of `/etc/os-release` on Linux, as `v0.1.0`'s `linuxDistroInfo` does, and
[REQ-2207] and [REQ-3012] depend on it. The fix reads the file and the kernel
release in `meowctl-cli`, the only crate allowed to read the machine, and fills
`Platform` before passing it down.

## Triage

A requirement covers it, so the fix enters at implement. It is critical because
the standard library's package-manager components `apt`, `dnf`, `pacman` and
`apk` scope themselves with `distros=` (history #32), so on Linux every one of
them is dropped and every `pkg()` fails with "no component handles the package
manager" (medium: inferred from #32 and [REQ-3012], not evaluated on Linux).

## Closed by

Open. The fix adds a test that fails when `distro` is empty on a Linux CI
runner.
