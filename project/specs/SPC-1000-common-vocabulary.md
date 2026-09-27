---
id: SPC-1000
artifact: spec
status: live
revised: 2026-09-27
checked-at:
states: [REQ-1001, REQ-1002, REQ-1003, REQ-1004, REQ-1005, REQ-1006, REQ-1010, REQ-1011, REQ-1012, REQ-1013, REQ-1020, REQ-1021, REQ-1022, REQ-1023, REQ-1030, REQ-1031, REQ-1032, REQ-1040, REQ-1041, REQ-1042, REQ-1043, REQ-1044, REQ-1050, REQ-1051, REQ-1100, REQ-1101, REQ-1102, REQ-1103, REQ-1104, REQ-1105, REQ-1106, REQ-1107, REQ-1108, REQ-1109, REQ-1110]
---

# The common vocabulary every crate shares

## Scope

This part is the `meowctl-common` crate, designed in
`docs/design/0.2.0-rust-rewrite.md` §3, and replaces code that `v0.1.0` spread
across `internal/cli`, `internal/lifecycle` and `internal/lock` (from
docs/spec/common.md, high).

This component owns the names every other crate uses: component identifiers,
lifecycle phases, module references, integrity hashes, the error taxonomy with
its exit codes, and the `Event` vocabulary the engine emits and the sinks
render. It performs no input or output and depends on no other crate in the
workspace (from docs/spec/common.md, high).

The `Event` type lives here rather than in `meowctl-engine` so that a sink never
depends on the producer, which lets the sinks be tested against a fixture stream
with no engine and no terminal (from docs/spec/common.md, high).

## Boundary

The public types, their string forms, and their parse rules. Nothing here reads
a file, spawns a process, or prints (from docs/spec/common.md, high).

| Surface | What it is |
| --- | --- |
| `ComponentId` | A component in one of three forms, with its module key and logical name (from docs/spec/common.md, high) |
| `ModuleRef` | A registry module or a GitHub module (from docs/spec/common.md, high) |
| `Integrity` | A Subresource Integrity hash, parsed or computed from bytes (from docs/spec/common.md, high) |
| `Phase` and `PhaseSet` | The thirteen lifecycle phases and the five sets that group them (from docs/spec/common.md, high) |
| Path resolution | The config directory, the module cache directory, the home directory and `~` expansion (from docs/spec/common.md, high) |
| Error taxonomy | The classification every crate's error maps onto, and its exit codes (from docs/spec/common.md, high) |
| `Event` | Everything a command produces that a sink renders (from docs/spec/common.md, high) |

## Behaviour

### Identifiers

A `ComponentId` is constructed from a bare name (`neovim`), a registry-qualified
path (`@stdlib//components/zsh`) or a GitHub-qualified path
(`github.com/owner/repo//components/zsh`) [REQ-1001], and its `Display`
reproduces the input exactly [REQ-1100] (from docs/spec/common.md, high).

A `ComponentId` exposes the module key its source belongs to: `dotmeow` for
`@dotmeow//path`, `github.com/o/r` for `github.com/o/r//path`, and nothing for a
bare name [REQ-1002] (from docs/spec/common.md, high).

A `ComponentId` exposes its logical name: the last path segment of a qualified
form and the whole of a bare one, so `@stdlib//components/node` and
`github://o/r//components/node` are both `node` [REQ-1006] (from
docs/spec/common.md, high).

A `ModuleRef` is either a registry module, named by a bare identifier, or a
GitHub module, named `github:owner/repo@ref` [REQ-1003] (from
docs/spec/common.md, high).

An `Integrity` holds a hash in the form `sha384-<base64>` [REQ-1004], and
computing one from bytes lives with the type, as SHA-384 in standard base64
[REQ-1005] (from docs/spec/common.md, high).

### Phases

`Phase` has the thirteen variants `install_check`, `install`,
`install_configure`, `update`, `upgrade_check`, `upgrade`, `upgrade_configure`,
`uninstall_check`, `uninstall`, `uninstall_cleanup`, `shell`, `login` and
`verify` [REQ-1010], and its string form is exactly those names [REQ-1103] (from
docs/spec/common.md, high).

`PhaseSet` has five variants, each yielding its phases in this order [REQ-1011]
(from docs/spec/common.md, high):

