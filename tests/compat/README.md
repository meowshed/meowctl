# The v0.1.0 compatibility corpus

This is the oracle the rewrite is verified against. `v0.1.0` still runs, so the
question "does `v0.2.0` do the same thing" has an answer that can be measured
rather than argued.

## Running it

```sh
mise run compat-record       # capture what the v0.1.0 binary does
mise run compat-check        # diff the Rust binary against it
mise run compat-self-check   # check the fixtures against v0.1.0 itself
```

`compat-check` reports that there is nothing to check until the Rust binary
exists. That is expected until the CLI crate lands.

## What is compared

Exit codes, the sandbox file tree with a hash per file, and the contents of
every file under the configuration directory. Rendered output is **not**
compared: terminal output is the one deliberate carve-out from the parity
constraint, so a sink that words a line differently must not fail the corpus.

The exception is a command whose stdout is a machine interface. `shell <shell>`
emits code the shell evaluates and `doctor --json` is read by programs, so
their stdout is compared byte for byte.

## What is normalised

Three paths and two build details, and nothing else:

| Rewritten | To | Why |
| --- | --- | --- |
| The sandbox, home, and config directories | `<SANDBOX>`, `<HOME>`, `<CONFIG>` | A fresh sandbox has a fresh path |
| Anything shaped like a timestamp | `<TIMESTAMP>` | `state.toml` records when the run started |
| The commit in `meowctl version` | `<BUILD>` | It changes on every commit |

Normalising hides a class of real difference: a binary that printed the wrong
path would still match if the wrong path happened to be the sandbox. The
alternative is a corpus that cannot be recorded at all. The rewrite is kept
narrow for that reason.

## What the corpus will not run

`verify` is not in the command list. Its whole job is to execute a component's
verification hook, and in the standard library that means `open -a <App>` to
check an application is installed. Running it launches every application the
configuration manages.

More generally, a command that evaluates hooks runs whatever the configuration
tells it to. For a configuration outside this repository, those commands are
skipped and `compat list` says so. Only the four that evaluate no hooks —
`version`, `status`, `doctor --json`, `dep list` — run against one.

The corpus configuration is exempt, because its hooks are in this repository
and touch nothing outside the sandbox.

## What is not recorded

A hook runs real commands, and those commands write their own state: Homebrew's
API cache, `gh`'s device id. Directories named `.cache`, `Caches`, or `state`
are recorded as existing and not descended into, so the corpus notices one
appearing or disappearing without comparing a third-party tool's bookkeeping.

That hides a real effect if meowctl ever writes into one of them. The
configuration directory, where meowctl actually writes, is never excluded.

## Fixtures are per platform

`fixtures/<os>-<arch>/`. The corpus compares two binaries on one machine, not
two machines. A `select` on the platform produces different files on macOS and
Linux, so a macOS recording checked on Linux would report platform differences
as rewrite defects.

## The cases

`configs/smoke` is a self-contained configuration that needs no network and
touches nothing outside its sandbox. It covers a dependency graph with an
ordering hint, a platform `select`, a package manager declared as a component,
file operations that journal an inverse, and a shell hook.

A real configuration can be added for a local run:

```sh
MEOWCTL_COMPAT_EXTRA=~/.config/meowctl mise run compat-self-check
```

It is copied into the sandbox and never written to in place. It is not
committed, because a real dotfiles configuration is personal and changes under
its owner.
