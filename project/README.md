# meowctl record

<!-- meow-method index -->

13 specifications in all: 13 live.

| Identifier | What it concluded | Status |
| --- | --- | --- |
| [SPC-1000](specs/SPC-1000-common-vocabulary.md) | The common vocabulary every crate shares | live |
| [SPC-1200](specs/SPC-1200-configuration-and-formats.md) | Configuration files and their on-disk formats | live |
| [SPC-1400](specs/SPC-1400-filesystem.md) | The filesystem trait and its three implementations | live |
| [SPC-1600](specs/SPC-1600-process-execution.md) | Process execution | live |
| [SPC-1800](specs/SPC-1800-network.md) | The network | live |
| [SPC-2000](specs/SPC-2000-reversible-operations.md) | Reversible operations | live |
| [SPC-2200](specs/SPC-2200-starlark-evaluation.md) | Starlark evaluation | live |
| [SPC-2400](specs/SPC-2400-module-resolution.md) | Module resolution | live |
| [SPC-2600](specs/SPC-2600-package-managers.md) | Package managers | live |
| [SPC-2800](specs/SPC-2800-the-ctx-object.md) | The ctx object | live |
| [SPC-3000](specs/SPC-3000-lifecycle-engine.md) | Lifecycle engine | live |
| [SPC-3200](specs/SPC-3200-terminal-output.md) | Terminal output | live |
| [SPC-3400](specs/SPC-3400-command-line-interface.md) | The command-line interface | live |
<!-- /meow-method index -->

## Epics

The generated block above holds the specification index, because the method
writes one index block per file and three kinds share this one.

| Identifier | What it realises | Status |
| --- | --- | --- |
| [EPC-0001](epics/EPC-0001-ship-0-2-0.md) | Ship 0.2.0, realising ADR-0001 | approved |

## Defects

| Identifier | What is wrong | Status |
| --- | --- | --- |
| [BUG-0001](bugs/BUG-0001-journal-never-truncated.md) | The rollback journal is never truncated, so a failed run undoes an earlier successful run | approved |
| [BUG-0002](bugs/BUG-0002-linux-distribution-never-read.md) | meowctl never reads the Linux distribution, so every `distros` guard and distribution case fails on Linux | approved |
| [BUG-0003](bugs/BUG-0003-cli-ignores-error-severity.md) | The CLI ignores the severity lower crates compute, so a module that won't fetch during `apply` exits 3 | approved |
| [BUG-0004](bugs/BUG-0004-legacy-layout-reported-as-unconfigured.md) | `require_configured` hides the legacy-layout message and I/O errors behind "run `meowctl init`" | approved |
| [BUG-0005](bugs/BUG-0005-init-writes-archive-unstaged.md) | `init <repo-url>` writes the archive straight into the configuration root, so a failure partway leaves files behind | approved |
| [BUG-0006](bugs/BUG-0006-self-update-leaves-staged-binary.md) | `self-update` leaves the staged binary behind when it can't make it executable | approved |
| [BUG-0007](bugs/BUG-0007-unreadable-star-file-skipped.md) | An `init.star` or `local.star` that exists but can't be read is skipped as if absent | approved |
| [BUG-0008](bugs/BUG-0008-recording-failure-hides-run-failure.md) | A failure recording packages after a failed run replaces the run's failure | approved |
| [BUG-0009](bugs/BUG-0009-narrow-terminal-laid-out-at-80.md) | A terminal narrower than 20 columns is laid out at 80, so every live frame wraps | approved |
| [BUG-0010](bugs/BUG-0010-prompt-waits-with-stderr-closed.md) | With stderr closed at startup, a prompt waits for an answer to a question nobody saw | approved |
| [BUG-0011](bugs/BUG-0011-hook-exits-3-on-stale-flag.md) | `hook` exits 3 and prints an error on every shell spawn when a stale `.hook-error` can't be removed | approved |
| [BUG-0012](bugs/BUG-0012-hook-drops-failed-flag-write.md) | `hook` drops a failed write of `.hook-error` silently | approved |
| [BUG-0013](bugs/BUG-0013-missing-github-ref-reported-as-status.md) | A GitHub ref that doesn't exist is reported as a bare HTTP 422 | approved |
| [BUG-0014](bugs/BUG-0014-self-update-repeats-url.md) | `self-update` names the failing URL twice | approved |
| [BUG-0015](bugs/BUG-0015-platform-has-no-architecture.md) | `platform()` carries no architecture attribute | approved |
| [BUG-0016](bugs/BUG-0016-top-level-query-pm-message-misleads.md) | A top-level `query_pm` blames the evaluation instead of saying it works only in a hook | approved |
| [BUG-0017](bugs/BUG-0017-interrogate-failure-omits-asking-component.md) | A raising `interrogate` doesn't name the asking component in its message | approved |
| [BUG-0018](bugs/BUG-0018-journal-found-unwritable-too-late.md) | An unwritable journal is found at the first reversible effect, after irreversible steps have run | approved |
| [BUG-0019](bugs/BUG-0019-torn-journal-record-swallows-next.md) | A torn journal record swallows the record appended after it | approved |
| [BUG-0020](bugs/BUG-0020-dry-run-runs-no-hook.md) | A dry run runs no hook, so a hook that would fail shows as planned | approved |
| [BUG-0021](bugs/BUG-0021-live-row-width-counts-characters.md) | The live sink counts characters rather than columns, so a wide character can wrap a row | approved |
| [BUG-0022](bugs/BUG-0022-downloads-share-the-thirty-second-bound.md) | `ctx.download` and `self-update` share the 30-second bound meant for module requests | approved |
| [BUG-0040](bugs/BUG-0040-cli-writes-to-the-terminal-directly.md) | `meowctl-cli` writes to stdout and stderr directly at three sites | approved |
