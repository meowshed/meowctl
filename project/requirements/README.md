# Requirements

<!-- markdownlint-disable MD013 -->

<!-- meow-method index -->

556 requirements in all: 554 approved, 2 withdrawn.

| Identifier | What it requires | Status |
| --- | --- | --- |
| [REQ-1001](REQ-1001-componentid-accepts-three-forms.md) | A `ComponentId` MUST be constructible from the three forms `v0.1.0` accepts: a bare name (`neovim`), a registry-qualified path (`@stdlib//components/zsh`), and a GitHub-qualified path (`github.com/owner/repo//components/zsh`). | approved |
| [REQ-1002](REQ-1002-componentid-exposes-module-key.md) | A `ComponentId` MUST expose the module key its source belongs to: `dotmeow` for `@dotmeow//path`, `github.com/o/r` for `github.com/o/r//path`, and nothing for a bare name. | approved |
| [REQ-1003](REQ-1003-moduleref-distinguishes-registry-github.md) | A `ModuleRef` MUST distinguish a registry module, named by a bare identifier, from a GitHub module, named `github:owner/repo@ref`. | approved |
| [REQ-1004](REQ-1004-integrity-holds-sri-hash.md) | An `Integrity` MUST hold a W3C Subresource Integrity hash in the form `sha384-<base64>`. | approved |
| [REQ-1005](REQ-1005-integrity-computation-lives-with-type.md) | Computing an `Integrity` from bytes MUST live with the type: SHA-384 in standard base64, which is what `computeSRI` in `internal/starlark/loader/github.go` produces and what every published `index.toml` carries. | approved |
| [REQ-1006](REQ-1006-componentid-exposes-logical-name.md) | A `ComponentId` MUST expose its logical name: the last path segment of a qualified form, and the whole of a bare one. | approved |
| [REQ-1010](REQ-1010-phase-has-thirteen-variants.md) | `Phase` MUST have exactly the thirteen variants `v0.1.0` defines in `internal/lifecycle/runner.go`: `install_check`, `install`, `install_configure`, `update`, `upgrade_check`, `upgrade`, `upgrade_configure`, `uninstall_check`, `uninstall`, `uninstall_cleanup`, `shell`, `login`, and `verify`. | approved |
| [REQ-1011](REQ-1011-phaseset-yields-phases-in-order.md) | `PhaseSet` MUST have the five variants `install`, `update`, `upgrade`, `uninstall`, and `verify`, each yielding its phases in the order `internal/lifecycle/runner.go` gives: `install` yields `install_check`, `install`, `install_configure`; `update` yields `update`; `upgrade` yields `upgrade_check`, `upgrade`, `upgrade_configure`; `uninstall` yields `uninstall_check`, `uninstall`, `uninstall_cleanup`; `verify` yields `verify`. | approved |
| [REQ-1012](REQ-1012-phase-reports-read-only.md) | `Phase` MUST report whether it is read-only. The read-only phases are `install_check`, `upgrade_check`, `uninstall_check`, and `verify`. | approved |
| [REQ-1013](REQ-1013-phase-reports-runtime-hook.md) | `Phase` MUST report whether it is a runtime hook phase. The runtime hook phases are `shell` and `login`. | approved |
| [REQ-1020](REQ-1020-config-directory-resolution-order.md) | The config directory MUST resolve to `$MEOWCTL_CONFIG` when set, then `$XDG_CONFIG_HOME/meowctl`, then `~/.config/meowctl`, in that order. The `--config` flag overrides all of them; see [REQ-3404]. | approved |
| [REQ-1021](REQ-1021-module-cache-honours-xdg.md) | The module cache directory MUST resolve to `$XDG_CACHE_HOME/meowctl/modules`, falling back to `~/.cache/meowctl/modules`. | approved |
| [REQ-1022](REQ-1022-path-resolution-expands-tilde.md) | Path resolution MUST expand a leading `~` to the home directory, matching `expandPath` in `internal/ctx/methods.go`. | approved |
| [REQ-1023](REQ-1023-home-directory-from-environment.md) | The home directory MUST come from `$HOME`, and from `$USERPROFILE` when `$HOME` is unset on Windows. | approved |
| [REQ-1030](REQ-1030-errors-map-to-v010-exit-codes.md) | The error taxonomy MUST map onto the exit codes `v0.1.0` defines in `internal/cli/errors.go`, and no others: 0 for success, 1 for a general error, 2 for a usage error, 3 for a configuration error (a malformed `init.star`, a missing component, a legacy layout), and 4 for a module error (a fetch that failed, an integrity hash that did not match). | approved |
| [REQ-1031](REQ-1031-error-enums-convert-to-exit-code.md) | Every crate's error enum MUST be convertible to an exit code. | approved |
| [REQ-1032](REQ-1032-errors-carry-source-span.md) | An error that has a source location MUST carry it as a span into the file it came from, so `meowctl-cli` can render a diagnostic that points at the line. | approved |
| [REQ-1040](REQ-1040-event-covers-rendered-output.md) | The `Event` enum MUST cover everything a command produces that a sink renders. At minimum: `PlanComputed`, `PhaseStarted`, `PhaseFinished`, `ComponentStarted`, `ComponentSkipped`, `ComponentFinished`, `OpApplied`, `ProcessStarted`, `ProcessOutput`, `ProcessFinished`, `TerminalRequested`, `TerminalReleased`, `Diagnostic`, and `Message { severity, text }`. | approved |
| [REQ-1041](REQ-1041-event-serialises-to-json.md) | An `Event` MUST be serializable to JSON, because `JsonSink` emits one object per event; see [REQ-3230]. | approved |
| [REQ-1042](REQ-1042-message-sole-unstructured-variant.md) | `Message` MUST be the only variant carrying unstructured text. | approved |
| [REQ-1043](REQ-1043-event-carries-no-presentation.md) | `Event` MUST NOT carry a rendered string, a colour, a glyph, or a width. | approved |
| [REQ-1044](REQ-1044-event-includes-shell-effects.md) | The `Event` set MUST include `ShellLine` and `PathPrepended`, which `ctx.emit` and `ctx.add_path` produce. | approved |
| [REQ-1050](REQ-1050-identifier-parsing-never-panics.md) | Parsing any identifier from an untrusted string MUST return an error rather than panic. | approved |
| [REQ-1051](REQ-1051-unknown-phase-string-fails.md) | Constructing a `Phase` from a string that names no phase MUST fail. | approved |
| [REQ-1100](REQ-1100-componentid-display-reproduces-input.md) | The `Display` of a `ComponentId` MUST reproduce the input exactly. | approved |
| [REQ-1101](REQ-1101-moduleref-rejects-other-strings.md) | Parsing a string that is neither a registry module nor a GitHub module as a `ModuleRef` MUST fail rather than produce a registry module with a strange name. | approved |
| [REQ-1102](REQ-1102-integrity-rejects-non-sri-string.md) | An `Integrity` MUST reject a string that is not a W3C Subresource Integrity hash in the form `sha384-<base64>`. | approved |
| [REQ-1103](REQ-1103-phase-string-form-matches-names.md) | The string form of `Phase` MUST match the names of the thirteen `v0.1.0` phases exactly, because they are the hook names a component author writes and they appear in `state.toml`. | approved |
| [REQ-1104](REQ-1104-path-resolution-rejects-relative-paths.md) | Path resolution MUST reject a path that is not absolute once expanded. | approved |
| [REQ-1105](REQ-1105-path-resolution-rejects-escaping-paths.md) | A path that escapes the directory it was resolved against MUST be rejected rather than silently normalised. | approved |
| [REQ-1106](REQ-1106-missing-home-is-an-error.md) | Neither `$HOME` nor `$USERPROFILE` being set MUST be an error rather than a guess. | approved |
| [REQ-1107](REQ-1107-one-place-chooses-exit-code.md) | The conversion from an error enum to an exit code MUST be the only place a code is chosen. | approved |
| [REQ-1108](REQ-1108-lower-crates-name-no-exit-code.md) | A crate below `meowctl-cli` MUST NOT choose the exit code for an error it returns. | approved |
| [REQ-1109](REQ-1109-message-carries-severity.md) | `Message` MUST carry a severity. | approved |
| [REQ-1110](REQ-1110-distinct-rendering-needs-own-variant.md) | Anything a sink needs to render differently from other text MUST be its own variant rather than a formatted string. | approved |
| [REQ-1201](REQ-1201-config-owns-eleven-file-names.md) | The component MUST own exactly these names, as `internal/cli/config.go` declares them: `init.star`, `local.star`, `deps.mod`, `deps.lock`, `deps.local.mod`, `deps.local.lock`, `state.toml`, `installed.lock`, `pkgs.lock`, `pkgs.local.lock`, and `.hook-error`. | approved |
| [REQ-1202](REQ-1202-config-writes-are-atomic.md) | Every write MUST be atomic through `FileSystem`; see [REQ-1403]. | approved |
| [REQ-1203](REQ-1203-absent-file-parses-as-empty.md) | A file that does not exist MUST parse as its empty value where `v0.1.0` treats it that way: an absent lock file is a clean slate, an absent `state.toml` is a first run, an absent `local.star` is no local components. | approved |
| [REQ-1204](REQ-1204-legacy-layout-reported-with-migration.md) | A configuration directory holding `meowctl.star` but no `init.star` MUST be reported as the pre-rename layout, with the three `mv` commands that migrate it. `reportLegacyConfig` in `internal/cli/commands.go` is the message. | approved |
| [REQ-1210](REQ-1210-declarations-evaluated-by-starlark-crate.md) | `init.star`, `local.star` and `deps.mod` MUST be evaluated by `meowctl-starlark`. This component never evaluates one. | approved |
| [REQ-1211](REQ-1211-deps-mod-supports-v010-statements.md) | `deps.mod` MUST support the statements `internal/modfile/modfile.go` documents: `module(name, version)`, `dep(name, version)` for a registry dependency, `dep(name, source)` for a GitHub dependency, and `replace(name, path)` or `replace(name, source)`. | approved |
| [REQ-1212](REQ-1212-dep-carries-version-or-source.md) | A `dep()` MUST carry a version or a source, never both and never neither, and a `replace()` a path or a source on the same terms. | approved |
| [REQ-1213](REQ-1213-deps-mod-keyword-order.md) | Writing `deps.mod` MUST emit keyword arguments in the order `name`, then `version` or `source`. | approved |
| [REQ-1214](REQ-1214-deps-mod-matches-modfile-write.md) | A written `deps.mod` MUST reproduce `modfile.Write` byte for byte: the header comment, `module()` across four lines with indented keyword arguments, each `dep()` on one line, a blank line after the dependency block, and each `replace()` on one line. | approved |
| [REQ-1220](REQ-1220-deps-lock-four-tables.md) | `deps.lock` MUST be TOML with the four tables `internal/lock/lock.go` defines: `meta`, `modules`, `github-modules`, and `packages`. | approved |
| [REQ-1221](REQ-1221-files-table-maps-path-hashes.md) | `files`, when present, MUST map each extracted path, relative to the module root, to its own integrity hash. | approved |
| [REQ-1222](REQ-1222-lock-meta-records-writer.md) | `meta` MUST record the version that wrote the file and an RFC 3339 timestamp. | approved |
| [REQ-1223](REQ-1223-lock-byte-identical-outside-meta.md) | A lock file written by `v0.2.0` from the same resolution as `v0.1.0` MUST be byte-identical outside the `meta` table, including key order and table order. | approved |
| [REQ-1224](REQ-1224-local-lock-shares-schema.md) | `deps.local.lock` MUST have the same schema as `deps.lock`. | approved |
| [REQ-1225](REQ-1225-pkgs-lock-records-declared-packages.md) | `pkgs.lock` and `pkgs.local.lock` MUST record the packages declared by every component whose `install` or `upgrade` ran, per manager. | approved |
| [REQ-1226](REQ-1226-no-packages-leaves-locks-alone.md) | A run that declared no packages MUST leave both `pkgs.lock` and `pkgs.local.lock` alone. | approved |
| [REQ-1227](REQ-1227-package-locks-not-byte-compared.md) | Replaced by REQ-1223. | withdrawn |
| [REQ-1230](REQ-1230-installed-lock-records-fingerprints.md) | `installed.lock` MUST record each installed component with the fingerprint of the module it was installed from, under schema version 2. | approved |
| [REQ-1231](REQ-1231-installed-lock-v1-still-parses.md) | The schema version 1 form of `installed.lock`, a bare `components = [names]` array, MUST still parse, with every version treated as unknown. | approved |
| [REQ-1232](REQ-1232-installed-lock-writes-sorted-v2.md) | Writing `installed.lock` MUST always produce schema version 2, sorted by component name, so the file does not churn between runs. | approved |
| [REQ-1233](REQ-1233-module-fingerprint-fallback-chain.md) | A module's fingerprint MUST be its resolved semver version for a registry module, its commit SHA for a GitHub module, and its integrity hash when neither is available. `moduleFingerprint` is the fallback chain. | approved |
| [REQ-1240](REQ-1240-state-toml-schema.md) | `state.toml` MUST hold `schema_version`, a `last_run` table, a `completed_components` array, and an optional `repo_url`, matching `internal/state/state.go`. | approved |
| [REQ-1241](REQ-1241-newer-state-reported.md) | A `state.toml` whose `schema_version` is higher than this build understands MUST be reported as written by a newer meowctl. | approved |
| [REQ-1242](REQ-1242-last-run-records-outcome.md) | `last_run` MUST record the phase set, the start time in UTC, whether the run completed, and the rollback outcome. | approved |
| [REQ-1243](REQ-1243-completed-components-append-only.md) | `completed_components` MUST be append-only within a run. | approved |
| [REQ-1244](REQ-1244-rollback-outcome-values.md) | The rollback outcome MUST be one of the empty string, `ok`, `partial`, or `failed`, matching the `RolledBack` constants and [REQ-2024]. | approved |
| [REQ-1250](REQ-1250-component-edit-preserves-formatting.md) | Adding or removing a `component()` declaration in `init.star` or `local.star` MUST preserve every comment, every blank line, and the formatting of every statement it does not touch. | approved |
| [REQ-1251](REQ-1251-dep-version-edit-any-order.md) | Changing a `dep()` version in `deps.mod` MUST work regardless of the order the keyword arguments are written in. | approved |
| [REQ-1252](REQ-1252-missed-edit-is-reported.md) | An edit that finds no matching declaration MUST report it rather than write the file unchanged. | approved |
| [REQ-1253](REQ-1253-last-component-removal-stays-valid.md) | Removing the last `component()` from a file MUST leave a valid file rather than an empty one, because the next `add` has to have somewhere to write. | approved |
| [REQ-1260](REQ-1260-malformed-toml-names-file-line.md) | A malformed TOML file MUST produce an error naming the file and the line. | approved |
| [REQ-1261](REQ-1261-malformed-lock-hash-fails-parse.md) | A lock file that names a module with an integrity hash in a form [REQ-1004] rejects MUST fail to parse rather than be treated as unverified. | approved |
| [REQ-1262](REQ-1262-failed-write-keeps-replaced-file.md) | A write that fails MUST leave the file it was replacing intact; see [REQ-1403] and [REQ-1431]. | approved |
| [REQ-1263](REQ-1263-concurrent-writes-never-interleave.md) | Two concurrent runs writing the same file MUST NOT interleave into a corrupt result. | approved |
| [REQ-1264](REQ-1264-hook-error-flag-format.md) | `.hook-error` MUST hold an RFC 3339 timestamp on the first line and the failure on the second. | approved |
| [REQ-1300](REQ-1300-absent-init-star-is-error.md) | An absent `init.star` MUST be an error. | approved |
| [REQ-1301](REQ-1301-legacy-layout-exits-configuration-code.md) | A configuration directory holding `meowctl.star` but no `init.star` MUST exit with the configuration code. | approved |
| [REQ-1302](REQ-1302-editor-and-evaluator-share-dialect.md) | The parser this component uses to edit a declaration and `meowctl-starlark` MUST use the same dialect. | approved |
| [REQ-1303](REQ-1303-invalid-deps-mod-entry-fails.md) | Reading a `deps.mod` that breaks either rule MUST fail, naming the entry. | approved |
| [REQ-1304](REQ-1304-deps-mod-header-kept-verbatim.md) | The header comment of a written `deps.mod`, which names the file `meowctl.mod`, MUST be reproduced as it is. | approved |
| [REQ-1305](REQ-1305-module-entry-keys-and-order.md) | A `deps.lock` module entry MUST carry `version`, `source`, `integrity`, `files`, and optionally `commit-sha`, `replaced`, and `path`, under exactly those key names and in that order. | approved |
| [REQ-1306](REQ-1306-github-module-entry-keys.md) | A `github-modules` entry MUST carry `commit` and `integrity`. | approved |
| [REQ-1307](REQ-1307-packages-entry-keys.md) | A `packages` entry is keyed by manager, then by package, and MUST carry `requested`, `installed`, and optionally `note`. | approved |
| [REQ-1308](REQ-1308-local-lock-read-as-overlay.md) | `deps.local.lock` MUST be read as an overlay: an entry present in both wins from the local file. | approved |
| [REQ-1309](REQ-1309-local-component-packages-go-local.md) | A component declared in `local.star` MUST have its packages recorded in the local file rather than the shared one. `appendPkgsLock` in `internal/cli/apply.go` is the split. | approved |
| [REQ-1310](REQ-1310-package-lock-merges-entries.md) | A run that declared some packages MUST merge into what is in `pkgs.lock` and `pkgs.local.lock` rather than replacing it. | approved |
| [REQ-1311](REQ-1311-undeclared-package-entry-survives.md) | An entry for a package no component declares any more MUST survive. | approved |
| [REQ-1312](REQ-1312-newer-state-not-overwritten.md) | A `state.toml` whose `schema_version` is higher than this build understands MUST NOT be overwritten. | approved |
| [REQ-1313](REQ-1313-completed-component-record-fields.md) | `completed_components` MUST record the phase, the component, and a UTC timestamp for each entry. | approved |
| [REQ-1314](REQ-1314-editor-parses-not-matches.md) | The editor MUST parse the file rather than match text. | approved |
| [REQ-1315](REQ-1315-malformed-toml-exits-configuration-code.md) | A malformed TOML file MUST exit with the configuration code; see [REQ-1030]. | approved |
| [REQ-1316](REQ-1316-hook-error-owner-only-mode.md) | `.hook-error` MUST be written with mode `0o600`. | approved |
| [REQ-1401](REQ-1401-filesystem-trait-operation-set.md) | `FileSystem` MUST cover exactly the operations the rest of the workspace needs: reading a file, writing a file, appending to a file, removing a file, copying a file, creating and removing a symlink, reading a symlink's target, creating a directory, listing a directory, renaming, and querying metadata including whether a path exists and whether it is a symlink. | approved |
| [REQ-1402](REQ-1402-methods-take-resolved-absolute-paths.md) | Every `FileSystem` method MUST take an already-resolved absolute path. | approved |
| [REQ-1403](REQ-1403-writes-are-atomic-renames.md) | A write MUST be atomic: the implementation writes a temporary file in the destination's directory and renames it into place, so a reader never observes a partial file and a crash mid-write leaves the prior content. | approved |
| [REQ-1404](REQ-1404-narrow-default-file-modes.md) | A created file MUST have mode `0o600` and a created directory `0o700`, matching `v0.1.0`, except where a caller passes an explicit mode. | approved |
| [REQ-1405](REQ-1405-trait-exposes-executable-bit.md) | The trait MUST expose setting a file's executable bit, and reporting it. | approved |
| [REQ-1406](REQ-1406-remove-handles-file-link-emptydir.md) | `remove` MUST remove a file, a symlink, or an empty directory. | approved |
| [REQ-1410](REQ-1410-realfs-performs-real-effects.md) | `RealFs` MUST perform the effect against the real filesystem. | approved |
| [REQ-1411](REQ-1411-dryrunfs-performs-no-mutation.md) | `DryRunFs` MUST perform no mutation. | approved |
| [REQ-1412](REQ-1412-dryrunfs-reads-back-recorded-writes.md) | `DryRunFs` MUST answer a read of a path it recorded a write for with the content of that write, not with what is on disk. | approved |
| [REQ-1413](REQ-1413-memfs-holds-tree-in-memory.md) | `MemFs` MUST hold the whole tree in memory and touch no disk. | approved |
| [REQ-1414](REQ-1414-implementations-interchangeable-trait-object.md) | The three implementations MUST be interchangeable behind a trait object. `meowctl-cli` constructs one from the flags and passes it down; no other crate chooses. | approved |
| [REQ-1420](REQ-1420-symlink-over-symlink-replaces.md) | Creating a symlink over an existing symlink MUST replace it. | approved |
| [REQ-1421](REQ-1421-symlink-over-file-needs-backup.md) | Creating a symlink over an existing regular file MUST fail unless the caller asked for a backup. | approved |
| [REQ-1422](REQ-1422-remove-symlink-refuses-non-symlink.md) | Removing a symlink MUST fail when the path is not a symlink, so that a mistaken `remove_symlink` on a real file cannot delete it. | approved |
| [REQ-1430](REQ-1430-errors-carry-path-and-operation.md) | Every method MUST return a typed error carrying the path and the operation. | approved |
| [REQ-1431](REQ-1431-failed-write-removes-temporary-file.md) | A failed atomic write MUST remove its temporary file. | approved |
| [REQ-1432](REQ-1432-missing-file-read-distinguishable.md) | A read of a file that does not exist MUST be distinguishable from a read that failed for another reason. | approved |
| [REQ-1433](REQ-1433-dryrunfs-predicts-visible-failures.md) | `DryRunFs` MUST fail where `RealFs` would fail for a reason it can see: a write into a directory that does not exist and was not recorded as created, a symlink over a regular file with no backup requested. | approved |
| [REQ-1500](REQ-1500-trait-never-resolves-paths.md) | The `FileSystem` trait MUST NOT expand `~`, join a relative path against a working directory, or consult the environment. | approved |
| [REQ-1501](REQ-1501-executable-bit-noop-without-modes.md) | Setting and reporting a file's executable bit MUST be a no-op on a platform without mode bits. | approved |
| [REQ-1502](REQ-1502-remove-refuses-nonempty-directory.md) | `remove` MUST refuse a directory that is not empty. | approved |
| [REQ-1503](REQ-1503-trait-exposes-recursive-removal.md) | The trait MUST also expose removing a directory and everything in it, which is a separate method because the two are different decisions: one is an undo, the other discards a subtree. | approved |
| [REQ-1504](REQ-1504-dryrunfs-records-mutating-intent.md) | Every mutating method of `DryRunFs` MUST succeed, record the intent, and return what the caller would have seen. | approved |
| [REQ-1505](REQ-1505-dryrunfs-reads-pass-through.md) | Reads on `DryRunFs` MUST pass through to the real filesystem, because a hook that reads a file it just wrote in a dry run has to see plausible content. | approved |
| [REQ-1506](REQ-1506-symlink-replacement-reports-prior-target.md) | When creating a symlink replaces an existing symlink, the implementation MUST report the prior target so the caller can journal an inverse. | approved |
| [REQ-1507](REQ-1507-backup-renames-file-to-sibling.md) | When a backup is requested, the file MUST be renamed to a sibling path before the symlink is created. | approved |
| [REQ-1508](REQ-1508-backup-path-reported-to-caller.md) | When a backup is requested, the sibling path the file was renamed to MUST be reported to the caller. | approved |
| [REQ-1570](REQ-1570-windows-symlink-refused.md) | Creating a symlink on Windows MUST fail with an unsupported-operation error naming the link path. | approved |
| [REQ-1601](REQ-1601-executor-runs-and-resolves-commands.md) | `Executor` MUST cover running a command to completion and resolving a command name on `PATH`. | approved |
| [REQ-1602](REQ-1602-commands-use-argument-lists.md) | A command MUST be built from a program name and an argument list, never from a shell string. | approved |
| [REQ-1603](REQ-1603-result-separates-output-and-exit.md) | The result MUST carry stdout, stderr, and the exit code separately. | approved |
| [REQ-1604](REQ-1604-nonzero-exit-is-not-error.md) | A non-zero exit MUST NOT be an error. | approved |
| [REQ-1605](REQ-1605-environment-merges-overrides-over-parent.md) | The environment MUST be the parent environment merged with per-call overrides, with the override winning. | approved |
| [REQ-1606](REQ-1606-windows-resolution-tries-pathext.md) | Resolving a name on Windows MUST also try the extensions `PATHEXT` names, in the order it names them, falling back to `.COM;.EXE;.BAT;.CMD` when it is unset. | approved |
| [REQ-1610](REQ-1610-real-executor-spawns-process.md) | `RealExecutor` MUST spawn the process. | approved |
| [REQ-1611](REQ-1611-dry-run-runs-read-only-commands.md) | `DryRunExecutor` MUST run a command issued from a read-only phase. | approved |
| [REQ-1612](REQ-1612-test-executor-answers-scripted-commands.md) | A test executor MUST be able to answer a scripted set of commands. | approved |
| [REQ-1620](REQ-1620-output-streams-as-process-events.md) | A process's output MUST reach the caller as `ProcessOutput` events carrying the stream and the line, in addition to being accumulated for the result. | approved |
| [REQ-1621](REQ-1621-interactive-command-brackets-terminal-events.md) | An interactive command MUST emit `TerminalRequested` before it starts and `TerminalReleased` after it exits. | approved |
| [REQ-1622](REQ-1622-exec-knows-no-renderer.md) | `meowctl-exec` MUST NOT know what a renderer is. | approved |
| [REQ-1623](REQ-1623-captured-command-never-requests-terminal.md) | A command that is not interactive MUST NOT request the terminal. | approved |
| [REQ-1630](REQ-1630-missing-command-gets-named-error.md) | A command that does not exist MUST produce a distinct error naming it, not a generic spawn failure. | approved |
| [REQ-1631](REQ-1631-terminal-released-on-every-exit.md) | `TerminalReleased` MUST be emitted even when the process fails or the run is interrupted. | approved |
| [REQ-1632](REQ-1632-unresolved-name-returns-absent.md) | Resolving a name on `PATH` MUST return an absent result rather than an error when nothing matches, because `ctx.which` is how a component asks whether a tool is present; see REQ-2821. | approved |
| [REQ-1700](REQ-1700-dry-run-skips-other-phase-commands.md) | `DryRunExecutor` MUST NOT run a command issued from any other phase. | approved |
| [REQ-1701](REQ-1701-test-executor-fails-unscripted-commands.md) | A test executor MUST fail the test when an unscripted command is run. | approved |
| [REQ-1702](REQ-1702-exec-holds-no-renderer.md) | `meowctl-exec` MUST NOT hold a renderer. | approved |
| [REQ-1703](REQ-1703-exec-takes-no-suspend-callback.md) | `meowctl-exec` MUST NOT take a callback that suspends a renderer. | approved |
| [REQ-1704](REQ-1704-suspend-output-does-not-exist.md) | The workspace MUST NOT define a type, function, field or trait named `SuspendOutput`. | approved |
| [REQ-1770](REQ-1770-skipped-command-reports-success.md) | A command `DryRunExecutor` does not run MUST return exit code 0 with empty standard output and standard error. | approved |
| [REQ-1801](REQ-1801-http-fetches-url-as-bytes.md) | `Http` MUST expose one method: fetch a URL and return its body as bytes. | approved |
| [REQ-1802](REQ-1802-requests-carry-thirty-second-timeout.md) | A module or registry request MUST be bounded by a timeout covering the whole exchange, from the DNS lookup to the last byte of the body, 30 seconds by default, which is what `v0.1.0` gives its module loaders. | approved |
| [REQ-1803](REQ-1803-real-http-refuses-non-https.md) | `RealHttp` MUST refuse a URL whose scheme is not `https`. | approved |
| [REQ-1804](REQ-1804-non-200-response-fails.md) | A response whose status is not 200 MUST fail. | approved |
| [REQ-1805](REQ-1805-no-redirect-to-plaintext.md) | `Http` MUST NOT follow a redirect to a non-`https` URL. | approved |
| [REQ-1806](REQ-1806-response-body-is-bounded.md) | A response body MUST be bounded at 64 MiB. | approved |
| [REQ-1810](REQ-1810-error-distinguishes-four-request-cases.md) | The error MUST distinguish four cases that a request can reach: the URL was refused before any request, the host could not be reached, the server answered with a status, and the body could not be read. | approved |
| [REQ-1811](REQ-1811-error-carries-its-url.md) | An error MUST carry the URL it is about. | approved |
| [REQ-1812](REQ-1812-real-http-performs-request.md) | `RealHttp` MUST perform the request. | approved |
| [REQ-1813](REQ-1813-scripted-http-answers-from-table.md) | `ScriptedHttp` MUST answer from a table of URL to response. | approved |
| [REQ-1814](REQ-1814-offline-http-fails-every-request.md) | `OfflineHttp` MUST fail every request with the offline error. | approved |
| [REQ-1900](REQ-1900-real-http-reports-refused-scheme.md) | `RealHttp` MUST say so rather than attempting the request when it refuses a URL whose scheme is not `https`. | approved |
| [REQ-1901](REQ-1901-status-error-carries-code.md) | The error for a response whose status is not 200 MUST carry the code. | approved |
| [REQ-1902](REQ-1902-oversized-body-fails-untruncated.md) | A body that exceeds the bound MUST fail rather than being truncated. | approved |
| [REQ-1903](REQ-1903-scripted-http-records-requests.md) | `ScriptedHttp` MUST record the requests it was given in order. | approved |
| [REQ-1904](REQ-1904-scripted-http-fails-unknown-url.md) | `ScriptedHttp` MUST fail a URL that is not in the table rather than returning empty bytes. | approved |
| [REQ-1905](REQ-1905-offline-http-makes-no-request.md) | `OfflineHttp` MUST make no request. | approved |
| [REQ-2001](REQ-2001-op-has-nine-journaled-variants.md) | `Op` MUST have exactly these nine journalled variants: `WriteFile`, `AppendFile`, `CopyFile`, `Symlink`, `LinkFile`, `Mkdir`, `Download`, `DefaultsWrite`, and `PlistSet`. | approved |
| [REQ-2002](REQ-2002-apply-goes-through-filesystem.md) | `Op::apply` MUST perform the effect through a `FileSystem`. | approved |
| [REQ-2003](REQ-2003-inverse-computed-before-apply.md) | `Op::inverse` MUST be computed before the effect is applied, because it depends on the state the effect is about to destroy: the prior content of a file, the prior target of a symlink, whether a directory already existed. | approved |
| [REQ-2004](REQ-2004-apply-then-inverse-restores-state.md) | Applying an `Op` and then applying its inverse MUST restore the filesystem to the state it had before. | approved |
| [REQ-2005](REQ-2005-no-op-still-gets-correct-inverse.md) | An `Op` that finds the system already in its target state MUST still produce a correct inverse. | approved |
| [REQ-2010](REQ-2010-write-file-inverse-restores-content.md) | `WriteFile`'s inverse MUST restore the prior content when the file existed. | approved |
| [REQ-2011](REQ-2011-append-file-wraps-in-markers.md) | `AppendFile` MUST wrap what it appends in begin and end markers carrying an identifier. | approved |
| [REQ-2012](REQ-2012-copy-file-inverse-deletes-destination.md) | `CopyFile`'s inverse MUST delete the destination. | approved |
| [REQ-2013](REQ-2013-mkdir-inverse-removes-created-directory.md) | `Mkdir`'s inverse MUST remove the directory only when meowctl created it. | approved |
| [REQ-2014](REQ-2014-symlink-inverse-recreates-prior-link.md) | `Symlink`'s inverse MUST re-create the prior symlink when one existed. | approved |
| [REQ-2015](REQ-2015-link-file-inverse-removes-symlink.md) | `LinkFile`'s inverse MUST remove the symlink. | approved |
| [REQ-2016](REQ-2016-download-inverse-restores-content.md) | `Download`'s inverse MUST restore prior content when the destination existed. | approved |
| [REQ-2017](REQ-2017-macos-writes-record-prior-value.md) | `DefaultsWrite` and `PlistSet` MUST record the prior value. | approved |
| [REQ-2020](REQ-2020-journal-uses-v010-ndjson-format.md) | The journal MUST be a file of newline-delimited JSON records, one per operation, in the format `internal/rollback/rollback.go` writes: a sequence number, the phase, the component, the kind, and the inverse payload. | approved |
| [REQ-2021](REQ-2021-record-flushed-before-apply.md) | A record MUST be appended and flushed to disk before its operation is applied. | approved |
| [REQ-2022](REQ-2022-replay-runs-in-reverse-order.md) | Replay MUST apply inverses in reverse sequence order. | approved |
| [REQ-2023](REQ-2023-replay-continues-past-failures.md) | Replay MUST continue after an individual inverse fails. | approved |
| [REQ-2024](REQ-2024-replay-reports-three-outcomes.md) | Replay MUST report one of three outcomes: every inverse applied, some applied, or none. | approved |
| [REQ-2025](REQ-2025-journal-truncated-after-success.md) | The journal MUST be truncated after a successful run. | approved |
| [REQ-2026](REQ-2026-dry-run-journals-no-op.md) | An `Op` MUST NOT be journaled during a dry run. | approved |
| [REQ-2030](REQ-2030-bad-record-does-not-abort-replay.md) | A journal record that cannot be parsed MUST NOT abort the replay of the records around it. | approved |
| [REQ-2031](REQ-2031-inverse-of-vanished-target-succeeds.md) | An inverse whose target no longer exists MUST succeed rather than fail. | approved |
| [REQ-2032](REQ-2032-failed-append-fails-operation.md) | When the journal append fails, the operation MUST NOT be applied. | approved |
| [REQ-2100](REQ-2100-op-may-carry-inverse-variants.md) | `Op` MAY carry further variants that express an inverse. | approved |
| [REQ-2101](REQ-2101-inverse-variants-stay-out-of-journal.md) | The further variants that express an inverse MUST NOT be written to a journal. | approved |
| [REQ-2102](REQ-2102-apply-uses-no-other-route.md) | `Op::apply` MUST NOT touch the filesystem by any other route. | approved |
| [REQ-2103](REQ-2103-restoration-holds-for-every-variant.md) | Restoring the prior state by applying an `Op` and then its inverse MUST hold for every variant. | approved |
| [REQ-2104](REQ-2104-property-test-checks-restoration.md) | A property test over `MemFs` MUST check that applying an `Op` and then its inverse restores the prior state. | approved |
| [REQ-2105](REQ-2105-write-file-inverse-deletes-new-file.md) | `WriteFile`'s inverse MUST delete the file when it did not exist. | approved |
| [REQ-2106](REQ-2106-append-inverse-removes-marked-block.md) | `AppendFile`'s inverse MUST remove the block those markers delimit rather than truncating the file. | approved |
| [REQ-2107](REQ-2107-marker-identifier-defaults-generated.md) | The `AppendFile` marker identifier MUST default to a generated one. | approved |
| [REQ-2108](REQ-2108-marker-identifier-caller-overridable.md) | The `AppendFile` marker identifier MUST be overridable by the caller, because `ctx.append_file` takes a `marker` argument; see REQ-2828. | approved |
| [REQ-2109](REQ-2109-mkdir-inverse-keeps-existing-directory.md) | `Mkdir`'s inverse MUST do nothing when the directory already existed. | approved |
| [REQ-2110](REQ-2110-symlink-inverse-removes-new-link.md) | `Symlink`'s inverse MUST remove the link when no prior symlink existed. | approved |
| [REQ-2111](REQ-2111-link-file-inverse-restores-backup.md) | `LinkFile`'s inverse MUST restore the backed-up original when a backup was taken. | approved |
| [REQ-2112](REQ-2112-download-inverse-deletes-new-file.md) | `Download`'s inverse MUST delete the file when the destination did not exist. | approved |
| [REQ-2113](REQ-2113-macos-inverses-restore-prior-value.md) | The inverses of `DefaultsWrite` and `PlistSet` MUST restore the recorded prior value. | approved |
| [REQ-2114](REQ-2114-macos-inverse-deletes-new-key.md) | Where no prior value existed, the inverse MUST delete the key rather than write an empty one. | approved |
| [REQ-2115](REQ-2115-prior-value-read-before-write.md) | Reading the prior value MUST be part of computing the inverse, per REQ-2003, because `defaults read` after the write returns the new value. | approved |
| [REQ-2116](REQ-2116-journals-replay-across-versions.md) | A journal written by `v0.1.0` MUST replay under `v0.2.0`. | approved |
| [REQ-2117](REQ-2117-replay-reports-failed-inverses.md) | Replay MUST report which inverses failed. | approved |
| [REQ-2118](REQ-2118-leftover-journal-reported-as-interrupted.md) | A non-empty journal at startup MUST be reported as an interrupted previous run; see REQ-3042. | approved |
| [REQ-2119](REQ-2119-dry-run-creates-no-journal.md) | A journal file MUST NOT be created during a dry run. | approved |
| [REQ-2120](REQ-2120-bad-record-reported-and-skipped.md) | A journal record that cannot be parsed MUST be reported and skipped, because the alternative is a corrupt line stranding every earlier operation. | approved |
| [REQ-2140](REQ-2140-journal-writable-before-first-phase.md) | A run that journals MUST NOT start its first phase unless its journal file can be written. | approved |
| [REQ-2141](REQ-2141-record-survives-torn-predecessor.md) | A journal record MUST stay readable on its own when the record before it was torn. | approved |
| [REQ-2190](REQ-2190-v0-2-0-journal-replays-under-v0-1-0.md) | A journal written by `v0.2.0` MUST replay under `v0.1.0`. | approved |
| [REQ-2201](REQ-2201-predeclared-set-matches-makepredeclared.md) | The predeclared set MUST be exactly what `makePredeclared` in `internal/starlark/builtins.go` provides: `component`, `pkg`, `unpkg`, `uppkg`, `repo`, `query_pm`, `dep`, `module`, `replace`, `select`, `platform`, and the `json` module. | approved |
| [REQ-2202](REQ-2202-component-records-its-declaration.md) | `component(name, after = [], kwargs)` MUST record a declaration carrying the name, the ordering hints, and any extra keyword arguments, in declaration order. `name` may be positional or named and is required; `after` is named only and must be a list of strings. | approved |
| [REQ-2203](REQ-2203-pkg-records-package-declaration.md) | `pkg(manager, name, version = "", kwargs)` MUST record a package declaration. | approved |
| [REQ-2204](REQ-2204-repo-records-repository-declaration.md) | `repo(...)` MUST record a repository declaration targeting a named manager. | approved |
| [REQ-2205](REQ-2205-query-pm-returns-interrogation.md) | `query_pm(manager)` MUST call the registered handler's `interrogate` on its own evaluation and return what it returns. | approved |
| [REQ-2206](REQ-2206-one-grammar-reads-every-manifest.md) | `dep`, `module`, and `replace` MUST read every manifest meowctl encounters: a `deps.mod`, which meowctl writes, and a `MODULE.meow`, which a module author writes. | approved |
| [REQ-2207](REQ-2207-platform-returns-machine-struct.md) | `platform()` MUST return a struct carrying the operating system, the architecture, and, on Linux, the distribution and its `ID_LIKE` value. | approved |
| [REQ-2208](REQ-2208-select-matches-distribution-through-id-like.md) | `select(cases)` MUST choose by platform using the matching rules in `matchesPlatform` and `matchesLinuxDistro`, including matching a distribution through `ID_LIKE` so a Mint machine matches a `debian` case. | approved |
| [REQ-2210](REQ-2210-declarations-collected-per-evaluation.md) | Declarations MUST be collected per evaluation, not in a global. | approved |
| [REQ-2211](REQ-2211-accumulator-holds-owned-data.md) | The accumulator MUST hold owned data. | approved |
| [REQ-2212](REQ-2212-declaration-order-preserved.md) | Declaration order MUST be preserved. | approved |
| [REQ-2220](REQ-2220-load-resolves-through-composite-loader.md) | `load()` MUST resolve through a composite loader covering the four schemes `internal/starlark/loader/composite.go` handles: a path relative to the config directory, `self://`, `user://`, `github://owner/repo@ref//path`, and `@name//path` for a registry module. | approved |
| [REQ-2221](REQ-2221-extensionless-registry-path-loads-init-star.md) | A registry URL whose final segment contains no `.` MUST resolve to `<path>/init.star` inside the module. A path whose final segment does contain a `.` is taken as written. | approved |
| [REQ-2222](REQ-2222-module-verified-before-evaluation.md) | A module file MUST be integrity-checked before it is evaluated; see [REQ-2430]. | approved |
| [REQ-2223](REQ-2223-module-url-evaluated-once.md) | The same module URL loaded twice within one command MUST be evaluated once and cached, matching `v0.1.0`, so a component graph that loads a shared helper from twenty components pays for it once. | approved |
| [REQ-2230](REQ-2230-hook-called-with-ctx-first.md) | The evaluator MUST call a named global function in an evaluated file, passing the `ctx` value first and whatever else the caller supplies after it, positionally and by keyword. | approved |
| [REQ-2231](REQ-2231-absent-hook-is-success.md) | A hook that is absent MUST be a success, not an error. | approved |
| [REQ-2232](REQ-2232-non-callable-hook-global-errors.md) | A global that is present but not callable MUST be an error naming the component and the hook. | approved |
| [REQ-2233](REQ-2233-evaluation-reports-string-globals.md) | An evaluation MUST report the value of every top-level name bound to a string. | approved |
| [REQ-2234](REQ-2234-evaluation-reports-string-list-globals.md) | An evaluation MUST likewise report the value of every top-level name bound to a list of strings. | approved |
| [REQ-2240](REQ-2240-error-carries-source-position.md) | An evaluation error MUST carry the file, the line, the column, and the source text of the offending line, so `meowctl-cli` can render a diagnostic that points at it; see [REQ-1032] and [REQ-3431]. | approved |
| [REQ-2241](REQ-2241-error-carries-load-chain.md) | An error raised inside a `load()`ed module MUST carry the chain of files that led to it, not only the innermost one. | approved |
| [REQ-2242](REQ-2242-fail-surfaces-as-configuration-error.md) | A Starlark `fail()` call MUST surface its message as a configuration error. | approved |
| [REQ-2250](REQ-2250-syntax-error-reports-position.md) | A file that does not parse MUST report the syntax error with its position. | approved |
| [REQ-2251](REQ-2251-argument-error-names-builtin-and-argument.md) | A builtin called with wrong or missing arguments MUST name the builtin, the argument, and what was expected. | approved |
| [REQ-2252](REQ-2252-runaway-evaluation-stoppable-from-outside.md) | A configuration that loops forever MUST be stoppable from outside the evaluation. | approved |
| [REQ-2253](REQ-2253-unfetchable-load-exits-module-code.md) | A `load()` of a module that cannot be fetched MUST exit with the module code, not the configuration code; see [REQ-1030]. | approved |
| [REQ-2300](REQ-2300-no-builtin-added-or-removed.md) | The predeclared set MUST NOT change: no name is added to it and none is removed from it. | approved |
| [REQ-2301](REQ-2301-unpkg-and-uppkg-share-shape.md) | `unpkg` and `uppkg` MUST take the same shape as `pkg` for removal and update. | approved |
| [REQ-2302](REQ-2302-package-argument-given-twice-errors.md) | Both `manager` and `name` are required, and either may be given positionally or by keyword; supplying one both ways MUST be an error. | approved |
| [REQ-2303](REQ-2303-module-accepts-and-records-compat.md) | `module(name, version = "", compat = None)` MUST accept `compat` and record it. | approved |
| [REQ-2304](REQ-2304-bare-module-name-loads-root-init.md) | `@name` alone MUST resolve to the module's root `init.star`. | approved |
| [REQ-2305](REQ-2305-fail-exits-with-configuration-code.md) | A Starlark `fail()` call MUST exit with the configuration code. | approved |
| [REQ-2306](REQ-2306-unparsed-file-applies-nothing.md) | A file that does not parse MUST NOT partially apply what came before it. | approved |
| [REQ-2307](REQ-2307-nothing-claims-an-evaluation-bound.md) | meowctl MUST NOT claim to bound a configuration that loops forever from inside the evaluation. | approved |
| [REQ-2340](REQ-2340-select-without-match-or-default-fails.md) | A `select` call whose cases match nothing and that carries no `//conditions:default` MUST fail the evaluation. | approved |
| [REQ-2341](REQ-2341-platform-reports-os-release.md) | On Linux, `platform()` MUST report the `ID`, `ID_LIKE` and `VERSION_ID` values of `/etc/os-release`. | approved |
| [REQ-2342](REQ-2342-unreadable-os-release-gives-empty-fields.md) | When `/etc/os-release` is absent or can't be read, `platform()` MUST return empty distribution fields in place of failing. | approved |
| [REQ-2343](REQ-2343-query-pm-outside-hook-says-so.md) | A `query_pm` call outside a hook MUST fail with a configuration error saying that `query_pm` works only inside a hook. | approved |
| [REQ-2370](REQ-2370-error-text-follows-the-library.md) | A Starlark runtime error MUST report the same mistake `v0.1.0` reports, in the wording `starlark-rust` produces. | approved |
| [REQ-2401](REQ-2401-mvs-selects-maximum-required-version.md) | Version selection MUST be Minimal Version Selection over the dependency graph, selecting for each module the maximum version any path requires. | approved |
| [REQ-2402](REQ-2402-semver-ordering-with-none-lowest.md) | Versions MUST be compared by semver ordering, with the literal `none` ordering below every valid version. | approved |
| [REQ-2403](REQ-2403-invalid-version-fails-resolution.md) | A dependency declaring a version that is not valid semver MUST fail resolution naming the module and the version, rather than being skipped or treated as `none`. | approved |
| [REQ-2404](REQ-2404-resolution-is-deterministic.md) | A resolution MUST be deterministic. | approved |
| [REQ-2405](REQ-2405-dependency-cycle-terminates.md) | A cycle in the dependency graph MUST terminate rather than loop. | approved |
| [REQ-2410](REQ-2410-registry-module-resolved-through-index.md) | A registry module MUST be resolved through the registry index, which lists each module's versions, tarball URLs, and integrity hashes. | approved |
| [REQ-2411](REQ-2411-github-module-name-form.md) | A GitHub module MUST be named `github:owner/repo@ref`, where the ref is a tag or a branch. | approved |
| [REQ-2412](REQ-2412-github-transitive-dependencies-walked.md) | A GitHub module's transitive dependencies MUST be read from its own manifest and walked, so an aggregate module brings its dependencies with it. | approved |
| [REQ-2413](REQ-2413-fetching-uses-https.md) | Fetching MUST use HTTPS. | approved |
| [REQ-2420](REQ-2420-path-replace-serves-local-directory.md) | `replace(name, path)` MUST serve every file for that module from the local directory, skipping fetch, cache, and integrity verification. | approved |
| [REQ-2421](REQ-2421-source-replace-resolves-from-source.md) | `replace(name, source)` MUST resolve the module from the replacement source. | approved |
| [REQ-2422](REQ-2422-local-override-wins.md) | An override in `deps.local.mod` MUST win over the same module in `deps.mod`, because the local file is how one machine differs from the rest. | approved |
| [REQ-2430](REQ-2430-tarball-verified-before-extraction.md) | Every fetched tarball MUST be verified against the integrity hash from the index before it is extracted. | approved |
| [REQ-2431](REQ-2431-hash-mismatch-fails-with-module-code.md) | A hash mismatch MUST fail with the module code. | approved |
| [REQ-2432](REQ-2432-extraction-preserves-executable-bit.md) | Extraction MUST preserve the executable bit from the tar entry, deriving the on-disk mode as `0o644` or `0o755`. | approved |
| [REQ-2433](REQ-2433-extraction-rejects-escaping-paths.md) | Extraction MUST reject an entry whose path escapes the module root. | approved |
| [REQ-2434](REQ-2434-extraction-drops-single-top-directory.md) | Extraction MUST drop a single top-level directory when every entry in the archive is inside one. | approved |
| [REQ-2440](REQ-2440-cache-keyed-by-name-and-version.md) | A module MUST be cached under the cache directory keyed by name and by resolved version, or by commit SHA for a GitHub module; see [REQ-1021]. | approved |
| [REQ-2441](REQ-2441-verified-cache-needs-no-network.md) | A cached module whose per-file hashes match what was recorded at extraction MUST be used without a network request. | approved |
| [REQ-2442](REQ-2442-mismatched-cache-is-refetched.md) | A cached module whose files do not match the recorded hashes MUST be re-fetched rather than used, because the cache is not authoritative. | approved |
| [REQ-2443](REQ-2443-locked-cached-resolution-works-offline.md) | Resolution MUST work offline when every module in the lock is cached and verified. | approved |
| [REQ-2444](REQ-2444-cache-record-unloadable-name.md) | The cache record MUST be written inside the cache directory under a name that cannot be loaded as a module file. | approved |
| [REQ-2445](REQ-2445-dry-run-populates-cache.md) | The cache MUST be populated during a dry run. | approved |
| [REQ-2450](REQ-2450-locked-module-skips-selection.md) | A module already present in the lock MUST be used at the locked version without running selection again. Selection runs when a module is absent, or when the caller asked to ignore the lock. | approved |
| [REQ-2451](REQ-2451-sync-writes-full-resolution.md) | Syncing MUST write the full resolution: version, source, integrity, and commit SHA where applicable. | approved |
| [REQ-2452](REQ-2452-each-manifest-gets-own-lock.md) | Syncing `deps.mod` and `deps.local.mod` MUST produce their own lock files. | approved |
| [REQ-2453](REQ-2453-upgrade-reresolves-named-modules-only.md) | An upgrade MUST clear the locked version for the named modules and re-resolve them, leaving the rest of the lock untouched. | approved |
| [REQ-2460](REQ-2460-index-fetch-failure-module-code.md) | A registry index that cannot be fetched MUST fail with the module code. | approved |
| [REQ-2461](REQ-2461-unknown-module-reported-by-name.md) | A module named in `deps.mod` but absent from the index MUST be reported by name, distinguished from a version of it that does not exist. | approved |
| [REQ-2462](REQ-2462-unsatisfiable-requirement-names-constraint.md) | A requirement no version satisfies MUST name the module and the constraint. | approved |
| [REQ-2463](REQ-2463-interrupted-fetch-leaves-no-partial-module.md) | A fetch interrupted partway MUST NOT leave a partial tarball or a partially extracted module that a later run would treat as cached; see [REQ-1431]. | approved |
| [REQ-2464](REQ-2464-missing-replace-path-fails.md) | A `replace` pointing at a path that does not exist MUST fail naming the path, not fall back to fetching the original. | approved |
| [REQ-2500](REQ-2500-same-graph-same-build-list.md) | The same graph MUST produce the same build list in the same order on every run and on every machine. | approved |
| [REQ-2501](REQ-2501-github-ref-resolved-to-commit.md) | A GitHub module MUST be resolved to the commit SHA that ref pointed to at resolution time. | approved |
| [REQ-2502](REQ-2502-github-commit-sha-recorded.md) | The SHA MUST be recorded so later fetches are reproducible; see [REQ-1220]. | approved |
| [REQ-2503](REQ-2503-fetching-never-runs-git.md) | Fetching MUST NOT shell out to `git`. | approved |
| [REQ-2504](REQ-2504-path-replace-recorded-in-lock.md) | The lock entry for a `replace(name, path)` MUST record that the module is replaced and where it points; see [REQ-1220]. | approved |
| [REQ-2505](REQ-2505-source-replace-verified-normally.md) | `replace(name, source)` MUST verify its integrity normally: a remote fork is not more trusted than the original. | approved |
| [REQ-2506](REQ-2506-files-verified-before-evaluation.md) | Every extracted file MUST be verified against its recorded per-file hash before it is evaluated. | approved |
| [REQ-2507](REQ-2507-hash-mismatch-names-both-hashes.md) | A hash mismatch MUST name the module, the expected hash, and the actual one. | approved |
| [REQ-2508](REQ-2508-hash-mismatch-leaves-no-files.md) | A hash mismatch MUST NOT leave the extracted files in the cache. | approved |
| [REQ-2509](REQ-2509-extraction-otherwise-drops-nothing.md) | Extraction MUST drop nothing when the entries are not all inside one top-level directory. | approved |
| [REQ-2510](REQ-2510-unrecorded-cache-is-refetched.md) | A cache with no record at all MUST be treated the same way as one whose files do not match: the module was extracted by something that did not write one, and what it holds is unknown. | approved |
| [REQ-2511](REQ-2511-cache-record-covers-tarball-integrity.md) | The cache record MUST cover the tarball's integrity as well as the per-file hashes. | approved |
| [REQ-2512](REQ-2512-sri-sidecar-stays-readable-by-v010.md) | `v0.1.0` writes half of the cache record already, as the `.sri` sidecar that carries the tarball hash, and the format here MUST keep the `.sri` sidecar readable by `v0.1.0`: the two binaries share one cache. | approved |
| [REQ-2513](REQ-2513-shared-module-resolves-per-lock.md) | A module in both `deps.mod` and `deps.local.mod` MUST resolve independently in each. | approved |
| [REQ-2514](REQ-2514-index-fetch-failure-names-cause.md) | A registry index that cannot be fetched MUST say whether the failure was the network, the status code, or the parse. | approved |
| [REQ-2540](REQ-2540-missing-github-ref-reported-by-name.md) | A `github://` load whose ref resolves to no commit MUST fail with the module exit code, naming the repository and the ref and saying that the ref wasn't found. | approved |
| [REQ-2601](REQ-2601-four-exports-register-handler.md) | A component MUST be registered as a handler when it exports all four of `pm_name`, `install_pkg`, `uninstall_pkg`, and `interrogate`. | approved |
| [REQ-2602](REQ-2602-update-and-repo-exports-optional.md) | `update_pkg` and `add_repo` MUST be optional. | approved |
| [REQ-2603](REQ-2603-handlers-registered-before-hooks.md) | Registration MUST happen during the first evaluation pass, before any hook runs, so that a hook in the first component can declare a package handled by the last. | approved |
| [REQ-2604](REQ-2604-duplicate-manager-names-reported.md) | Two components exporting the same `pm_name` MUST be reported rather than silently resolved. | approved |
| [REQ-2610](REQ-2610-pkg-dispatches-to-install.md) | A `pkg()` declaration MUST dispatch to `install_pkg(ctx, name, version, kwargs)` on the handler for its manager. | approved |
| [REQ-2611](REQ-2611-unpkg-uppkg-dispatch-to-handlers.md) | An `unpkg()` declaration MUST dispatch to `uninstall_pkg`, and a `uppkg()` declaration to `update_pkg`. | approved |
| [REQ-2612](REQ-2612-uppkg-falls-back-to-install-latest.md) | When a handler exports no `update_pkg`, `uppkg()` MUST fall back to `install_pkg(ctx, name, "latest", kwargs)`. | approved |
| [REQ-2613](REQ-2613-repo-dispatches-to-add-repo.md) | A `repo()` declaration MUST dispatch to `add_repo(ctx, kwargs)`. | approved |
| [REQ-2614](REQ-2614-dispatch-passes-caller-ctx.md) | Dispatch MUST pass the calling component's `ctx`, so the handler's effects are journaled against the component that asked for the package rather than against the handler. | approved |
| [REQ-2615](REQ-2615-declarations-dispatched-in-order.md) | Declarations MUST be dispatched in declaration order within a component. | approved |
| [REQ-2620](REQ-2620-query-pm-returns-interrogate-result.md) | `query_pm(manager)` MUST call the handler's `interrogate(ctx)` and return its result to the caller. | approved |
| [REQ-2621](REQ-2621-interrogate-callable-in-dry-run.md) | `interrogate` MUST be callable during a dry run, because it reads rather than writes; see REQ-1611. | approved |
| [REQ-2630](REQ-2630-unknown-manager-lists-registered.md) | A `pkg()` naming a manager with no registered handler MUST fail naming the manager and listing the managers that are registered. | approved |
| [REQ-2631](REQ-2631-raising-handler-fails-declaring-component.md) | A handler function that raises MUST fail the component that declared the package, not the handler component. | approved |
| [REQ-2632](REQ-2632-wrong-return-type-is-handler-defect.md) | A handler returning a value of an unexpected type MUST be reported as a handler defect naming the function, rather than being coerced. | approved |
| [REQ-2700](REQ-2700-partial-exports-not-registered.md) | A component exporting some but not all of them MUST NOT be registered. | approved |
| [REQ-2701](REQ-2701-partial-exports-are-warned.md) | The omission MUST be warned about when a component exports some but not all of the four, because it is nearly always a mistake. | approved |
| [REQ-2702](REQ-2702-repo-without-add-repo-fails.md) | A `repo()` declaration MUST fail naming the manager when the handler exports no `add_repo`. | approved |
| [REQ-2703](REQ-2703-components-processed-in-graph-order.md) | Components MUST be processed in the graph order when package declarations are dispatched; see REQ-3013. | approved |
| [REQ-2704](REQ-2704-handler-failure-names-both-components.md) | The message for a handler function that raises MUST name both the declaring component and the handler component. | approved |
| [REQ-2801](REQ-2801-ctx-exposes-six-data-properties.md) | `ctx` MUST expose the six data properties `New` in `internal/ctx/ctx.go` registers: `home`, `dry_run`, `component_dir`, `state_dir`, `shell`, and `platform`. | approved |
| [REQ-2802](REQ-2802-shell-named-in-runtime-hooks.md) | `shell` MUST name the shell in a runtime hook phase. | approved |
| [REQ-2803](REQ-2803-component-and-state-directories-differ.md) | `component_dir` MUST be the component's own source directory and `state_dir` its persistent per-component directory. | approved |
| [REQ-2810](REQ-2810-ctx-exposes-twenty-four-methods.md) | `ctx` MUST expose exactly the twenty-four methods `New` registers, under those names: `log`, `env`, `write_file`, `append_file`, `delete_file`, `copy_file`, `symlink`, `remove_symlink`, `link_file`, `mkdir`, `read_file`, `file_exists`, `list_dir`, `run`, `git_clone`, `download`, `defaults_write`, `plist_set`, `prompt`, `emit`, `add_path`, `render`, `render_file`, and `which`. | approved |
| [REQ-2811](REQ-2811-unknown-attribute-not-found.md) | An unknown attribute MUST report attribute-not-found rather than raise an internal error, matching `Attr` returning `(nil, nil)`. | approved |
| [REQ-2812](REQ-2812-methods-reject-relative-paths.md) | Every method MUST reject a relative path. | approved |
| [REQ-2813](REQ-2813-mutating-methods-go-through-ops.md) | Every mutating method MUST go through an `Op`. | approved |
| [REQ-2814](REQ-2814-methods-never-branch-on-dry-run.md) | A method MUST NOT branch on whether this is a dry run. | approved |
| [REQ-2820](REQ-2820-run-returns-stdout-stderr-exit-code.md) | `run(cmd, args = [], env = {}, cwd = None, interactive = False)` MUST return a value carrying `stdout`, `stderr`, and `exit_code`, under those names. | approved |
| [REQ-2821](REQ-2821-which-returns-path-or-falsey.md) | `which(name)` MUST return the resolved path or a falsey value when nothing matches; see REQ-1632. | approved |
| [REQ-2822](REQ-2822-git-clone-runs-git-command.md) | `git_clone(url, dst, ref = None)` MUST build a `git` command and run it through the `Executor`, rather than implementing a fetch. | approved |
| [REQ-2823](REQ-2823-download-fetches-over-https.md) | `download(url, dst, checksum = None)` MUST fetch over HTTPS. | approved |
| [REQ-2824](REQ-2824-emit-writes-in-shell-phases.md) | `emit(text)` MUST write to stdout only during the `shell` and `login` phases. | approved |
| [REQ-2825](REQ-2825-add-path-prepends-process-path.md) | `add_path(dir)` MUST prepend the directory to the `PATH` every later `run` in this process sees. | approved |
| [REQ-2826](REQ-2826-prompt-uses-interaction-trait.md) | `prompt(question)` MUST go through the `Interaction` trait, not read stdin directly; see REQ-3260. | approved |
| [REQ-2827](REQ-2827-render-substitutes-and-returns.md) | `render(template_str, vars)` MUST substitute `{{name}}` for each key in `vars` and return the result. | approved |
| [REQ-2828](REQ-2828-append-file-accepts-caller-marker.md) | `append_file(dst, content, marker = None)` MUST accept a caller-supplied marker. | approved |
| [REQ-2830](REQ-2830-read-only-ctx-has-no-mutators.md) | In a read-only phase, `ctx` MUST expose no mutating method. | approved |
| [REQ-2831](REQ-2831-runtime-ctx-exposes-eight-attributes.md) | In a runtime hook phase, `ctx` MUST expose exactly the eight attributes `ShellCtxAllowList` names: `emit`, `file_exists`, `list_dir`, `platform`, `read_file`, `run`, `shell`, and `state_dir`. | approved |
| [REQ-2832](REQ-2832-restriction-enforced-by-hook-value.md) | The restriction MUST be enforced by the value the hook receives, not by a check inside each method. | approved |
| [REQ-2840](REQ-2840-type-error-names-method-and-argument.md) | A method called with a wrong argument type MUST name the method, the argument, and the expected type. | approved |
| [REQ-2841](REQ-2841-failed-effect-fails-hook.md) | A failed effect MUST propagate as a hook failure, which fails the component and triggers rollback; see REQ-3030. | approved |
| [REQ-2842](REQ-2842-read-file-missing-fails.md) | `read_file` on a missing file MUST fail. | approved |
| [REQ-2843](REQ-2843-run-nonzero-exit-does-not-fail.md) | `run` MUST NOT fail on a non-zero exit; see REQ-1604. | approved |
| [REQ-2844](REQ-2844-remove-symlink-on-non-symlink-fails.md) | `remove_symlink` on a path that is not a symlink MUST fail; see REQ-1422. | approved |
| [REQ-2872](REQ-2872-download-bounded-at-five-minutes.md) | `download` MUST allow a transfer to take up to 5 minutes end to end. | approved |
| [REQ-2900](REQ-2900-shell-none-outside-runtime-hooks.md) | `shell` MUST be `None` in every other phase. | approved |
| [REQ-2901](REQ-2901-platform-matches-platform-builtin.md) | `platform` MUST be the same struct `platform()` returns; see REQ-2207 and REQ-1013. | approved |
| [REQ-2902](REQ-2902-methods-expand-leading-tilde.md) | Every method MUST expand a leading `~`; see REQ-1022. | approved |
| [REQ-2903](REQ-2903-download-journals-download-op.md) | `download(url, dst, checksum = None)` MUST journal a `Download` op, so an interrupted run restores what was at `dst`; see REQ-2016. | approved |
| [REQ-2904](REQ-2904-download-verifies-checksum-before-writing.md) | When a checksum is given it MUST be verified before anything is written, for the reason REQ-2430 gives. | approved |
| [REQ-2905](REQ-2905-emit-is-no-op-elsewhere.md) | `emit(text)` MUST be a no-op outside the `shell` and `login` phases. | approved |
| [REQ-2906](REQ-2906-add-path-skips-first-entry.md) | `add_path(dir)` MUST do nothing when the directory is already first. | approved |
| [REQ-2907](REQ-2907-render-file-reads-component-template.md) | `render_file(src, vars)` MUST read `src` relative to `component_dir`, render it the same way, and return the result. | approved |
| [REQ-2908](REQ-2908-render-vars-map-strings-to-strings.md) | `vars` MUST be a mapping of strings to strings. | approved |
| [REQ-2909](REQ-2909-non-string-vars-are-errors.md) | Anything other than a mapping of strings to strings passed as `vars` MUST be an error rather than being formatted. | approved |
| [REQ-2910](REQ-2910-append-file-generates-missing-marker.md) | `append_file(dst, content, marker = None)` MUST generate a marker when it is absent. | approved |
| [REQ-2911](REQ-2911-read-only-write-attempt-not-found.md) | A hook that tries to write in a read-only phase MUST get attribute-not-found rather than a silent no-op. | approved |
| [REQ-2912](REQ-2912-file-exists-missing-returns-false.md) | `file_exists` on a missing file MUST return false. | approved |
| [REQ-2940](REQ-2940-download-writes-only-verified-bytes.md) | `ctx.download` MUST NOT write any bytes whose hash differs from the given checksum. | approved |
| [REQ-2941](REQ-2941-run-reports-minus-one-without-exit-code.md) | `ctx.run` MUST report `exit_code` as `-1` for a process that ended without an exit code. | approved |
| [REQ-3001](REQ-3001-stages-are-distinct-types.md) | Discovery, resolution, planning, and execution MUST be distinct types, so a stage cannot be entered before the one it depends on has produced its value. | approved |
| [REQ-3002](REQ-3002-plan-is-effect-free-value.md) | `Plan` MUST be a value computed from the configuration, the lock state, and the sentinel, without performing an effect. | approved |
| [REQ-3003](REQ-3003-execution-takes-plan-and-effects.md) | Execution MUST take a `Plan` and the effects. | approved |
| [REQ-3010](REQ-3010-discovery-merges-local-star.md) | Components MUST be discovered by evaluating `init.star`, then merging `local.star` when it exists, with a component declared in both counting once. | approved |
| [REQ-3011](REQ-3011-dependencies-expand-transitively.md) | A component's dependencies MUST be expanded transitively, so declaring an aggregate brings in what it needs. | approved |
| [REQ-3012](REQ-3012-unmatched-guard-drops-component.md) | A component whose platform or distribution guard does not match the current machine MUST be dropped from the graph. | approved |
| [REQ-3013](REQ-3013-topological-order-declaration-tiebreak.md) | Execution order MUST be a topological sort of the dependency graph, with declaration order as the tie-break between components that have no dependency between them. | approved |
| [REQ-3014](REQ-3014-after-read-from-both-places.md) | A component's `after` list MUST be read from both the `component()` declaration and the component file's own top-level `after` global. | approved |
| [REQ-3015](REQ-3015-cycle-names-its-components.md) | A cycle MUST be reported with the components on it. | approved |
| [REQ-3016](REQ-3016-filter-restricts-run.md) | A filter naming components MUST restrict the run to those components and their dependencies. | approved |
| [REQ-3017](REQ-3017-bare-name-lookup-order.md) | A component named by a bare name MUST be looked for as `components/<name>.star` and then as `components/<name>/init.star`. | approved |
| [REQ-3018](REQ-3018-module-bare-name-resolves-locally.md) | A bare name in the `after` list of a module's component MUST resolve inside that module, as `<module>//components/<name>`. | approved |
| [REQ-3020](REQ-3020-first-pass-evaluates-every-component.md) | Every component MUST be evaluated once to collect its globals and register package-manager handlers, before any hook runs. See REQ-2603. | approved |
| [REQ-3021](REQ-3021-second-pass-reuses-globals.md) | The second pass MUST call hooks in graph order, reusing the globals from the first rather than re-evaluating. | approved |
| [REQ-3030](REQ-3030-phase-runs-hooks-in-order.md) | A phase MUST run its hook for every component in order. | approved |
| [REQ-3031](REQ-3031-phase-set-follows-phase-order.md) | A phase set MUST run its phases in the order REQ-1011 gives. | approved |
| [REQ-3032](REQ-3032-failed-phase-set-rolls-back.md) | A failed phase set MUST trigger rollback unless the caller disabled it. | approved |
| [REQ-3033](REQ-3033-absent-hook-records-completion.md) | A component whose hook is absent MUST be recorded as completed, not skipped. | approved |
| [REQ-3034](REQ-3034-dry-run-executor-per-phase.md) | During a dry run the engine MUST give each phase an executor built for that phase, because whether a command runs depends on whether the phase is read-only; see REQ-1611. | approved |
| [REQ-3035](REQ-3035-runtime-phase-runs-without-plan.md) | A runtime hook phase MUST run over every component in graph order without a `Plan`. | approved |
| [REQ-3040](REQ-3040-completed-component-is-skipped.md) | A component already recorded as completed for a phase MUST be skipped, unless the caller forced a re-run. | approved |
| [REQ-3041](REQ-3041-completion-recorded-immediately.md) | A component MUST be recorded as completed immediately after its hook succeeds, not at the end of the phase, so an interrupted run resumes where it stopped. | approved |
| [REQ-3042](REQ-3042-leftover-journal-reported-first.md) | A non-empty rollback journal at startup MUST be reported as an interrupted previous run before anything else happens; see REQ-2025. | approved |
| [REQ-3043](REQ-3043-stale-fingerprint-clears-completion.md) | A component whose module fingerprint differs from what `installed.lock` recorded MUST have its completion records cleared, so the next run re-links it. | approved |
| [REQ-3044](REQ-3044-staleness-uses-module-fingerprint.md) | Staleness MUST be computed from the fingerprint in REQ-1233, so a GitHub module re-synced to a new commit invalidates even though no version changed. | approved |
| [REQ-3050](REQ-3050-engine-emits-lifecycle-events.md) | The engine MUST emit `PlanComputed` before execution, `PhaseStarted` and `PhaseFinished` around each phase, and `ComponentStarted`, `ComponentSkipped` or `ComponentFinished` around each component. | approved |
| [REQ-3051](REQ-3051-engine-holds-no-renderer.md) | The engine MUST NOT hold a renderer. | approved |
| [REQ-3052](REQ-3052-every-skip-carries-reason.md) | Every skip MUST carry its reason: already completed, filtered out, or platform mismatch. | approved |
| [REQ-3060](REQ-3060-hook-failure-names-component-phase.md) | A hook failing MUST name the component, the phase, and the underlying error. | approved |
| [REQ-3061](REQ-3061-partial-rollback-reports-failed-inverses.md) | A rollback that partially succeeds MUST report which inverses failed. | approved |
| [REQ-3062](REQ-3062-interrupt-stops-before-next-component.md) | A run interrupted by a signal MUST stop before starting the next component. | approved |
| [REQ-3063](REQ-3063-failure-keeps-earlier-successes-reported.md) | A component that fails MUST NOT prevent the phase from reporting the components that already succeeded. | approved |
| [REQ-3064](REQ-3064-interrupted-run-never-rolls-back.md) | An interrupted run MUST NOT roll back. | approved |
| [REQ-3100](REQ-3100-plan-names-phases-and-skips.md) | `Plan` MUST name the phases to run, the components in each, and which are skipped and why. | approved |
| [REQ-3101](REQ-3101-dry-run-renders-plan.md) | A dry run MUST render the plan. | approved |
| [REQ-3102](REQ-3102-dry-run-executes-nothing.md) | A dry run MUST NOT execute the plan. | approved |
| [REQ-3103](REQ-3103-undeclared-dependency-is-marked.md) | A component reached through transitive expansion and not declared by the configuration MUST be marked as such, because a `meowctl remove` of one tool must not run the uninstall hook of the package manager it was reached through; see REQ-3032. | approved |
| [REQ-3104](REQ-3104-undeclared-after-name-pulled-in.md) | A name in a component's `after` list that the configuration does not declare MUST be pulled into the graph. | approved |
| [REQ-3105](REQ-3105-cycle-exits-configuration-code.md) | A cycle MUST exit with the configuration code. | approved |
| [REQ-3106](REQ-3106-unmatched-filter-name-errors.md) | A name in a filter that matches nothing MUST be an error rather than an empty run. | approved |
| [REQ-3107](REQ-3107-component-dir-follows-layout.md) | The `component_dir` of a component named by a bare name MUST be `components/<name>/` when that directory exists and `components/` otherwise. | approved |
| [REQ-3108](REQ-3108-phase-stops-at-first-failure.md) | A phase MUST stop at the first component that fails. | approved |
| [REQ-3109](REQ-3109-phase-set-stops-at-failure.md) | A phase set MUST stop at the first failed phase. | approved |
| [REQ-3110](REQ-3110-rollback-outcome-is-recorded.md) | The rollback outcome of a failed phase set MUST be recorded; see REQ-2024. | approved |
| [REQ-3111](REQ-3111-absent-hook-reports-nothing-to-do.md) | A component whose hook is absent MUST also be reported as finishing with nothing to do rather than as succeeding, so a reader can tell a component that did work from one that had none. | approved |
| [REQ-3112](REQ-3112-runtime-phase-ignores-sentinel.md) | A runtime hook phase MUST NOT consult the sentinel. | approved |
| [REQ-3113](REQ-3113-runtime-phase-records-nothing.md) | A runtime hook phase MUST NOT record what finished. | approved |
| [REQ-3114](REQ-3114-runtime-phase-writes-no-journal.md) | A runtime hook phase MUST NOT journal. | approved |
| [REQ-3115](REQ-3115-runtime-phase-never-rolls-back.md) | A runtime hook phase MUST NOT roll back. | approved |
| [REQ-3116](REQ-3116-stale-module-clears-transitive-components.md) | Every component belonging to a module whose fingerprint changed MUST be cleared, including transitive ones `installed.lock` does not name directly. | approved |
| [REQ-3117](REQ-3117-engine-takes-no-renderer-field.md) | The engine MUST NOT take a renderer as a field. | approved |
| [REQ-3118](REQ-3118-engine-formats-no-output.md) | The engine MUST NOT format output. | approved |
| [REQ-3119](REQ-3119-no-writer-field-or-fallback.md) | Replaced by REQ-3051 and REQ-3117. | withdrawn |
| [REQ-3120](REQ-3120-partial-rollback-records-outcome.md) | A rollback that partially succeeds MUST record the `partial` outcome. | approved |
| [REQ-3121](REQ-3121-interrupt-keeps-journal-intact.md) | A run interrupted by a signal MUST leave the journal intact for the next run to find; see REQ-3042. | approved |
| [REQ-3140](REQ-3140-broken-local-star-stops-command.md) | A `local.star` that exists and can't be read or evaluated MUST stop the command before any phase runs. | approved |
| [REQ-3141](REQ-3141-discovery-failure-stops-before-phases.md) | A failure during discovery MUST stop the command before any phase runs. | approved |
| [REQ-3142](REQ-3142-dry-run-exits-non-zero-on-hook-failure.md) | A dry run MUST exit non-zero when a hook fails against the dry-run effects. | approved |
| [REQ-3172](REQ-3172-dry-run-runs-hooks-against-dry-run-effects.md) | A dry run of a phase set MUST run each planned hook against the dry-run effects after it renders the plan. | approved |
| [REQ-3201](REQ-3201-one-glyph-one-meaning.md) | One glyph MUST mean the same thing in every command. | approved |
| [REQ-3202](REQ-3202-output-uses-two-indent-levels.md) | Output MUST use at most two levels of indent: column zero for the command speaking about itself, two spaces for an item, and captured subprocess output indented under its item and dimmed. | approved |
| [REQ-3203](REQ-3203-colour-is-redundant.md) | Colour MUST be redundant. | approved |
| [REQ-3204](REQ-3204-no-sink-uses-raw-mode.md) | A sink MUST NOT put the terminal into raw mode. | approved |
| [REQ-3210](REQ-3210-four-sinks-share-one-stream.md) | Four sinks MUST consume the same stream: `LiveSink` for a capable terminal, `PlainSink` for everything else, `JsonSink` for `--format json`, and `ShellSink` for `meowctl hook`. | approved |
| [REQ-3211](REQ-3211-sink-chosen-once-per-command.md) | A sink MUST be chosen once per command, from the detected capabilities and the flags. | approved |
| [REQ-3212](REQ-3212-run-sinks-render-every-event.md) | Every sink that renders a run MUST render every event. | approved |
| [REQ-3213](REQ-3213-shell-sink-not-flag-selectable.md) | `ShellSink` MUST NOT be selectable by a flag. | approved |
| [REQ-3220](REQ-3220-live-sink-redraws-in-place.md) | `LiveSink` MUST redraw its region in place. | approved |
| [REQ-3221](REQ-3221-subprocess-output-appears-live.md) | Captured subprocess output MUST appear under its component while the process runs. | approved |
| [REQ-3222](REQ-3222-live-sink-yields-terminal.md) | On `TerminalRequested`, `LiveSink` MUST erase its region, restore the cursor, and write nothing until `TerminalReleased`; see REQ-1621. | approved |
| [REQ-3223](REQ-3223-cursor-restored-on-exit.md) | The cursor MUST be restored on exit, on interruption, and on suspension. | approved |
| [REQ-3224](REQ-3224-sub-work-on-status-line.md) | Sub-work within a component, such as packages being installed, MUST be shown on the component's own status line rather than by indenting further, so REQ-3202 holds. | approved |
| [REQ-3225](REQ-3225-width-reread-per-frame.md) | The terminal width MUST be re-read per frame, so a resize is picked up without a signal handler. | approved |
| [REQ-3226](REQ-3226-spinner-advances-on-events.md) | The spinner MUST advance on the events the sink receives. | approved |
| [REQ-3230](REQ-3230-json-sink-one-object-per-line.md) | `JsonSink` MUST emit one JSON object per event, one per line, in the order the events arrived. | approved |
| [REQ-3231](REQ-3231-json-sink-emits-no-decoration.md) | `JsonSink` MUST emit no colour, no glyph, and no padding. | approved |
| [REQ-3232](REQ-3232-every-command-supports-json.md) | Every command MUST support `--format json`. | approved |
| [REQ-3240](REQ-3240-detection-resolves-four-capabilities.md) | Detection MUST resolve four things independently: whether the destination is a terminal, whether motion is allowed, whether the locale advertises UTF-8, and the colour depth. | approved |
| [REQ-3241](REQ-3241-motion-disabled-pipe-dumb-ci.md) | Motion MUST be disabled for a pipe, for `TERM` unset or `dumb`, and when `CI` is set, even when a pty is present. | approved |
| [REQ-3242](REQ-3242-no-color-disables-colour.md) | `NO_COLOR` MUST disable colour. | approved |
| [REQ-3243](REQ-3243-non-utf8-locale-selects-ascii.md) | A non-UTF-8 locale MUST select the ASCII glyph set. | approved |
| [REQ-3244](REQ-3244-meowctl-output-overrides-detection.md) | `MEOWCTL_OUTPUT` MUST override detection with `live` or `plain`. | approved |
| [REQ-3245](REQ-3245-colorterm-selects-colour-depth.md) | `COLORTERM` naming `truecolor` or `24bit` MUST select the 24-bit depth, and a `TERM` containing `256` the 256-colour one. | approved |
| [REQ-3246](REQ-3246-clicolor-force-colours-pipe.md) | `CLICOLOR_FORCE`, set to anything but `0`, MUST turn colour back on for a destination that is not a terminal. | approved |
| [REQ-3250](REQ-3250-palette-is-data.md) | The palette MUST be data rather than colours at call sites, with the Catppuccin values as the built-in default. | approved |
| [REQ-3251](REQ-3251-call-sites-name-roles.md) | Call sites MUST name a role, never a colour, so the palette changes in one place. | approved |
| [REQ-3252](REQ-3252-malformed-theme-warns-falls-back.md) | A theme file that is malformed MUST warn and fall back to the default, not fail the command. | approved |
| [REQ-3253](REQ-3253-theme-file-in-config-dir.md) | The theme file MUST be `theme.toml` in the configuration directory. | approved |
| [REQ-3254](REQ-3254-unnamed-role-keeps-default.md) | A role the theme file does not name MUST keep its default, so a user who wants one colour changed writes one table. | approved |
| [REQ-3255](REQ-3255-theme-parsing-reads-no-file.md) | Theme parsing MUST NOT read a file. | approved |
| [REQ-3256](REQ-3256-theme-read-once-before-sink.md) | The theme MUST be read once, before the sink is built. | approved |
| [REQ-3260](REQ-3260-prompting-uses-interaction-trait.md) | Prompting MUST go through an `Interaction` trait, separate from rendering. | approved |
| [REQ-3261](REQ-3261-confirmation-asks-on-stderr.md) | A confirmation MUST send its question to stderr. | approved |
| [REQ-3262](REQ-3262-non-interactive-prompt-fails.md) | In a non-interactive session, a prompt MUST fail with a message naming what it wanted, rather than blocking. | approved |
| [REQ-3263](REQ-3263-interaction-asks-free-text.md) | The `Interaction` trait MUST also ask a free-text question and return the answer, because `ctx.prompt` returns a string; see REQ-2826. | approved |
| [REQ-3270](REQ-3270-failed-write-never-panics.md) | A write to the output destination that fails MUST NOT panic. | approved |
| [REQ-3271](REQ-3271-new-event-variant-needs-handling.md) | Adding a variant to `Event` MUST NOT compile until every sink that renders a run handles it. | approved |
| [REQ-3280](REQ-3280-sinks-testable-from-recorded-stream.md) | Every sink MUST be testable against a recorded event stream with no terminal, no subprocess, and no filesystem. | approved |
| [REQ-3281](REQ-3281-snapshots-cover-capability-matrix.md) | Snapshots MUST cover the capability matrix: motion, colour depth, glyph tier, and width. | approved |
| [REQ-3300](REQ-3300-state-legible-without-colour.md) | Every state MUST be legible from its symbol and its wording alone, so losing colour loses decoration and never information. | approved |
| [REQ-3301](REQ-3301-sink-never-changes-mid-run.md) | A sink MUST NOT change mid-run. | approved |
| [REQ-3302](REQ-3302-shell-sink-writes-lines-verbatim.md) | `ShellSink` MUST write each `ShellLine` verbatim, with no decoration, no indent and no colour. | approved |
| [REQ-3303](REQ-3303-shell-sink-ignores-other-events.md) | `ShellSink` MUST write nothing for any event other than `ShellLine`, because whatever it wrote the shell would evaluate; see REQ-3461 and REQ-1044. | approved |
| [REQ-3304](REQ-3304-hook-json-produces-event-stream.md) | `--format json` on `hook` MUST still produce the event stream, because a program reading events is not a shell evaluating them. | approved |
| [REQ-3305](REQ-3305-live-sink-erases-before-writing.md) | `LiveSink` MUST erase its region before writing anything permanent, issuing exactly one cursor-up per drawn line. | approved |
| [REQ-3306](REQ-3306-sink-restores-cursor-on-drop.md) | The sink MUST restore the cursor when it is finished with and when it is dropped, which covers a normal exit and a panic. | approved |
| [REQ-3307](REQ-3307-spinner-has-no-timer.md) | The spinner MUST NOT be driven by a timer. | approved |
| [REQ-3308](REQ-3308-colour-downsampled-to-reported-depth.md) | Colour MUST be downsampled to the depth the terminal reports rather than assumed. | approved |
| [REQ-3309](REQ-3309-ascii-glyphs-keep-distinctions.md) | The ASCII glyph set MUST carry the same distinctions as the Unicode one. | approved |
| [REQ-3310](REQ-3310-unknown-output-mode-falls-back.md) | An unrecognised `MEOWCTL_OUTPUT` value MUST fall back to detection rather than erroring. | approved |
| [REQ-3311](REQ-3311-other-terminals-get-sixteen-colours.md) | Anything other than a `COLORTERM` naming `truecolor` or `24bit` or a `TERM` containing `256` MUST be the 16-colour depth. | approved |
| [REQ-3312](REQ-3312-no-color-beats-clicolor-force.md) | `NO_COLOR` MUST still win over `CLICOLOR_FORCE`. | approved |
| [REQ-3313](REQ-3313-user-can-supply-palette.md) | A user MUST be able to point at their own palette. | approved |
| [REQ-3314](REQ-3314-theme-table-per-role.md) | The theme file MUST be a table per role naming `r`, `g`, `b` and `ansi16`. | approved |
| [REQ-3315](REQ-3315-named-role-replaced-whole.md) | A role the theme file names MUST be replaced whole. | approved |
| [REQ-3316](REQ-3316-failed-theme-read-keeps-default.md) | A theme read that fails for any reason -- absent, unreadable, malformed -- MUST leave the default in place. | approved |
| [REQ-3317](REQ-3317-absent-theme-is-silent.md) | An absent theme file MUST be silent. | approved |
| [REQ-3318](REQ-3318-unreadable-theme-warns.md) | An unreadable or malformed theme file MUST warn. | approved |
| [REQ-3319](REQ-3319-confirmation-defaults-to-no.md) | A confirmation MUST treat a bare Enter and end-of-input as no. | approved |
| [REQ-3320](REQ-3320-only-explicit-yes-confirms.md) | A confirmation MUST treat only an explicit yes as yes. | approved |
| [REQ-3321](REQ-3321-free-text-asks-on-stderr.md) | The `Interaction` trait MUST send a free-text question to stderr for the same reason a confirmation does. | approved |
| [REQ-3322](REQ-3322-free-text-eof-is-empty.md) | The `Interaction` trait MUST treat end-of-input on a free-text question as an empty answer, which is what `starPrompt` does. | approved |
| [REQ-3323](REQ-3323-failed-write-keeps-run-going.md) | A write to the output destination that fails MUST NOT abort a run in progress. | approved |
| [REQ-3340](REQ-3340-live-sink-still-when-too-narrow.md) | The live sink MUST NOT move the cursor while the terminal is narrower than its minimum width. | approved |
| [REQ-3341](REQ-3341-prompt-waits-only-after-visible-question.md) | A prompt MUST NOT wait for input unless its question was written to a terminal. | approved |
| [REQ-3342](REQ-3342-zero-width-keeps-known-layout.md) | When the terminal reports a zero or unknown width, the live sink MUST keep laying out at a width it last knew. | approved |
| [REQ-3370](REQ-3370-live-row-fits-terminal-width.md) | A row the live sink draws MUST NOT exceed the terminal width. | approved |
| [REQ-3401](REQ-3401-exact-v010-command-set.md) | The command set MUST be exactly what `newRootCmd` in `internal/cli/root.go` registers: `init`, `apply`, `add`, `remove`, `upgrade`, `verify`, `update`, `status`, `doctor`, `check`, `hook`, `shell`, `self-update`, `version`, the `dep` group, and shell completions. | approved |
| [REQ-3402](REQ-3402-dep-group-six-subcommands.md) | The `dep` group MUST carry `list`, `add`, `remove`, `upgrade`, `sync`, and `tidy`. | approved |
| [REQ-3403](REQ-3403-init-scaffolds-config-directory.md) | `init` with no argument MUST scaffold a config directory. | approved |
| [REQ-3404](REQ-3404-config-flag-overrides-resolution.md) | `--config` MUST override the config directory resolution in [REQ-1020]. | approved |
| [REQ-3405](REQ-3405-lifecycle-commands-accept-shared-flags.md) | Each lifecycle command MUST accept `--dry-run`, `--verbose` and `--no-rollback`. | approved |
| [REQ-3406](REQ-3406-format-json-on-every-command.md) | `--format json` MUST be available on every command. | approved |
| [REQ-3410](REQ-3410-effects-constructed-once-in-cli.md) | The `FileSystem`, the `Executor`, and the sink MUST be constructed here, once, from the parsed flags, and passed down. No crate below constructs one; see [REQ-1414]. | approved |
| [REQ-3411](REQ-3411-dry-run-selects-dry-run-effects.md) | `--dry-run` MUST select `DryRunFs`, and a dry-run `Executor` for each phase that runs. | approved |
| [REQ-3412](REQ-3412-command-tree-built-per-invocation.md) | Commands MUST be constructed per invocation. | approved |
| [REQ-3413](REQ-3413-version-string-from-build.md) | The version string MUST come from the build rather than from variables patched by linker flags. | approved |
| [REQ-3414](REQ-3414-cli-installs-interrupt-handler.md) | The binary MUST install a handler for interruption and suspension that finishes the sink before the process stops, so the cursor is restored; see [REQ-3223]. | approved |
| [REQ-3415](REQ-3415-verify-accepts-no-rollback.md) | `--no-rollback` on `verify` MUST be accepted. | approved |
| [REQ-3416](REQ-3416-ignore-lock-accepted.md) | `--ignore-lock` MUST be accepted. | approved |
| [REQ-3420](REQ-3420-commands-write-through-sink.md) | Every command MUST write through the sink. | approved |
| [REQ-3421](REQ-3421-shell-snippet-to-stdout.md) | `shell <shell>` MUST emit its integration snippet to stdout, because the output is evaluated by the shell. | approved |
| [REQ-3422](REQ-3422-dry-run-renders-plan.md) | A dry run MUST render the `Plan`. | approved |
| [REQ-3430](REQ-3430-exit-code-mapping-in-cli.md) | Mapping an error to an exit code MUST happen here and only here, using the taxonomy in [REQ-1030]. | approved |
| [REQ-3431](REQ-3431-span-errors-render-as-diagnostics.md) | An error carrying a source span MUST be rendered as a diagnostic showing the file, the line, and the offending source text; see [REQ-2240]. | approved |
| [REQ-3432](REQ-3432-usage-error-prints-usage-exits-2.md) | A usage error MUST print usage and exit 2. | approved |
| [REQ-3433](REQ-3433-errors-go-to-stderr.md) | An error MUST go to stderr. | approved |
| [REQ-3450](REQ-3450-missing-config-suggests-init.md) | A command that needs a configuration and is run outside one MUST say so and suggest `meowctl init`, rather than reporting a missing file. | approved |
| [REQ-3451](REQ-3451-interrupt-reaches-engine.md) | An interrupt MUST reach the engine, which stops cleanly; see [REQ-3062]. | approved |
| [REQ-3452](REQ-3452-unknown-command-usage-error.md) | An unknown command or flag MUST exit 2 with the usage error. | approved |
| [REQ-3453](REQ-3453-second-interrupt-stops-at-once.md) | A second interrupt MUST stop the process at once, with the default disposition. | approved |
| [REQ-3454](REQ-3454-interrupted-run-says-so.md) | A run that stopped because it was interrupted MUST say so. | approved |
| [REQ-3460](REQ-3460-hook-accepts-shell-login-only.md) | `hook` MUST accept `shell` and `login` and no other phase. | approved |
| [REQ-3461](REQ-3461-hook-runs-phase-in-graph-order.md) | `hook` MUST run the named phase for every discovered component, in graph order. | approved |
| [REQ-3462](REQ-3462-hook-failure-recorded-in-flag-file.md) | A failure to evaluate a component or to run a hook MUST be recorded in `.hook-error`. | approved |
| [REQ-3463](REQ-3463-clean-hook-run-removes-flag.md) | A run in which nothing failed MUST remove `.hook-error`. | approved |
| [REQ-3464](REQ-3464-status-doctor-report-hook-flag.md) | `status` and `doctor` MUST report the flag when it is present. | approved |
| [REQ-3465](REQ-3465-hook-does-not-journal.md) | `hook` MUST NOT journal what a hook does. | approved |
| [REQ-3470](REQ-3470-self-update-refuses-unverified-binary.md) | `self-update` MUST NOT install a binary it has not verified. | approved |
| [REQ-3471](REQ-3471-release-publishes-checksums-sri.md) | A release MUST publish `checksums.sri`, one line per asset, each `<sri> <asset name>`. | approved |
| [REQ-3472](REQ-3472-self-update-asks-latest-release.md) | `self-update` MUST ask for the latest release. | approved |
| [REQ-3473](REQ-3473-asset-named-for-this-platform.md) | The asset MUST be the one named for this platform. | approved |
| [REQ-3474](REQ-3474-atomic-binary-replacement.md) | Replacing the running binary MUST be atomic: written beside it, made executable, then renamed over it. | approved |
| [REQ-3475](REQ-3475-self-update-runs-only-alone.md) | `self-update` MUST NOT run as part of anything else. | approved |
| [REQ-3476](REQ-3476-releases-env-overrides-endpoint.md) | `MEOWCTL_RELEASES` MUST override where the latest release is asked for. | approved |
| [REQ-3500](REQ-3500-init-bootstraps-from-repository-url.md) | `init <repo-url>` MUST bootstrap from a public dotfiles repository over HTTPS with no `git` subprocess. | approved |
| [REQ-3501](REQ-3501-verbose-flag-on-every-command.md) | `--verbose` MUST be available on every command. | approved |
| [REQ-3502](REQ-3502-install-commands-accept-force-ignore-lock.md) | `apply`, `add` and `upgrade` MUST additionally accept `--force` and `--ignore-lock`, matching `v0.1.0`. | approved |
| [REQ-3503](REQ-3503-format-json-selects-json-sink.md) | `--format json` MUST select `JsonSink`; see [REQ-3232]. | approved |
| [REQ-3504](REQ-3504-json-alias-on-doctor-status.md) | `--json` MUST remain accepted as an alias on `doctor` and `status`, which are the two commands that had it. | approved |
| [REQ-3505](REQ-3505-dry-run-not-passed-as-boolean.md) | Code below `meowctl-cli` MUST NOT consult the dry-run flag except to construct an effect implementation or to report the flag to a hook. | approved |
| [REQ-3506](REQ-3506-no-dry-run-branch-below-cli.md) | Code below `meowctl-cli` MUST NOT branch on the dry-run flag while it performs an effect; see [REQ-2814]. | approved |
| [REQ-3507](REQ-3507-version-string-matches-v010-form.md) | The version string MUST report the version, the target, the commit, and the build date, in the form `internal/version/version.go` produces. | approved |
| [REQ-3508](REQ-3508-verify-no-rollback-changes-nothing.md) | `--no-rollback` on `verify` MUST change nothing. | approved |
| [REQ-3509](REQ-3509-ignore-lock-changes-nothing.md) | `--ignore-lock` MUST change nothing, which is what `v0.1.0` does. | approved |
| [REQ-3510](REQ-3510-no-direct-stdout-stderr-writes.md) | A command MUST NOT write to stdout or stderr directly; the workspace lint denies it outside this crate, and `meowctl-tui` owns the exception. | approved |
| [REQ-3511](REQ-3511-sink-renders-snippet-verbatim.md) | The sink MUST render the integration snippet verbatim. | approved |
| [REQ-3512](REQ-3512-dry-run-reports-skips-with-reasons.md) | A dry run MUST report every skip with its reason; see [REQ-3052]. | approved |
| [REQ-3513](REQ-3513-work-failure-omits-usage.md) | A failure in the work MUST NOT print usage, because a wall of flags on top of a real error buries it. | approved |
| [REQ-3514](REQ-3514-errors-keep-piped-stdout-clean.md) | An error MUST NOT corrupt a piped stdout. | approved |
| [REQ-3515](REQ-3515-missing-config-exits-configuration-code.md) | A command that needs a configuration and is run outside one MUST exit with the configuration code. | approved |
| [REQ-3516](REQ-3516-reporting-commands-do-not-refuse.md) | A command that only reports MUST NOT refuse. | approved |
| [REQ-3517](REQ-3517-sink-restores-terminal-on-interrupt.md) | The sink MUST restore the terminal on the way out; see [REQ-3223]. | approved |
| [REQ-3518](REQ-3518-suggest-nearest-command.md) | An unknown command MUST suggest the nearest command when one is close. | approved |
| [REQ-3519](REQ-3519-interrupted-run-exits-general-code.md) | A run that stopped because it was interrupted MUST exit with the general code. | approved |
| [REQ-3520](REQ-3520-unsupported-hook-phase-usage-error.md) | An unsupported phase MUST exit 2 with the usage error, naming the two that work; see [REQ-1013]. | approved |
| [REQ-3521](REQ-3521-hook-stdout-carries-only-emit.md) | `hook` MUST write what `ctx.emit` contributed to stdout and nothing else. | approved |
| [REQ-3522](REQ-3522-hook-failure-keeps-exit-zero.md) | A failure to evaluate a component or to run a hook MUST NOT change the exit code, which stays 0. | approved |
| [REQ-3523](REQ-3523-status-says-hook-failed.md) | `status` MUST say that the hook run failed and where to look. | approved |
| [REQ-3524](REQ-3524-doctor-renders-hook-error-text.md) | `doctor` MUST render the text recorded in `.hook-error`. | approved |
| [REQ-3525](REQ-3525-hook-flag-at-warning-level.md) | `status` and `doctor` MUST use the warning level, which is what a reader scans for. | approved |
| [REQ-3526](REQ-3526-hook-does-not-roll-back.md) | `hook` MUST NOT roll back. | approved |
| [REQ-3527](REQ-3527-self-update-checks-release-checksum.md) | `self-update` MUST check the integrity of what it downloaded against a checksum the release publishes. | approved |
| [REQ-3528](REQ-3528-self-update-refuses-on-mismatch.md) | `self-update` MUST refuse rather than replace the running binary when the download and the published checksum disagree; see [REQ-1005] and [REQ-2430]. | approved |
| [REQ-3529](REQ-3529-self-update-refuses-without-checksums.md) | `self-update` MUST refuse to proceed when `checksums.sri` is absent. | approved |
| [REQ-3530](REQ-3530-self-update-compares-tag-with-version.md) | `self-update` MUST compare the tag of the latest release with the running version. | approved |
| [REQ-3531](REQ-3531-self-update-no-op-when-current.md) | `self-update` MUST report being up to date and change nothing when the tag and the running version match. | approved |
| [REQ-3532](REQ-3532-missing-asset-names-platform.md) | A release with no asset for this platform MUST say which platform was looked for and where the release is, rather than reporting a network failure. | approved |
| [REQ-3540](REQ-3540-network-failure-keeps-running-binary.md) | A network failure during `self-update` MUST leave the running binary unchanged. | approved |
| [REQ-3541](REQ-3541-failed-init-leaves-directory-unchanged.md) | A failed `init <repo-url>` MUST leave the configuration directory as it was before the command. | approved |
| [REQ-3542](REQ-3542-hook-reports-unwritable-flag.md) | When `hook` can't record or clear `.hook-error`, it MUST say so on stderr. | approved |
| [REQ-3543](REQ-3543-failed-replacement-removes-staged-file.md) | A replacement of the running binary that fails at any step MUST remove the staged file. | approved |
| [REQ-3570](REQ-3570-release-publishes-four-targets.md) | Each release MUST publish a binary for `x86_64-unknown-linux-gnu`, `aarch64-apple-darwin`, `x86_64-apple-darwin` and `x86_64-pc-windows-msvc`. | approved |
| [REQ-3571](REQ-3571-hook-shell-opens-no-connection.md) | `meowctl hook shell` MUST NOT open a network connection or start an asynchronous runtime. | approved |
| [REQ-3572](REQ-3572-self-update-download-bounded-at-five-minutes.md) | `self-update` MUST allow its binary download to take up to 5 minutes end to end. | approved |
| [REQ-3590](REQ-3590-unknown-flag-suggests-nearest-flag.md) | An unknown flag MUST suggest the nearest flag when one is close. | approved |

