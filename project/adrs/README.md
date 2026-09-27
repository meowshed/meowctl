# Decisions

<!-- meow-method index -->

50 decisions in all: 50 approved.

| Identifier | What it concluded | Status |
| --- | --- | --- |
| [ADR-0001](ADR-0001-rewrite-rather-than-port.md) | Rewrite meowctl in Rust with the architecture corrected, rather than port the Go code | approved |
| [ADR-0002](ADR-0002-parity-is-the-contract.md) | Keep the Starlark API, the command surface and every format compatible with v0.1.0 | approved |
| [ADR-0003](ADR-0003-terminal-output-is-redesigned.md) | Redesign terminal output rather than port it, keeping v0.1.0's vocabulary | approved |
| [ADR-0004](ADR-0004-effects-are-traits.md) | Put effects behind traits constructed once at the binary | approved |
| [ADR-0005](ADR-0005-operations-are-data.md) | Make every reversible effect a variant of Op with an inverse | approved |
| [ADR-0006](ADR-0006-the-plan-is-a-value.md) | Compute the apply plan as a value, and execute exactly that value | approved |
| [ADR-0007](ADR-0007-staged-pipeline-types.md) | Make discovery, the graph, the plan and the report distinct types | approved |
| [ADR-0008](ADR-0008-no-async-runtime.md) | Use ureq and rustls with no async runtime | approved |
| [ADR-0009](ADR-0009-use-starlark-rust.md) | Evaluate Starlark with the starlark-rust crate rather than an evaluator of our own | approved |
| [ADR-0010](ADR-0010-no-raw-mode-no-tui-framework.md) | Render with terminal primitives only, and never take raw mode | approved |
| [ADR-0011](ADR-0011-engine-emits-events.md) | The engine emits typed events and holds no renderer | approved |
| [ADR-0012](ADR-0012-terminal-hand-off-through-events.md) | Negotiate terminal ownership through events, and delete SuspendOutput | approved |
| [ADR-0013](ADR-0013-universal-json-output.md) | Make --format json available on every command, as a sink over the event stream | approved |
| [ADR-0014](ADR-0014-structured-messages.md) | Carry everything worth printing as an event variant, and unstructured text only as Message with a severity | approved |
| [ADR-0015](ADR-0015-theme-as-data.md) | Hold the palette as data that call sites name by role | approved |
| [ADR-0016](ADR-0016-prompts-through-interaction.md) | Prompt through an Interaction trait that fails when nobody can answer | approved |
| [ADR-0017](ADR-0017-sub-work-on-status-line.md) | Show sub-work on the component's own status line rather than a third indent | approved |
| [ADR-0018](ADR-0018-snapshot-the-capability-matrix.md) | Snapshot every sink across the whole capability matrix | approved |
| [ADR-0019](ADR-0019-network-is-the-third-trait.md) | Put every network request behind one Http trait in meowctl-net | approved |
| [ADR-0020](ADR-0020-meowctl-config-variable.md) | Add $MEOWCTL_CONFIG ahead of the XDG config directory | approved |
| [ADR-0021](ADR-0021-directories-created-0700.md) | Create directories with mode 0700 | approved |
| [ADR-0022](ADR-0022-dry-run-reads-see-writes.md) | Answer a dry-run read from what the dry run recorded | approved |
| [ADR-0023](ADR-0023-dry-run-predicts-failure.md) | Fail a dry run where the real run would fail for a reason it can see | approved |
| [ADR-0024](ADR-0024-dry-run-executor-by-phase.md) | Choose whether a dry run runs a command by the phase that issued it | approved |
| [ADR-0025](ADR-0025-property-test-every-inverse.md) | Check apply-then-undo with a property test over every Op variant | approved |
| [ADR-0026](ADR-0026-refuse-a-newer-state-file.md) | Refuse to overwrite a state.toml written by a newer meowctl | approved |
| [ADR-0027](ADR-0027-edit-starlark-by-parsing.md) | Edit Starlark configuration files by parsing them, not by regular expression | approved |
| [ADR-0028](ADR-0028-refetch-a-stale-cache.md) | Re-fetch a cached module whose files don't match, rather than fail | approved |
| [ADR-0029](ADR-0029-starlark-error-text-is-the-librarys.md) | Accept starlark-rust's error messages rather than translate them | approved |
| [ADR-0030](ADR-0030-report-duplicate-manager-handlers.md) | Report two handlers that claim one package manager | approved |
| [ADR-0031](ADR-0031-filter-matching-nothing-fails.md) | Fail when a component filter matches nothing | approved |
| [ADR-0032](ADR-0032-every-skip-carries-a-reason.md) | Require a reason on every skip | approved |
| [ADR-0033](ADR-0033-commands-built-per-invocation.md) | Construct the command tree per invocation | approved |
| [ADR-0034](ADR-0034-version-from-build-metadata.md) | Take the version from build metadata, not linker flags | approved |
| [ADR-0035](ADR-0035-stop-a-phase-at-the-first-failure.md) | Stop a phase at the first component that fails | approved |
| [ADR-0036](ADR-0036-read-only-phases-get-a-read-only-ctx.md) | Give a read-only phase a ctx with no mutating methods | approved |
| [ADR-0037](ADR-0037-the-lock-records-its-writer.md) | Fill the lock's meta table, and leave meta out of the byte comparison | approved |
| [ADR-0038](ADR-0038-per-file-hashes-in-the-cache.md) | Record per-file hashes in the module cache, not in the lock | approved |
| [ADR-0039](ADR-0039-no-journal-in-a-dry-run.md) | Journal nothing during a dry run | approved |
| [ADR-0040](ADR-0040-malformed-theme-falls-back.md) | Warn and fall back to the default theme on a malformed theme file | approved |
| [ADR-0041](ADR-0041-config-parses-with-starlark-syntax.md) | Give meowctl-config the starlark_syntax parser as an external dependency | approved |
| [ADR-0042](ADR-0042-reproduce-the-v010-toml-layout.md) | Reproduce v0.1.0's TOML layout byte for byte rather than emit the toml crate's | approved |
| [ADR-0043](ADR-0043-package-managers-produce-a-call.md) | Have meowctl-pm produce a Call for the engine to make | approved |
| [ADR-0044](ADR-0044-signal-work-on-a-thread.md) | Handle signals in the binary, on a thread woken by signal-hook | approved |
| [ADR-0045](ADR-0045-delete-the-oracle-at-cutover.md) | Delete the compatibility corpus with the Go tree, and keep three fixtures | approved |
| [ADR-0046](ADR-0046-pathext-order-is-a-pure-function.md) | Compute the PATHEXT order as a pure function of the name and the variable | approved |
| [ADR-0047](ADR-0047-no-color-outranks-clicolor-force.md) | Let NO_COLOR win over CLICOLOR_FORCE | approved |
| [ADR-0048](ADR-0048-test-matrix-on-linux-only.md) | Run the test matrix on Linux only, while releases build for four targets | approved |
| [ADR-0049](ADR-0049-refuse-relative-and-user-paths.md) | Refuse relative paths and `~user` in path resolution | approved |
| [ADR-0050](ADR-0050-verify-before-self-update.md) | Verify a download before `self-update` installs it | approved |
<!-- /meow-method index -->
