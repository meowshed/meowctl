---
id: SPC-3400
artifact: spec
status: live
revised: 2026-09-27
checked-at:
states: [REQ-3401, REQ-3402, REQ-3403, REQ-3404, REQ-3405, REQ-3406, REQ-3410, REQ-3411, REQ-3412, REQ-3413, REQ-3414, REQ-3415, REQ-3416, REQ-3420, REQ-3421, REQ-3422, REQ-3430, REQ-3431, REQ-3432, REQ-3433, REQ-3450, REQ-3451, REQ-3452, REQ-3453, REQ-3454, REQ-3460, REQ-3461, REQ-3462, REQ-3463, REQ-3464, REQ-3465, REQ-3470, REQ-3471, REQ-3472, REQ-3473, REQ-3474, REQ-3475, REQ-3476, REQ-3500, REQ-3501, REQ-3502, REQ-3503, REQ-3504, REQ-3505, REQ-3506, REQ-3507, REQ-3508, REQ-3509, REQ-3510, REQ-3511, REQ-3512, REQ-3513, REQ-3514, REQ-3515, REQ-3516, REQ-3517, REQ-3518, REQ-3519, REQ-3520, REQ-3521, REQ-3522, REQ-3523, REQ-3524, REQ-3525, REQ-3526, REQ-3527, REQ-3528, REQ-3529, REQ-3530, REQ-3531, REQ-3532, REQ-3540, REQ-3541, REQ-3542, REQ-3543, REQ-3570, REQ-3571, REQ-3572, REQ-3590]
---

# The command-line interface

## Scope

This component is the command surface and nothing else: it parses arguments,
constructs the effects, calls into `meowctl-engine`, chooses a sink, and maps an
error to an exit code, and it holds no domain logic (from docs/spec/cli.md,
high). It lives in the `meowctl-cli` crate, its design is
`docs/design/0.2.0-rust-rewrite.md` §3 with defects #1, #2, #7 and #8, and its
`v0.1.0` equivalent is `internal/cli/` (from docs/spec/cli.md, high).

In `v0.1.0` this layer also owns apply planning, lockfile reading and writing,
module syncing and the staleness computation, which is defects #1 and #2; in
`0.2.0` those move to the crates that own the concepts (from docs/spec/cli.md,
high). The command set, the flag names and shorthands, the exit codes and the
version string format are read from `internal/cli/` and `internal/version/`
(from docs/spec/cli.md, high).

This specification also covers `self-update` and the `meowctl-release` crate,
which holds what a release is, which asset belongs to this platform and whether
the downloaded bytes are that asset (from CLAUDE.md, high). The engine, the
sinks, the configuration schemas and the exit-code taxonomy have specifications
of their own (from docs/spec/cli.md, high).

## Boundary

The boundary is the commands, their flags, the exit codes, and what gets
constructed where (from docs/spec/cli.md, high).

| Surface | What is observable | Requirements |
| --- | --- | --- |
| Commands | `init`, `apply`, `add`, `remove`, `upgrade`, `verify`, `update`, `status`, `doctor`, `check`, `hook`, `shell`, `self-update`, `version`, the `dep` group and shell completions | [REQ-3401], [REQ-3402] |
| Global flags | `--config`, `--verbose`, `--format json` | [REQ-3404], [REQ-3501], [REQ-3406] |
| Lifecycle flags | `--dry-run` (`-n`), `--no-rollback`, `--verbose` (`-v`), `--force` (`-f`), `--ignore-lock` | [REQ-3405], [REQ-3502] |
| Standard output | The sink's rendering; the `shell` snippet and `hook` output verbatim | [REQ-3420], [REQ-3421], [REQ-3521] |
| Standard error | Errors, never mixed into a piped stdout | [REQ-3433], [REQ-3514] |
| Exit codes | 2 for a usage error, the configuration code outside a configuration, the general code after an interrupt, 0 from `hook` | [REQ-3432], [REQ-3515], [REQ-3519], [REQ-3522] |
| Files | `.hook-error`, written by `hook` and read by `status` and `doctor` | [REQ-3462], [REQ-3463], [REQ-3464] |
| Release | `checksums.sri` on every release, and the `MEOWCTL_RELEASES` environment variable | [REQ-3471], [REQ-3476] |

## Behaviour

### Commands

The command set is exactly what `newRootCmd` in `internal/cli/root.go` registers
[REQ-3401], and the `dep` group carries `list`, `add`, `remove`, `upgrade`,
`sync` and `tidy` [REQ-3402] (from docs/spec/cli.md, high).

