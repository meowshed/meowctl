---
id: SPC-1200
artifact: spec
status: live
revised: 2026-09-27
checked-at:
states: [REQ-1201, REQ-1202, REQ-1203, REQ-1204, REQ-1210, REQ-1211, REQ-1212, REQ-1213, REQ-1214, REQ-1220, REQ-1221, REQ-1222, REQ-1223, REQ-1224, REQ-1225, REQ-1226, REQ-1230, REQ-1231, REQ-1232, REQ-1233, REQ-1240, REQ-1241, REQ-1242, REQ-1243, REQ-1244, REQ-1250, REQ-1251, REQ-1252, REQ-1253, REQ-1260, REQ-1261, REQ-1262, REQ-1263, REQ-1264, REQ-1300, REQ-1301, REQ-1302, REQ-1303, REQ-1304, REQ-1305, REQ-1306, REQ-1307, REQ-1308, REQ-1309, REQ-1310, REQ-1311, REQ-1312, REQ-1313, REQ-1314, REQ-1315, REQ-1316]
---

# Configuration files and their on-disk formats

## Scope

This part is the `meowctl-config` crate, designed in
`docs/design/0.2.0-rust-rewrite.md` §3 against defects #5 and #6, and replaces
`internal/lock/`, `internal/modfile/`, `internal/state/`, `internal/rewrite/`
and the lockfile code in `internal/cli/apply.go` from `v0.1.0` (from
docs/spec/config.md, high).

This component owns every file meowctl reads or writes in the config directory:
the schemas, the parsers, the writers, the schema versions, and the syntax-aware
editor that changes a Starlark file without reformatting it. No other crate
parses one of these formats (from docs/spec/config.md, high).

Parity is strictest here, because a serializer that reorders keys breaks
reproducibility without breaking a test (from docs/spec/config.md, high).

## Boundary

The files, their schemas, their versions, and the editor. The directory is
resolved by `meowctl-common`; see [REQ-1020] (from docs/spec/config.md, high).

| File | What it holds |
| --- | --- |
| `init.star`, `local.star` | Component declarations, evaluated by `meowctl-starlark` and edited here (from docs/spec/config.md, high) |
| `deps.mod`, `deps.local.mod` | Module and dependency declarations (from docs/spec/config.md, high) |
| `deps.lock`, `deps.local.lock` | Resolved modules, GitHub modules and packages (from docs/spec/config.md, high) |
| `pkgs.lock`, `pkgs.local.lock` | Packages declared by components that ran, per manager (from docs/spec/config.md, high) |
| `installed.lock` | Installed components and the fingerprint of the module each came from (from docs/spec/config.md, high) |
| `state.toml` | The sentinel state of the last run (from docs/spec/config.md, high) |
| `.hook-error` | The last shell-spawn failure (from docs/spec/config.md, high) |

## Behaviour

### The files

The component owns exactly the names `init.star`, `local.star`, `deps.mod`,
`deps.lock`, `deps.local.mod`, `deps.local.lock`, `state.toml`,
`installed.lock`, `pkgs.lock`, `pkgs.local.lock` and `.hook-error` [REQ-1201]
(from docs/spec/config.md, high).

Every write goes through `FileSystem` atomically [REQ-1202] (from
docs/spec/config.md, high).

An absent lock file parses as a clean slate, an absent `state.toml` as a first
run, and an absent `local.star` as no local components [REQ-1203] (from
docs/spec/config.md, high).

A configuration directory holding `meowctl.star` but no `init.star` is reported
as the pre-rename layout, with the three `mv` commands that migrate it
[REQ-1204], and exits with the configuration code [REQ-1301] (from
docs/spec/config.md, high).

### Declarations

`meowctl-starlark` evaluates `init.star`, `local.star` and `deps.mod`, and this
component never evaluates one [REQ-1210] (from docs/spec/config.md, high). The
parser this component uses to edit a declaration and `meowctl-starlark` use the
same dialect [REQ-1302] (from docs/spec/config.md, high).

`deps.mod` supports `module(name, version)`, `dep(name, version)` for a registry
dependency, `dep(name, source)` for a GitHub dependency, and
`replace(name, path)` or `replace(name, source)` [REQ-1211] (from
docs/spec/config.md, high).

A `dep()` carries a version or a source, and a `replace()` a path or a source,
never both and never neither [REQ-1212] (from docs/spec/config.md, high).

Writing `deps.mod` emits keyword arguments in the order `name`, then `version`
or `source` [REQ-1213] (from docs/spec/config.md, high).

A written `deps.mod` reproduces `modfile.Write` byte for byte: the header
comment, `module()` across four lines with indented keyword arguments, each
`dep()` on one line, a blank line after the dependency block, and each
`replace()` on one line [REQ-1214] (from docs/spec/config.md, high). The header
names the file `meowctl.mod`, as `v0.1.0` writes it [REQ-1304] (from
docs/spec/config.md, high).

### Lock files

`deps.lock` is TOML with the four tables `meta`, `modules`, `github-modules` and
`packages` [REQ-1220] (from docs/spec/config.md, high).