By topic:

- cli: REQ-3401, REQ-3402, REQ-3403, REQ-3404, REQ-3405, REQ-3406, REQ-3410, REQ-3411, REQ-3412, REQ-3413, REQ-3414, REQ-3415, REQ-3416, REQ-3420, REQ-3421, REQ-3422, REQ-3430, REQ-3431, REQ-3432, REQ-3433, REQ-3450, REQ-3451, REQ-3452, REQ-3453, REQ-3454, REQ-3460, REQ-3461, REQ-3462, REQ-3463, REQ-3464, REQ-3465, REQ-3470, REQ-3471, REQ-3472, REQ-3473, REQ-3474, REQ-3475, REQ-3476, REQ-3500, REQ-3501, REQ-3502, REQ-3503, REQ-3504, REQ-3505, REQ-3506, REQ-3507, REQ-3508, REQ-3509, REQ-3510, REQ-3511, REQ-3512, REQ-3513, REQ-3514, REQ-3515, REQ-3516, REQ-3517, REQ-3518, REQ-3519, REQ-3520, REQ-3521, REQ-3522, REQ-3523, REQ-3524, REQ-3525, REQ-3526, REQ-3527, REQ-3528, REQ-3529, REQ-3530, REQ-3531, REQ-3532, REQ-3540, REQ-3541, REQ-3542, REQ-3543, REQ-3570, REQ-3571, REQ-3572, REQ-3590
- common: REQ-1001, REQ-1002, REQ-1003, REQ-1004, REQ-1005, REQ-1006, REQ-1010, REQ-1011, REQ-1012, REQ-1013, REQ-1020, REQ-1021, REQ-1022, REQ-1023, REQ-1030, REQ-1031, REQ-1032, REQ-1040, REQ-1041, REQ-1042, REQ-1043, REQ-1044, REQ-1050, REQ-1051, REQ-1100, REQ-1101, REQ-1102, REQ-1103, REQ-1104, REQ-1105, REQ-1106, REQ-1107, REQ-1108, REQ-1109, REQ-1110
- config: REQ-1201, REQ-1202, REQ-1203, REQ-1204, REQ-1210, REQ-1211, REQ-1212, REQ-1213, REQ-1214, REQ-1220, REQ-1221, REQ-1222, REQ-1223, REQ-1224, REQ-1225, REQ-1226, REQ-1227, REQ-1230, REQ-1231, REQ-1232, REQ-1233, REQ-1240, REQ-1241, REQ-1242, REQ-1243, REQ-1244, REQ-1250, REQ-1251, REQ-1252, REQ-1253, REQ-1260, REQ-1261, REQ-1262, REQ-1263, REQ-1264, REQ-1300, REQ-1301, REQ-1302, REQ-1303, REQ-1304, REQ-1305, REQ-1306, REQ-1307, REQ-1308, REQ-1309, REQ-1310, REQ-1311, REQ-1312, REQ-1313, REQ-1314, REQ-1315, REQ-1316
- ctx: REQ-2801, REQ-2802, REQ-2803, REQ-2810, REQ-2811, REQ-2812, REQ-2813, REQ-2814, REQ-2820, REQ-2821, REQ-2822, REQ-2823, REQ-2824, REQ-2825, REQ-2826, REQ-2827, REQ-2828, REQ-2830, REQ-2831, REQ-2832, REQ-2840, REQ-2841, REQ-2842, REQ-2843, REQ-2844, REQ-2872, REQ-2900, REQ-2901, REQ-2902, REQ-2903, REQ-2904, REQ-2905, REQ-2906, REQ-2907, REQ-2908, REQ-2909, REQ-2910, REQ-2911, REQ-2912, REQ-2940, REQ-2941
- engine: REQ-3001, REQ-3002, REQ-3003, REQ-3010, REQ-3011, REQ-3012, REQ-3013, REQ-3014, REQ-3015, REQ-3016, REQ-3017, REQ-3018, REQ-3020, REQ-3021, REQ-3030, REQ-3031, REQ-3032, REQ-3033, REQ-3034, REQ-3035, REQ-3040, REQ-3041, REQ-3042, REQ-3043, REQ-3044, REQ-3050, REQ-3051, REQ-3052, REQ-3060, REQ-3061, REQ-3062, REQ-3063, REQ-3064, REQ-3100, REQ-3101, REQ-3102, REQ-3103, REQ-3104, REQ-3105, REQ-3106, REQ-3107, REQ-3108, REQ-3109, REQ-3110, REQ-3111, REQ-3112, REQ-3113, REQ-3114, REQ-3115, REQ-3116, REQ-3117, REQ-3118, REQ-3119, REQ-3120, REQ-3121, REQ-3140, REQ-3141, REQ-3142, REQ-3172
- exec: REQ-1601, REQ-1602, REQ-1603, REQ-1604, REQ-1605, REQ-1606, REQ-1610, REQ-1611, REQ-1612, REQ-1620, REQ-1621, REQ-1622, REQ-1623, REQ-1630, REQ-1631, REQ-1632, REQ-1700, REQ-1701, REQ-1702, REQ-1703, REQ-1704, REQ-1770
- fs: REQ-1401, REQ-1402, REQ-1403, REQ-1404, REQ-1405, REQ-1406, REQ-1410, REQ-1411, REQ-1412, REQ-1413, REQ-1414, REQ-1420, REQ-1421, REQ-1422, REQ-1430, REQ-1431, REQ-1432, REQ-1433, REQ-1500, REQ-1501, REQ-1502, REQ-1503, REQ-1504, REQ-1505, REQ-1506, REQ-1507, REQ-1508, REQ-1570
- module: REQ-2401, REQ-2402, REQ-2403, REQ-2404, REQ-2405, REQ-2410, REQ-2411, REQ-2412, REQ-2413, REQ-2420, REQ-2421, REQ-2422, REQ-2430, REQ-2431, REQ-2432, REQ-2433, REQ-2434, REQ-2440, REQ-2441, REQ-2442, REQ-2443, REQ-2444, REQ-2445, REQ-2450, REQ-2451, REQ-2452, REQ-2453, REQ-2460, REQ-2461, REQ-2462, REQ-2463, REQ-2464, REQ-2500, REQ-2501, REQ-2502, REQ-2503, REQ-2504, REQ-2505, REQ-2506, REQ-2507, REQ-2508, REQ-2509, REQ-2510, REQ-2511, REQ-2512, REQ-2513, REQ-2514, REQ-2540
- net: REQ-1801, REQ-1802, REQ-1803, REQ-1804, REQ-1805, REQ-1806, REQ-1810, REQ-1811, REQ-1812, REQ-1813, REQ-1814, REQ-1900, REQ-1901, REQ-1902, REQ-1903, REQ-1904, REQ-1905
- ops: REQ-2001, REQ-2002, REQ-2003, REQ-2004, REQ-2005, REQ-2010, REQ-2011, REQ-2012, REQ-2013, REQ-2014, REQ-2015, REQ-2016, REQ-2017, REQ-2020, REQ-2021, REQ-2022, REQ-2023, REQ-2024, REQ-2025, REQ-2026, REQ-2030, REQ-2031, REQ-2032, REQ-2100, REQ-2101, REQ-2102, REQ-2103, REQ-2104, REQ-2105, REQ-2106, REQ-2107, REQ-2108, REQ-2109, REQ-2110, REQ-2111, REQ-2112, REQ-2113, REQ-2114, REQ-2115, REQ-2116, REQ-2117, REQ-2118, REQ-2119, REQ-2120, REQ-2140, REQ-2141, REQ-2190
- pm: REQ-2601, REQ-2602, REQ-2603, REQ-2604, REQ-2610, REQ-2611, REQ-2612, REQ-2613, REQ-2614, REQ-2615, REQ-2620, REQ-2621, REQ-2630, REQ-2631, REQ-2632, REQ-2700, REQ-2701, REQ-2702, REQ-2703, REQ-2704
- starlark: REQ-2201, REQ-2202, REQ-2203, REQ-2204, REQ-2205, REQ-2206, REQ-2207, REQ-2208, REQ-2210, REQ-2211, REQ-2212, REQ-2220, REQ-2221, REQ-2222, REQ-2223, REQ-2230, REQ-2231, REQ-2232, REQ-2233, REQ-2234, REQ-2240, REQ-2241, REQ-2242, REQ-2250, REQ-2251, REQ-2252, REQ-2253, REQ-2300, REQ-2301, REQ-2302, REQ-2303, REQ-2304, REQ-2305, REQ-2306, REQ-2307, REQ-2340, REQ-2341, REQ-2342, REQ-2343, REQ-2370
- tui: REQ-3201, REQ-3202, REQ-3203, REQ-3204, REQ-3210, REQ-3211, REQ-3212, REQ-3213, REQ-3220, REQ-3221, REQ-3222, REQ-3223, REQ-3224, REQ-3225, REQ-3226, REQ-3230, REQ-3231, REQ-3232, REQ-3240, REQ-3241, REQ-3242, REQ-3243, REQ-3244, REQ-3245, REQ-3246, REQ-3250, REQ-3251, REQ-3252, REQ-3253, REQ-3254, REQ-3255, REQ-3256, REQ-3260, REQ-3261, REQ-3262, REQ-3263, REQ-3270, REQ-3271, REQ-3280, REQ-3281, REQ-3300, REQ-3301, REQ-3302, REQ-3303, REQ-3304, REQ-3305, REQ-3306, REQ-3307, REQ-3308, REQ-3309, REQ-3310, REQ-3311, REQ-3312, REQ-3313, REQ-3314, REQ-3315, REQ-3316, REQ-3317, REQ-3318, REQ-3319, REQ-3320, REQ-3321, REQ-3322, REQ-3323, REQ-3340, REQ-3341, REQ-3342, REQ-3370
<!-- /meow-method index -->
