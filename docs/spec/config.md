# Configuration and on-disk formats

**Crate:** `meowctl-config`
**Design:** `docs/design/0.2.0-rust-rewrite.md` §3, defects #5 and #6
**v0.1.0 equivalent:** `internal/lock/`, `internal/modfile/`,
`internal/state/`, `internal/rewrite/`, and the lockfile code in
`internal/cli/apply.go`

## Scope

This component owns every file meowctl reads or writes in the config directory:
the schemas, the parsers, the writers, the schema versions, and the
syntax-aware editor that changes a Starlark file without reformatting it. No
other crate parses one of these formats.

It exists because in `v0.1.0` the formats are spread across five packages and
two of them are defined by ad-hoc readers inside the CLI layer. `installed.lock`
has two schema versions and the migration lives in `apply.go`; `pkgs.lock` is
read and written by helpers next to the cobra commands.

Parity is strictest here. These files are shared between two binaries during
the rewrite, and a serializer that reorders keys breaks reproducibility without
breaking a test.

## Boundary

The files, their schemas, their versions, and the editor. The directory is
resolved by `meowctl-common`; see [R-COMMON-020].

## The files

**[R-CONFIG-001]** The component MUST own exactly these names, as
`internal/cli/config.go` declares them: `init.star`, `local.star`, `deps.mod`,
`deps.lock`, `deps.local.mod`, `deps.local.lock`, `state.toml`,
`installed.lock`, `pkgs.lock`, and `pkgs.local.lock`.

**[R-CONFIG-002]** Every write MUST be atomic through `FileSystem`; see
[R-FS-003]. These files are read by the next run and by a second binary, and a
partial one is indistinguishable from a valid one that lost data.

`v0.1.0` is atomic for `deps.lock` and `state.toml` and is not for `deps.mod`,
which `internal/modfile/modfile.go` writes with a plain `os.WriteFile`. Making
all of them atomic is a deliberate change: a `deps.mod` truncated by a crash
during `meowctl dep add` leaves a configuration that does not parse.

**[R-CONFIG-003]** A file that does not exist MUST parse as its empty value
where `v0.1.0` treats it that way: an absent lock file is a clean slate, an
absent `state.toml` is a first run, an absent `local.star` is no local
components. An absent `init.star` MUST be an error.

**[R-CONFIG-004]** A configuration directory holding `meowctl.star` but no
`init.star` MUST be reported as the pre-rename layout, with the three `mv`
commands that migrate it, and MUST exit with the configuration code.
`reportLegacyConfig` in `internal/cli/commands.go` is the message.

## Declarations

**[R-CONFIG-010]** `init.star`, `local.star` and `deps.mod` MUST be evaluated
by [`meowctl-starlark`](starlark.md). This component never evaluates one: a
second implementation of the language is a second set of semantics, and the two
drift. It owns what the resulting declarations mean on disk, not how they are
produced.

It does parse them, for [R-CONFIG-050] and [R-CONFIG-051], which change one
declaration without disturbing the rest of the file. Parsing to locate a
statement is not evaluating it, and the two need different things: editing
needs the syntax tree, evaluation needs the builtins and the accumulator.

Both MUST use the same dialect. A file one accepts and the other rejects would
mean `meowctl add` succeeding on a configuration that then fails to apply, so
the dialect is named in both crates and a test runs one file through both.

**[R-CONFIG-011]** `deps.mod` MUST support the statements
`internal/modfile/modfile.go` documents: `module(name, version)`,
`dep(name, version)` for a registry dependency, `dep(name, source)` for a
GitHub dependency, and `replace(name, path)` or `replace(name, source)`.

A module's own `MODULE.meow` uses the same statements plus `compat` on
`module()`; see [R-STAR-006]. meowctl writes `deps.mod` and never writes a
`MODULE.meow`, so this component's writer emits no `compat`.

**[R-CONFIG-012]** A `dep()` MUST carry a version or a source, never both and
never neither.

**[R-CONFIG-013]** Writing `deps.mod` MUST emit keyword arguments in the order
`name`, then `version` or `source`. `v0.1.0` requires this order for its
regular-expression rewriter; here it is required so the two binaries produce
identical files.

**[R-CONFIG-014]** A written `deps.mod` MUST reproduce `modfile.Write` byte for
byte: the header comment, `module()` across four lines with indented keyword
arguments, each `dep()` on one line, a blank line after the dependency block,
and each `replace()` on one line.

The header `v0.1.0` writes names the file `meowctl.mod`, which the rename to
`deps.mod` left behind. The text MUST be reproduced as it is, because the file
is compared byte for byte; correcting it is a `0.3.0` change that costs a
corpus rebaseline and buys a comment nobody reads.

## Lock files

**[R-CONFIG-020]** `deps.lock` MUST be TOML with the four tables
`internal/lock/lock.go` defines: `meta`, `modules`, `github-modules`, and
`packages`. A module entry MUST carry `version`, `source`, `integrity`,
`files`, and optionally `commit-sha`, `replaced`, and `path`, under exactly
those key names and in that order.

A `github-modules` entry MUST carry `commit` and `integrity`. A `packages`
entry is keyed by manager, then by package, and MUST carry `requested`,
`installed`, and optionally `note`.

**[R-CONFIG-021]** `files`, when present, MUST map each extracted path,
relative to the module root, to its own integrity hash. Nothing writes it:
`v0.1.0` declares the table and never fills it, and `v0.2.0` records the
per-file hashes in the cache instead, for the reasons in [R-MODULE-044]. The
schema keeps it because a lock either binary wrote has to round-trip through
the other.