`init` with no argument scaffolds a config directory [REQ-3403], and
`init <repo-url>` bootstraps from a public dotfiles repository over HTTPS with
no `git` subprocess [REQ-3500] (from docs/spec/cli.md, high).

`--config` overrides the config directory resolution in [REQ-1020] [REQ-3404],
and `--verbose` is available on every command [REQ-3501] (from docs/spec/cli.md,
high).

Each lifecycle command accepts `--dry-run`, `--verbose` and `--no-rollback`
[REQ-3405], and `apply`, `add` and `upgrade` also accept `--force` and
`--ignore-lock` [REQ-3502], as this table shows (from docs/spec/cli.md, high):

| Command | `-n` | `--no-rollback` | `-v` | `-f` | `--ignore-lock` |
| --- | --- | --- | --- | --- | --- |
| `apply` | yes | yes | yes | yes | yes |
| `add` | yes | yes | yes | yes | yes |
| `upgrade` | yes | yes | yes | yes | yes |
| `remove` | yes | yes | yes | no | no |
| `update` | yes | yes | yes | no | no |
| `verify` | yes | yes | yes | no | no |

`--format json` is available on every command [REQ-3406] and selects `JsonSink`,
as [REQ-3232] describes [REQ-3503] (from docs/spec/cli.md, high). `--json` stays
accepted as an alias on `doctor` and `status` [REQ-3504] (from docs/spec/cli.md,
high).

### Construction

`meowctl-cli` constructs the `FileSystem`, the `Executor` and the sink once,
from the parsed flags, and passes them down; no crate below constructs one
[REQ-3410] (from docs/spec/cli.md, high).

`--dry-run` selects `DryRunFs` and the read-only `Executor` [REQ-3411]. It is
not passed down as a boolean [REQ-3505], and no code below `meowctl-cli`
branches on it [REQ-3506] (from docs/spec/cli.md, high).

The command tree is constructed per invocation [REQ-3412] (from
docs/spec/cli.md, high).

The version string comes from the build, not from variables patched by linker
flags [REQ-3413], and it reports the version, the target, the commit and the
build date in the form `internal/version/version.go` produces [REQ-3507] (from
docs/spec/cli.md, high).

The binary installs a handler for interruption and suspension that finishes the
sink before the process stops, so the cursor is restored [REQ-3414] (from
docs/spec/cli.md, high).

`--no-rollback` on `verify` is accepted [REQ-3415] and changes nothing
[REQ-3508] (from docs/spec/cli.md, high). `--ignore-lock` is accepted [REQ-3416]
and changes nothing [REQ-3509] (from docs/spec/cli.md, high).

### Output

Every command writes through the sink [REQ-3420], and no command writes to
stdout or stderr directly [REQ-3510] (from docs/spec/cli.md, high).

`shell <shell>` emits its integration snippet to stdout [REQ-3421], and the sink
renders the snippet verbatim [REQ-3511] (from docs/spec/cli.md, high).

A dry run renders the `Plan` [REQ-3422] and reports every skip with its reason
[REQ-3512] (from docs/spec/cli.md, high).

### Errors

`meowctl-cli` is the only place that maps an error to an exit code, using the
taxonomy in [REQ-1030] [REQ-3430] (from docs/spec/cli.md, high).

An error carrying a source span renders as a diagnostic showing the file, the
line and the offending source text [REQ-3431] (from docs/spec/cli.md, high).

A usage error prints usage and exits 2 [REQ-3432], and a failure in the work
prints no usage [REQ-3513] (from docs/spec/cli.md, high).

An error goes to stderr [REQ-3433] and leaves a piped stdout intact [REQ-3514]
(from docs/spec/cli.md, high).

### Updating the binary

`self-update` asks for the latest release [REQ-3472], at the address
`MEOWCTL_RELEASES` names when it is set [REQ-3476] (from docs/spec/cli.md,
high). It compares the release tag with the running version [REQ-3530], and when
they match it reports being up to date and changes nothing [REQ-3531] (from
docs/spec/cli.md, high).

`self-update` picks the asset named for this platform [REQ-3473] (from
docs/spec/cli.md, high). It installs no binary it has not verified [REQ-3470]:
it checks the download against the checksum the release publishes [REQ-3527]
(from docs/spec/cli.md, high).

Every release publishes `checksums.sri`, one line per asset, each
`<sri>  <asset name>` [REQ-3471] (from docs/spec/cli.md, high). Every release
publishes a binary for `x86_64-unknown-linux-gnu`, `aarch64-apple-darwin`,
`x86_64-apple-darwin` and `x86_64-pc-windows-msvc` [REQ-3570] (from
.github/workflows/release.yml:23-30, high).

