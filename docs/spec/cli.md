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

**[R-CLI-005]** Each lifecycle command MUST accept `--dry-run`, `--verbose`
and `--no-rollback`, and `apply`, `add` and `upgrade` MUST additionally accept
`--force` and `--ignore-lock`, matching `v0.1.0`:

| Command | `-n` | `--no-rollback` | `-v` | `-f` | `--ignore-lock` |
| --- | --- | --- | --- | --- | --- |
| `apply` | yes | yes | yes | yes | yes |
| `add` | yes | yes | yes | yes | yes |
| `upgrade` | yes | yes | yes | yes | yes |
| `remove` | yes | yes | yes | no | no |
| `update` | yes | yes | yes | no | no |
| `verify` | yes | yes | yes | no | no |

An earlier draft said `--dry-run` on "each mutating command" and the other
four on `apply` alone. `addLifecycleFlags` and `addInstallFlags` in
`internal/cli/lifecycle.go` are the two groups, and the table is which command
calls which. The draft was written from `apply` rather than from the helpers,
and the implementation followed it: five commands shipped without
`--no-rollback`, two without `--force`, and `verify` without `--dry-run`.

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

**[R-CLI-015]** `--no-rollback` on `verify` MUST be accepted and MUST change
nothing. `verify` runs one read-only phase, which journals nothing, so there is
never anything to undo; see [R-COMMON-012].

**[R-CLI-016]** `--ignore-lock` MUST be accepted and MUST change nothing,
which is what `v0.1.0` does.

The flag is declared in `internal/cli/lifecycle.go:110` and read nowhere:
`IgnoreLock` is set on `runConfig` and no code path consults it. Resolving
without the lock is behaviour neither binary has ever had. Dropping the flag
would break a script that passes it; implementing it would be new behaviour
and needs its own requirement. Until then it is inert, and this says so rather
than leaving a reader to infer it from a help string.

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

## Updating the binary

**[R-CLI-070]** *Forward obligation, with [R-CLI-071] holding until the
release pipeline publishes a checksum.* `self-update` MUST NOT install a
binary it has not verified.
It MUST check the integrity of what it downloaded against a checksum the
release publishes, and MUST refuse rather than replace the running binary when
the two disagree; see [R-COMMON-005] and [R-MODULE-030], which is the rule
everything else meowctl downloads already follows.

`v0.1.0` does not. `runSelfUpdate` fetches a release asset over HTTPS and
renames it over `os.Executable()` with no check of any kind, and the source
says so: `SRI verification deferred (TODO below)`. Anything that can answer
for the download URL replaces the user's `meowctl`. This is the one place the
parity constraint does not reach, because reproducing it would mean shipping
the weakness deliberately, in the one crate that already knows how to verify a
download.

**[R-CLI-071]** Until a release publishes a checksum, `self-update` MUST say
that it cannot update this build and how to update it by hand, and MUST exit
with the general code.

Saying so is not the same as the command being absent: [R-CLI-001] keeps it on
the surface, and a script that calls it gets an error it can read rather than
"unknown command". The repository publishes no release assets yet, so there is
nothing to verify against and nothing to download.

## Runtime hooks

`hook <phase>` is what a shell runs on every spawn. `meowctl shell <shell>`
emits a snippet that calls it, so the requirements here are about a command
nobody types and everybody runs.

**[R-CLI-060]** `hook` MUST accept `shell` and `login` and no other phase. An
unsupported phase MUST exit 2 with the usage error, naming the two that work;
see [R-COMMON-013].

**[R-CLI-061]** `hook` MUST run the named phase for every discovered
component, in graph order, and MUST write what `ctx.emit` contributed to
stdout and nothing else. A decorated line would be evaluated by the shell.

**[R-CLI-062]** A failure to evaluate a component or to run a hook MUST be
recorded in `.hook-error` and MUST NOT change the exit code, which stays 0.

A shell that cannot start is worse than a shell that starts without its
integration. `v0.1.0` chose this and it is right: the failure is a
configuration error the user fixes at their leisure, and the alternative is a
terminal that reports an error on every prompt and a login that fails. The
flag file is how the failure stays visible without being fatal.

**[R-CLI-063]** A run in which nothing failed MUST remove `.hook-error`.

**[R-CLI-064]** `status` and `doctor` MUST report the flag when it is present.
`status` MUST say that it failed and where to look; `doctor` MUST render the
recorded text. Both MUST use the warning level, which is what a reader scans
for.

`v0.1.0` writes the `status` line to stderr so that `meowctl status | ...`
stays clean. That does not carry over: a sink owns its destination and writes
one stream, and splitting one command across two would mean the live region
and the JSON stream disagreeing about what the command said. The line is a
warning in the stream instead, which is visible rather than hidden. This is
inside the terminal output carve-out; see [R-TUI-032].

**[R-CLI-065]** `hook` MUST NOT journal what a hook does and MUST NOT roll
back. A `shell` hook that writes is misusing the phase, and a rollback on a
shell spawn would undo the previous one.

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

**[R-CLI-053]** A second interrupt MUST stop the process at once, with the
default disposition.

**[R-CLI-054]** A run that stopped because it was interrupted MUST say so and
MUST exit with the general code.

Not zero: the command did not do what was asked, and a script that treats an
interrupted `apply` as a successful one goes on to the next step. Not a code
of its own either -- [R-COMMON-030] fixes the four `v0.1.0` defines and says
"and no others". The process still dies from the second interrupt, which is
where the shell's 130 comes from, as it does under `v0.1.0`.

A user who has asked twice is not asking for a tidier stop. The first
interrupt asks the run to finish what it is doing and stop; if that is taking
longer than they are willing to wait, the second has to work, and a process
that swallows its own interrupt is one nobody can stop.

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
