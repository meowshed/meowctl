# Common vocabulary

**Crate:** `meowctl-common`
**Design:** `docs/design/0.2.0-rust-rewrite.md` §3
**v0.1.0 equivalent:** scattered across `internal/cli`, `internal/lifecycle`, `internal/lock`

## Scope

This component owns the names every other crate uses: component identifiers,
lifecycle phases, module references, integrity hashes, the error taxonomy with
its exit codes, and the `Event` vocabulary the engine emits and the sinks
render. It performs no input or output and depends on no other crate in the
workspace.

It exists because `v0.1.0` had no such layer. A component was a `string`, a
phase was a `string`, and an exit code was an integer literal passed to
`exitErrorf` at the call site. Every defect where the wrong string reached the
wrong function was possible because nothing distinguished them.

The `Event` type lives here rather than in `meowctl-engine` so that a sink
never depends on the producer. That is what lets the sinks be tested against a
fixture stream with no engine and no terminal.

## Boundary

The public types, their string forms, and their parse rules. Nothing here
reads a file, spawns a process, or prints.

## Identifiers

**[R-COMMON-001]** A `ComponentId` MUST be constructible from the three forms
`v0.1.0` accepts: a bare name (`neovim`), a registry-qualified path
(`@stdlib//components/zsh`), and a GitHub-qualified path
(`github.com/owner/repo//components/zsh`). Its `Display` MUST reproduce the
input exactly.

**[R-COMMON-002]** A `ComponentId` MUST expose the module key its source
belongs to: `dotmeow` for `@dotmeow//path`, `github.com/o/r` for
`github.com/o/r//path`, and nothing for a bare name. This is the key used to
look an entry up in a lock file, and `internal/cli/apply.go`'s
`moduleKeyFromComponentURL` is the behaviour to reproduce.

**[R-COMMON-003]** A `ModuleRef` MUST distinguish a registry module, named by a
bare identifier, from a GitHub module, named `github:owner/repo@ref`. Parsing a
string that is neither MUST fail rather than produce a registry module with a
strange name.

**[R-COMMON-006]** A `ComponentId` MUST expose its logical name: the last
path segment of a qualified form, and the whole of a bare one. So
`@stdlib//components/node` and `github://o/r//components/node` are both `node`.
This is the name an `after` list refers to and the key a lock entry and a
sentinel record use; `logicalName` in `internal/starlark/accumulator.go` is the
behaviour.

**[R-COMMON-004]** An `Integrity` MUST hold a W3C Subresource Integrity hash in
the form `sha384-<base64>`, and MUST reject a string that is not one. This is
what `deps.lock` stores; see [R-CONFIG-020].

**[R-COMMON-005]** Computing an `Integrity` from bytes MUST live with the type:
SHA-384 in standard base64, which is what `computeSRI` in
`internal/starlark/loader/github.go` produces and what every published
`index.toml` carries. Two things hash bytes -- the module cache and
`ctx.download` -- and an implementation each is how a tool ends up with two
encodings of the same hash and no way to tell them apart.

## Phases

**[R-COMMON-010]** `Phase` MUST have exactly the thirteen variants `v0.1.0`
defines in `internal/lifecycle/runner.go`: `install_check`, `install`,
`install_configure`, `update`, `upgrade_check`, `upgrade`,
`upgrade_configure`, `uninstall_check`, `uninstall`, `uninstall_cleanup`,
`shell`, `login`, and `verify`. Its string form MUST match those names exactly,
because they are the hook names a component author writes and they appear in
`state.toml`.

**[R-COMMON-011]** `PhaseSet` MUST have the five variants `install`, `update`,
`upgrade`, `uninstall`, and `verify`, each yielding its phases in the order
`internal/lifecycle/runner.go` gives:

| Set | Phases |
| --- | --- |
| `install` | `install_check`, `install`, `install_configure` |
| `update` | `update` |
| `upgrade` | `upgrade_check`, `upgrade`, `upgrade_configure` |
| `uninstall` | `uninstall_check`, `uninstall`, `uninstall_cleanup` |
| `verify` | `verify` |

**[R-COMMON-012]** `Phase` MUST report whether it is read-only. The read-only
phases are `install_check`, `upgrade_check`, `uninstall_check`, and `verify`. A
read-only phase gets a restricted `ctx`; see [R-CTX-030].

An earlier draft of this requirement cited a `validCheckPhases` set in
`internal/ctx/methods.go`. There is no such set: `v0.1.0` never asks whether a
phase is read-only, and every hook in every phase gets the full `ctx`. The four
names are what the phases mean, not what a Go identifier says.

**[R-COMMON-013]** `Phase` MUST report whether it is a runtime hook phase. The
runtime hook phases are `shell` and `login`. Only in those does `ctx.emit`
write to stdout; see [R-CTX-024].

## Paths

**[R-COMMON-020]** The config directory MUST resolve to `$MEOWCTL_CONFIG` when
set, then `$XDG_CONFIG_HOME/meowctl`, then `~/.config/meowctl`, in that order.
The `--config` flag overrides all of them; see [R-CLI-004].