`self-update` replaces the running binary atomically: it writes the new binary
beside the old one, makes it executable, then renames it over the old one
[REQ-3474] (from docs/spec/cli.md, high).

`self-update` runs only on its own, never as part of another command [REQ-3475]
(from docs/spec/cli.md, high).

### Runtime hooks

`hook <phase>` is what a shell runs on every spawn, through the snippet
`meowctl shell <shell>` emits (from docs/spec/cli.md, high).

`hook` accepts the `shell` and `login` phases and no other [REQ-3460] (from
docs/spec/cli.md, high). It runs the named phase for every discovered component,
in graph order [REQ-3461], and writes to stdout what `ctx.emit` contributed and
nothing else [REQ-3521] (from docs/spec/cli.md, high).

`hook` neither journals what a hook does [REQ-3465] nor rolls back [REQ-3526]
(from docs/spec/cli.md, high).

`hook shell` opens no network connection and starts no asynchronous runtime
[REQ-3571], because a shell pays for it on every spawn (from
docs/design/0.2.0-rust-rewrite.md:27-28, high).

A `hook` run in which nothing failed removes `.hook-error` [REQ-3463] (from
docs/spec/cli.md, high).

When `.hook-error` is present, `status` and `doctor` report it [REQ-3464]:
`status` says the hook run failed and where to look [REQ-3523], and `doctor`
renders the recorded text [REQ-3524], both at the warning level [REQ-3525] (from
docs/spec/cli.md, high).

`self-update` allows its binary download up to 5 minutes end to end [REQ-3572]
(from
project/requirements/REQ-3572-self-update-download-bounded-at-five-minutes.md,
high).

## Failure paths

| Condition | What happens | Requirements |
| --- | --- | --- |
| A command that needs a configuration runs outside one | It says so, suggests `meowctl init` and exits with the configuration code; it does not report a missing file | [REQ-3450], [REQ-3515] |
| A command that only reports runs outside a configuration | It answers: `status` says no runs are recorded and exits zero, and `dep list` prints an empty list | [REQ-3516] |
| An unknown command or flag | Exit 2 with the usage error, suggesting the nearest command or flag when one is close | [REQ-3452], [REQ-3518], [REQ-3590] |
| A usage error | Usage is printed and the exit code is 2 | [REQ-3432] |
| `--config` names a directory that doesn't exist | The same as running outside a configuration; `init` creates the directory | [REQ-3404], [REQ-3450] |
| A failure in the work | The error goes to stderr with no usage | [REQ-3433], [REQ-3513] |
| An error carries a source span | A diagnostic with the file, the line and the source text | [REQ-3431] |
| The first interrupt | It reaches the engine, which stops cleanly, and the sink restores the terminal on the way out | [REQ-3451], [REQ-3517] |
| A second interrupt | The process stops at once, with the default disposition | [REQ-3453] |
| A run stopped by an interrupt | It says so and exits with the general code | [REQ-3454], [REQ-3519] |
| `hook` is given a phase other than `shell` or `login` | Exit 2 with the usage error, naming the two phases that work, as [REQ-1013] describes | [REQ-3520] |
| `hook` fails to evaluate a component or to run a hook | The failure is recorded in `.hook-error` and the exit code stays 0 | [REQ-3462], [REQ-3522] |
| `.hook-error` can't be written or removed | `hook` exits 0 and writes one line to stderr naming the path | [REQ-3522], [REQ-3542] |
| The release publishes no `checksums.sri` | `self-update` refuses to proceed | [REQ-3529] |
| The download disagrees with its published checksum | `self-update` refuses and leaves the running binary in place | [REQ-3528] |
| The release has no asset for this platform | `self-update` says which platform it looked for and where the release is, and reports no network failure | [REQ-3532] |
| A request fails during `self-update` | It exits with the module code, names the URL once, and leaves the running binary unchanged | [REQ-1030], [REQ-3540] |
| The staged binary can't be made executable or renamed into place | The running binary is unchanged, the staged file is removed, and the command exits with the general code | [REQ-3474], [REQ-3543] |
| `init <repo-url>` can't fetch the archive | It exits with the module code, names the URL and writes nothing | [REQ-1030], [REQ-3541] |
| The archive has no `init.star` at its root | It exits with the configuration code and leaves the directory as it was | [REQ-1030], [REQ-3541] |

Every row states `docs/spec/cli.md` (from docs/spec/cli.md, high).
