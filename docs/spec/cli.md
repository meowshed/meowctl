# Command-line interface

**Crate:** `meowctl-cli`
**Design:** `docs/design/0.2.0-rust-rewrite.md` §3, defects #1, #2, #7 and #8
**v0.1.0 equivalent:** `internal/cli/`

## Scope

This component is the command surface and nothing else: it parses arguments,
constructs the effects, calls into [`meowctl-engine`](engine.md), chooses a
sink, and maps an error to an exit code. It holds no domain logic.

That boundary is the point. In `v0.1.0` this layer also owns apply planning,
lockfile reading and writing, module syncing, and the staleness computation,
which is defects #1 and #2.

## Boundary

The commands, their flags, the exit codes, and what gets constructed where.

## Commands

**[R-CLI-001]** The command set MUST be exactly what `newRootCmd` in
`internal/cli/root.go` registers: `init`, `apply`, `add`, `remove`, `upgrade`,
`verify`, `update`, `status`, `doctor`, `check`, `hook`, `shell`,
`self-update`, `version`, the `dep` group, and shell completions.

**[R-CLI-002]** The `dep` group MUST carry `list`, `add`, `remove`, `upgrade`,
`sync`, and `tidy`.

**[R-CLI-003]** `init` with no argument MUST scaffold a config directory, and
`init <repo-url>` MUST bootstrap from a public dotfiles repository over HTTPS
with no `git` subprocess.

**[R-CLI-004]** `--config` MUST override the config directory resolution in
[R-COMMON-020], and `--verbose` MUST be available on every command.

**[R-CLI-005]** Each mutating command MUST accept `--dry-run`, and `apply`
MUST additionally accept `--force`, `--no-rollback`, and `--ignore-lock`,
matching `v0.1.0`.

**[R-CLI-006]** `--format json` MUST be available on every command and MUST
select `JsonSink`; see [R-TUI-032]. `--json` MUST remain accepted as an alias
on `doctor` and `status`, which are the two commands that had it.

## Construction

**[R-CLI-010]** The `FileSystem`, the `Executor`, and the sink MUST be
constructed here, once, from the parsed flags, and passed down. No crate below
constructs one; see [R-FS-014].

**[R-CLI-011]** `--dry-run` MUST select `DryRunFs` and the read-only
`Executor`. It MUST NOT be passed down as a boolean, and no code below this
crate MUST branch on it; see [R-CTX-014].

**[R-CLI-012]** Commands MUST be constructed per invocation. `v0.1.0` holds
`var RootCmd = newRootCmd()`, a package-level value built at init time, which
makes the command tree shared mutable state between tests.

**[R-CLI-013]** The version string MUST come from the build rather than from
variables patched by linker flags, and MUST report the version, the target,
the commit, and the build date, in the form `internal/version/version.go`
produces.

**[R-CLI-014]** The binary MUST install a handler for interruption and
suspension that finishes the sink before the process stops, so the cursor is
restored; see [R-TUI-023]. The handler belongs here because a signal arrives
at the process, and a sink that installed one would be a library taking a
process-wide resource.

## Output

**[R-CLI-020]** Every command MUST write through the sink. A command MUST NOT
write to stdout or stderr directly; the workspace lint denies it outside this
crate, and `meowctl-tui` owns the exception.

**[R-CLI-021]** `shell <shell>` MUST emit its integration snippet to stdout,
because the output is evaluated by the shell. This is the one command whose
stdout is a machine interface, and the sink MUST render it verbatim.

**[R-CLI-022]** A dry run MUST render the `Plan` and MUST report every skip
with its reason; see [R-ENGINE-052].

## Errors

**[R-CLI-030]** Mapping an error to an exit code MUST happen here and only
here, using the taxonomy in [R-COMMON-030].

**[R-CLI-031]** An error carrying a source span MUST be rendered as a
diagnostic showing the file, the line, and the offending source text; see
[R-STAR-040].

**[R-CLI-032]** A usage error MUST print usage and exit 2. A failure in the
work MUST NOT print usage, because a wall of flags on top of a real error
buries it.

**[R-CLI-033]** An error MUST go to stderr and MUST NOT corrupt a piped stdout.

## Failure paths

**[R-CLI-050]** A command that needs a configuration and is run outside one
MUST say so and suggest `meowctl init`, rather than reporting a missing file,
and MUST exit with the configuration code.

A command that only reports MUST NOT refuse. `status` in an empty directory
says no runs are recorded and exits zero, and `dep list` prints an empty
list; both are answers to the question asked. `apply`, `add`, `remove`,
`upgrade`, `verify`, `update` and the `dep` subcommands that write all need a
configuration and refuse without one.

**[R-CLI-051]** An interrupt MUST reach the engine, which stops cleanly; see
[R-ENGINE-062]. The sink MUST restore the terminal on the way out; see
[R-TUI-023].

**[R-CLI-052]** An unknown command or flag MUST exit 2 with the usage error,
and MUST suggest the nearest command when one is close.

## Parity with v0.1.0

The command set, the flag names and shorthands, the exit codes, and the version
string format are read from `internal/cli/` and `internal/version/`.

Three things change. Commands are constructed per invocation rather than in a
package variable. `--format json` becomes universal rather than a hand-written
special case on `doctor`. And this crate stops owning domain logic: apply
planning, lockfile access, module syncing, and staleness move to the crates
that own those concepts, which is what defects #1 and #2 ask for.