**[R-COMMON-021]** The module cache directory MUST resolve to
`$XDG_CACHE_HOME/meowctl/modules`, falling back to `~/.cache/meowctl/modules`.

`cacheDir` in `internal/cli/sync.go` uses the fallback unconditionally and
never reads `$XDG_CACHE_HOME`, so a machine that relocates its cache still has
`v0.1.0` writing to the default. Honouring the variable is a deliberate change,
and it is why the compatibility corpus cannot share a cache with `v0.1.0`
through that variable.

**[R-COMMON-023]** The home directory MUST come from `$HOME`, and from
`$USERPROFILE` when `$HOME` is unset on Windows. Neither being set MUST be an
error rather than a guess.

`os.UserHomeDir`, which `v0.1.0` calls, reads `$USERPROFILE` on Windows and
`$HOME` everywhere else. Reading only `$HOME` makes every invocation on
Windows fail before it does anything, which is what the test matrix caught
once a test ran the binary rather than a function.

**[R-COMMON-022]** Path resolution MUST expand a leading `~` to the home
directory, matching `expandPath` in `internal/ctx/methods.go`, and MUST reject
a path that is not absolute once expanded. A path that escapes the directory it
was resolved against MUST be rejected rather than silently normalised.

`requirePath` rejects only the empty string, so `v0.1.0` accepts a relative
path and resolves it against whatever the process working directory happens to
be. A hook has no defined working directory, so that result is unpredictable;
refusing is a deliberate change. `~user` is refused for the same reason:
`expandPath` passes it through, which creates a directory literally named
`~user`.

## Errors and exit codes

**[R-COMMON-030]** The error taxonomy MUST map onto the exit codes `v0.1.0`
defines in `internal/cli/errors.go`, and no others:

| Code | Meaning |
| --- | --- |
| 0 | Success |
| 1 | General error |
| 2 | Usage error |
| 3 | Configuration error: a malformed `init.star`, a missing component, a legacy layout |
| 4 | Module error: a fetch that failed, an integrity hash that did not match |

**[R-COMMON-031]** Every crate's error enum MUST be convertible to an exit
code, and that conversion MUST be the only place a code is chosen. A crate
below `meowctl-cli` MUST NOT name an exit code.

**[R-COMMON-032]** An error that has a source location MUST carry it as a span
into the file it came from, so `meowctl-cli` can render a diagnostic that
points at the line. `v0.1.0` could not do this, and its Starlark errors name a
file and nothing more.

## Events

**[R-COMMON-040]** The `Event` enum MUST cover everything a command produces
that a sink renders. At minimum: `PlanComputed`, `PhaseStarted`,
`PhaseFinished`, `ComponentStarted`, `ComponentSkipped`, `ComponentFinished`,
`OpApplied`, `ProcessStarted`, `ProcessOutput`, `ProcessFinished`,
`TerminalRequested`, `TerminalReleased`, `Diagnostic`, and
`Message { severity, text }`.

**[R-COMMON-044]** The set MUST include `ShellLine` and `PathPrepended`, which
`ctx.emit` and `ctx.add_path` produce. Both are effects on the process rather
than on the filesystem, and `ctx` reaches every effect through something it was
given; see [R-CTX-024] and [R-CTX-025]. `v0.1.0` writes to stdout and calls
`setenv` from inside the method, which is why neither can be dry-run, rendered
as JSON, or tested without a process.

**[R-COMMON-041]** An `Event` MUST be serializable to JSON, because `JsonSink`
emits one object per event; see [R-TUI-030].

**[R-COMMON-042]** `Message` MUST be the only variant carrying unstructured
text, and it MUST carry a severity. Anything a sink needs to render differently
from other text MUST be its own variant rather than a formatted string.

**[R-COMMON-043]** `Event` MUST NOT carry a rendered string, a colour, a glyph,
or a width. Those are the sink's decisions, and an event that carries one makes
the JSON sink emit terminal decoration.

## Failure paths

**[R-COMMON-050]** Parsing any identifier from an untrusted string MUST return
an error rather than panic. The inputs reach this crate from config files and
lock files, both of which a user edits.

**[R-COMMON-051]** Constructing a `Phase` from a string that names no phase
MUST fail. A `state.toml` written by a newer version can contain one; see
[R-CONFIG-041].

## Parity with v0.1.0

The phase names, the phase-set composition, the exit codes, and the config
directory resolution are all reproduced exactly. The types are new: `v0.1.0`
used `string` for a component, a phase, and a module key alike, and
`internal/lifecycle` declared a `ComponentID` alias that was still a string
underneath.

Three requirements deliberately change behaviour. [R-COMMON-020] adds
`$MEOWCTL_CONFIG`, which `internal/cli/config.go` does not consult, because the
compat corpus needs to point two binaries at one configuration without a flag
on every invocation. [R-COMMON-021] honours `$XDG_CACHE_HOME`, which
`internal/cli/sync.go` ignores. [R-COMMON-022] refuses a relative path and
`~user`, both of which `internal/ctx/methods.go` accepts and resolves to
something the component author did not ask for.