A module entry carries `version`, `source`, `integrity`, `files`, and optionally
`commit-sha`, `replaced` and `path`, under exactly those key names and in that
order [REQ-1305] (from docs/spec/config.md, high).

A `github-modules` entry carries `commit` and `integrity` [REQ-1306] (from
docs/spec/config.md, high).

A `packages` entry is keyed by manager, then by package, and carries
`requested`, `installed`, and optionally `note` [REQ-1307] (from
docs/spec/config.md, high).

`files`, when present, maps each extracted path, relative to the module root, to
its own integrity hash, and nothing writes it [REQ-1221] (from
docs/spec/config.md, high).

`meta` records the version that wrote the file and an RFC 3339 timestamp
[REQ-1222] (from docs/spec/config.md, high).

A lock file written by `v0.2.0` from the same resolution as `v0.1.0` is
byte-identical outside the `meta` table, including key order and table order
[REQ-1223] (from docs/spec/config.md, high).

`deps.local.lock` has the same schema as `deps.lock` [REQ-1224] and is read as
an overlay, where an entry present in both files wins from the local file
[REQ-1308] (from docs/spec/config.md, high).

`pkgs.lock` and `pkgs.local.lock` record the packages declared by every
component whose `install` or `upgrade` ran, per manager [REQ-1225], and a
component declared in `local.star` has its packages recorded in the local file
[REQ-1309] (from docs/spec/config.md, high).

A run that declared no packages leaves both package locks alone [REQ-1226]; a
run that declared some merges into what is there [REQ-1310], and an entry for a
package no component declares any more survives [REQ-1311] (from
docs/spec/config.md, high).

The package locks aren't byte-compared against `v0.1.0`, because only
`deps.lock` falls under [REQ-1223] (from docs/spec/config.md, high).

### Installed components

`installed.lock` records each installed component with the fingerprint of the
module it was installed from, under schema version 2 [REQ-1230] (from
docs/spec/config.md, high).

The schema version 1 form, a bare `components = [names]` array, still parses,
with every version treated as unknown [REQ-1231] (from docs/spec/config.md,
high).

Writing always produces schema version 2, sorted by component name [REQ-1232]
(from docs/spec/config.md, high).

A module's fingerprint is its resolved semver version for a registry module, its
commit SHA for a GitHub module, and its integrity hash when neither is available
[REQ-1233] (from docs/spec/config.md, high).

### Sentinel state

`state.toml` holds `schema_version`, a `last_run` table, a
`completed_components` array, and an optional `repo_url` [REQ-1240] (from
docs/spec/config.md, high).

`last_run` records the phase set, the start time in UTC, whether the run
completed, and the rollback outcome [REQ-1242] (from docs/spec/config.md, high).

`completed_components` is append-only within a run [REQ-1243], and records the
phase, the component and a UTC timestamp for each entry [REQ-1313] (from
docs/spec/config.md, high).

The rollback outcome is one of the empty string, `ok`, `partial` or `failed`
[REQ-1244] (from docs/spec/config.md, high).

### The hook error flag

`.hook-error` holds an RFC 3339 timestamp on the first line and the failure on
the second [REQ-1264], and is written with mode `0o600` [REQ-1316] (from
docs/spec/config.md, high).

### The Starlark editor

Adding or removing a `component()` declaration in `init.star` or `local.star`
preserves every comment, every blank line, and the formatting of every statement
it doesn't touch [REQ-1250] (from docs/spec/config.md, high).

Changing a `dep()` version in `deps.mod` works whatever order the keyword
arguments are written in [REQ-1251], because the editor parses the file and
doesn't match text [REQ-1314] (from docs/spec/config.md, high).

Removing the last `component()` from a file leaves a valid file, not an empty
one [REQ-1253] (from docs/spec/config.md, high).

## Failure paths

| Condition | What happens |
| --- | --- |
| `init.star` is absent | Reading the configuration is an error [REQ-1300] (from docs/spec/config.md, high) |
| The directory holds `meowctl.star` and no `init.star` | The pre-rename layout is reported with its migration, and the process exits with the configuration code [REQ-1204] [REQ-1301] (from docs/spec/config.md, high) |
| A `deps.mod` entry carries both a version and a source, or neither, or a `replace()` both a path and a source, or neither | Reading the file fails, naming the entry [REQ-1303] (from docs/spec/config.md, high) |
| A TOML file is malformed | The error names the file and the line [REQ-1260], and the process exits with the configuration code [REQ-1315] (from docs/spec/config.md, high) |
| A lock file names a module with an integrity hash in a form [REQ-1004] rejects | The lock fails to parse and isn't treated as unverified [REQ-1261] (from docs/spec/config.md, high) |
| `state.toml` carries a `schema_version` higher than this build understands | The file is reported as written by a newer meowctl [REQ-1241], and isn't overwritten [REQ-1312] (from docs/spec/config.md, high) |
| An edit finds no matching declaration | The editor reports it and doesn't write the file unchanged [REQ-1252] (from docs/spec/config.md, high) |
| A write fails | The file it was replacing stays intact [REQ-1262] (from docs/spec/config.md, high) |
| Two concurrent runs write the same file | The atomic rename keeps the writes from interleaving into a corrupt result [REQ-1263] (from docs/spec/config.md, high) |