**[R-CONFIG-022]** `meta` MUST record the version that wrote the file and an
RFC 3339 timestamp.

`v0.1.0` declares both fields and sets neither, so every lock it has ever
written carries two empty strings. Filling them is a deliberate change, and it
is the one place a user can see which binary last touched their lock -- which
matters most during a cutover, when two of them share a configuration
directory. It is also why [R-CONFIG-023] excludes this table.

**[R-CONFIG-023]** A lock file written by `v0.2.0` from the same resolution as
`v0.1.0` MUST be byte-identical outside the `meta` table, including key order
and table order. A lock `v0.1.0` wrote is checked in beside the resolver
test that reproduces it.

`meta` is excluded because it is the one table the two binaries are meant to
disagree about; see [R-CONFIG-022]. Everything a run depends on -- versions,
sources, hashes, commits, packages -- is inside the comparison.

**[R-CONFIG-024]** `deps.local.lock` MUST have the same schema as `deps.lock`
and MUST be read as an overlay: an entry present in both wins from the local
file.

**[R-CONFIG-025]** `pkgs.lock` and `pkgs.local.lock` MUST record resolved
package installations per manager, and a component declared in `local.star`
MUST have its packages recorded in the local file rather than the shared one.
`appendPkgsLock` in `internal/cli/apply.go` is the split.

## Installed components

**[R-CONFIG-030]** `installed.lock` MUST record each installed component with
the fingerprint of the module it was installed from, under schema version 2.

**[R-CONFIG-031]** The schema version 1 form, a bare `components = [names]`
array, MUST still parse, with every version treated as unknown.
`installedLock.versionMap` in `internal/cli/apply.go` handles both, and a user
upgrading from an earlier build has the older form on disk.

**[R-CONFIG-032]** Writing MUST always produce schema version 2, sorted by
component name, so the file does not churn between runs.

**[R-CONFIG-033]** A module's fingerprint MUST be its resolved semver version
for a registry module, its commit SHA for a GitHub module, and its integrity
hash when neither is available. `moduleFingerprint` is the fallback chain, and
it exists so that a module bumped without a version change still invalidates.

## Sentinel state

**[R-CONFIG-040]** `state.toml` MUST hold `schema_version`, a `last_run` table,
a `completed_components` array, and an optional `repo_url`, matching
`internal/state/state.go`.

**[R-CONFIG-041]** A `state.toml` whose `schema_version` is higher than this
build understands MUST be reported as written by a newer meowctl and MUST NOT
be overwritten. `v0.1.0` ignores the field entirely, which means an older
binary silently rewrites a newer file and loses whatever it did not understand.

**[R-CONFIG-042]** `last_run` MUST record the phase set, the start time in UTC,
whether the run completed, and the rollback outcome.

**[R-CONFIG-043]** `completed_components` MUST be append-only within a run and
MUST record the phase, the component, and a UTC timestamp for each.

**[R-CONFIG-044]** The rollback outcome MUST be one of the empty string, `ok`,
`partial`, or `failed`, matching the `RolledBack` constants and
[R-OPS-024].

## The Starlark editor

**[R-CONFIG-050]** Adding or removing a `component()` declaration in
`init.star` or `local.star` MUST preserve every comment, every blank line, and
the formatting of every statement it does not touch.

The file belongs to the user. A round trip through a parsed model and a printer
would lose the comments they wrote and the grouping they chose, so the edit is
made to the text at the position the parse reports.

**[R-CONFIG-051]** Changing a `dep()` version in `deps.mod` MUST work
regardless of the order the keyword arguments are written in.
`internal/rewrite/rewrite.go` matches a regular expression that requires `name`
before `version` and documents the limitation; a file a user hand-edited into
the other order is silently not found. The editor MUST parse the file rather
than match text.

**[R-CONFIG-052]** An edit that finds no matching declaration MUST report it
rather than write the file unchanged.

**[R-CONFIG-053]** Removing the last `component()` from a file MUST leave a
valid file rather than an empty one, because the next `add` has to have
somewhere to write.

## Failure paths

**[R-CONFIG-060]** A malformed TOML file MUST produce an error naming the file
and the line, and MUST exit with the configuration code; see [R-COMMON-030].

**[R-CONFIG-061]** A lock file that names a module with an integrity hash in a
form [R-COMMON-004] rejects MUST fail to parse rather than be treated as
unverified.

**[R-CONFIG-062]** A write that fails MUST leave the previous file intact; see
[R-FS-003] and [R-FS-031].

**[R-CONFIG-063]** Two concurrent runs writing the same file MUST NOT interleave
into a corrupt result. The atomic rename gives this; nothing depends on a lock.

## Parity with v0.1.0

Every schema, key name, and default in this spec is read from the Go source:
`internal/lock/lock.go` for the lock schema, `internal/modfile/modfile.go` for
`deps.mod`, `internal/state/state.go` for the sentinel, and
`internal/cli/apply.go` for `installed.lock` and `pkgs.lock`, which have no
package of their own.

Four requirements deliberately change behaviour. [R-CONFIG-002] makes
`deps.mod` writes atomic, where `internal/modfile/modfile.go` uses a plain
`os.WriteFile`. [R-CONFIG-041] refuses to overwrite a newer `state.toml`, where
`v0.1.0` overwrites it. [R-CONFIG-051] replaces regular-expression editing with
parsing, which removes the documented keyword-order limitation. [R-CONFIG-052]
reports a missed edit, where `AppendComponent` in `internal/rewrite/rewrite.go`
can append a duplicate.