| Set | Phases |
| --- | --- |
| `install` | `install_check`, `install`, `install_configure` |
| `update` | `update` |
| `upgrade` | `upgrade_check`, `upgrade`, `upgrade_configure` |
| `uninstall` | `uninstall_check`, `uninstall`, `uninstall_cleanup` |
| `verify` | `verify` |

`Phase` reports whether it is read-only, and the read-only phases are
`install_check`, `upgrade_check`, `uninstall_check` and `verify` [REQ-1012]
(from docs/spec/common.md, high).

`Phase` reports whether it is a runtime hook phase, and the runtime hook phases
are `shell` and `login` [REQ-1013] (from docs/spec/common.md, high).

### Paths

The config directory resolves to `$MEOWCTL_CONFIG` when set, then
`$XDG_CONFIG_HOME/meowctl`, then `~/.config/meowctl`, and the `--config` flag
overrides all three [REQ-1020] (from docs/spec/common.md, high).

The module cache directory resolves to `$XDG_CACHE_HOME/meowctl/modules`, and
falls back to `~/.cache/meowctl/modules` [REQ-1021] (from docs/spec/common.md,
high).

The home directory comes from `$HOME`, and from `$USERPROFILE` when `$HOME` is
unset on Windows [REQ-1023] (from docs/spec/common.md, high).

Path resolution expands a leading `~` to the home directory [REQ-1022] (from
docs/spec/common.md, high).

### Errors and exit codes

The error taxonomy maps onto the exit codes `v0.1.0` defines, and no others
[REQ-1030] (from docs/spec/common.md, high):

| Code | Meaning |
| --- | --- |
| 0 | Success |
| 1 | General error |
| 2 | Usage error |
| 3 | Configuration error: a malformed `init.star`, a missing component, a legacy layout |
| 4 | Module error: a fetch that failed, an integrity hash that did not match |

Every crate's error enum converts to an exit code [REQ-1031], that conversion is
the only place a code is chosen [REQ-1107], and no crate below `meowctl-cli`
names an exit code [REQ-1108] (from docs/spec/common.md, high).

An error that has a source location carries it as a span into the file it came
from [REQ-1032] (from docs/spec/common.md, high).

### Events

The `Event` enum covers everything a command produces that a sink renders,
including at least `PlanComputed`, `PhaseStarted`, `PhaseFinished`,
`ComponentStarted`, `ComponentSkipped`, `ComponentFinished`, `OpApplied`,
`ProcessStarted`, `ProcessOutput`, `ProcessFinished`, `TerminalRequested`,
`TerminalReleased`, `Diagnostic` and `Message { severity, text }` [REQ-1040]
(from docs/spec/common.md, high).

The set includes `ShellLine` and `PathPrepended`, which `ctx.emit` and
`ctx.add_path` produce [REQ-1044] (from docs/spec/common.md, high).

An `Event` serialises to JSON [REQ-1041] (from docs/spec/common.md, high).

`Message` is the only variant carrying unstructured text [REQ-1042] and carries
a severity [REQ-1109], and anything a sink renders differently from other text
is its own variant [REQ-1110] (from docs/spec/common.md, high).

An `Event` carries no rendered string, colour, glyph or width [REQ-1043] (from
docs/spec/common.md, high).

## Failure paths

| Condition | What happens |
| --- | --- |
| A `ModuleRef` string is neither a registry module nor a GitHub module | Parsing fails, and no registry module is produced [REQ-1101] (from docs/spec/common.md, high) |
| An `Integrity` string isn't `sha384-<base64>` | The `Integrity` rejects it [REQ-1102] (from docs/spec/common.md, high) |
| Any identifier is parsed from an untrusted, malformed string | Parsing returns an error and doesn't panic [REQ-1050] (from docs/spec/common.md, high) |
| A string names no phase | Constructing a `Phase` from it fails [REQ-1051] (from docs/spec/common.md, high) |
| A path isn't absolute once `~` is expanded | Path resolution rejects it [REQ-1104] (from docs/spec/common.md, high) |
| A path escapes the directory it was resolved against | Path resolution rejects it and doesn't normalise it [REQ-1105] (from docs/spec/common.md, high) |
| Neither `$HOME` nor `$USERPROFILE` is set | Resolving the home directory is an error [REQ-1106] (from docs/spec/common.md, high) |
