# Reversible operations

**Crate:** `meowctl-ops`
**Design:** `docs/design/0.2.0-rust-rewrite.md` §3
**v0.1.0 equivalent:** `internal/rollback/` plus the effectful half of
`internal/ctx/methods.go`

## Scope

This component owns the closed set of reversible effects and the write-ahead
journal that makes a failed run undoable. An `Op` is data: it knows how to
apply itself against a [`FileSystem`](fs.md) and how to produce the `Op` that
undoes it.

It exists in this shape because of what `v0.1.0` makes easy to get wrong. There,
each `ctx` method performs its effect and separately calls the matching
`Append*` helper on the rollback stack, so a new method ships with no rollback
support and nothing notices until a failed run leaves a machine half
configured. Here, adding an effect means adding a variant, and the compiler
asks for its inverse.

## Boundary

The `Op` enum, `apply`, `inverse`, the journal file format, and replay.

## Operations

**[R-OPS-001]** `Op` MUST have exactly these nine variants: `WriteFile`,
`AppendFile`, `CopyFile`, `Symlink`, `LinkFile`, `Mkdir`, `Download`,
`DefaultsWrite`, and `PlistSet`.

`v0.1.0` journals the first seven. It declares `defaults_write` and `plist_set`
as `Kind*` constants, and `applyInverse` returns "inverse not implemented" for
both, but nothing ever appends a record of either kind: `internal/ctx` calls
neither `Append*` helper, and none exists. The two macOS operations are
unjournaled and irreversible today. [R-OPS-017] closes that, and it is a
deliberate change rather than a transcription.

**[R-OPS-002]** `Op::apply` MUST perform the effect through a `FileSystem` and
MUST NOT touch the filesystem by any other route. This is what makes a dry run
and an in-memory test possible.

**[R-OPS-003]** `Op::inverse` MUST be computed before the effect is applied,
because it depends on the state the effect is about to destroy: the prior
content of a file, the prior target of a symlink, whether a directory already
existed.

**[R-OPS-004]** Applying an `Op` and then applying its inverse MUST restore the
filesystem to the state it had before. This MUST hold for every variant, and a
property test over `MemFs` MUST check it.

**[R-OPS-005]** An `Op` that finds the system already in its target state MUST
still produce a correct inverse. Re-linking an already-correct symlink must not
journal a removal that would delete the user's working configuration.

## Inverses

**[R-OPS-010]** `WriteFile`'s inverse MUST restore the prior content when the
file existed, and MUST delete the file when it did not. `inverseWriteFile`
carries exactly this distinction, and the `had_prior` flag is what separates
them.

**[R-OPS-011]** `AppendFile` MUST wrap what it appends in begin and end markers
carrying an identifier, and its inverse MUST remove the block those markers
delimit rather than truncating the file. `inverseAppendFile` stores the marker;
`removeMarkedBlock` is the removal. The identifier MUST default to a generated
one and MUST be overridable by the caller, because `ctx.append_file` takes a
`marker` argument; see [R-CTX-028].

**[R-OPS-012]** `CopyFile`'s inverse MUST delete the destination.

**[R-OPS-013]** `Mkdir`'s inverse MUST remove the directory only when meowctl
created it, and MUST do nothing when it already existed.
`inverseMkdir.created_by_meowctl` is the flag.

**[R-OPS-014]** `Symlink`'s inverse MUST re-create the prior symlink when one
existed and MUST remove the link when none did.

**[R-OPS-015]** `LinkFile`'s inverse MUST remove the symlink and MUST restore
the backed-up original when a backup was taken. A backup that is not restored
is how a user loses a file they had before meowctl ran.

**[R-OPS-016]** `Download`'s inverse MUST restore prior content when the
destination existed and MUST delete the file when it did not.

**[R-OPS-017]** `DefaultsWrite` and `PlistSet` MUST record the prior value, and
their inverses MUST restore it. Where no prior value existed, the inverse MUST
delete the key rather than write an empty one. Reading the prior value MUST be
part of computing the inverse, per [R-OPS-003], because `defaults read` after
the write returns the new value.

This is new behaviour. In `v0.1.0` both operations apply with no journal entry
at all, so a failed run leaves the user's system preferences changed with no
record of what they were.

## The journal

**[R-OPS-020]** The journal MUST be a file of newline-delimited JSON records,
one per operation, in the format `internal/rollback/rollback.go` writes: a
sequence number, the phase, the component, the kind, and the inverse payload. A
journal written by `v0.1.0` MUST replay under `v0.2.0` and the reverse.

**[R-OPS-021]** A record MUST be appended and flushed to disk before its
operation is applied. A crash between the append and the effect leaves a
journal that replays a no-op, which is safe; the reverse order leaves an effect
with no undo, which is not.

**[R-OPS-022]** Replay MUST apply inverses in reverse sequence order.

**[R-OPS-023]** Replay MUST continue after an individual inverse fails, and
MUST report which ones failed. Stopping at the first failure leaves the rest of
the run un-undone, which is worse than a partial restore.

**[R-OPS-024]** Replay MUST report one of three outcomes: every inverse
applied, some applied, or none. These map to the `ok`, `partial`, and `failed`
values `state.toml` records; see [R-CONFIG-044].

**[R-OPS-025]** The journal MUST be truncated after a successful run, and a
non-empty journal at startup MUST be reported as an interrupted previous run;
see [R-ENGINE-042].

**[R-OPS-026]** No `Op` MUST be journaled during a dry run, and no journal file
MUST be created. The `DryRunFs` makes the effects no-ops, and a journal of
things that did not happen would replay into damage.

## Failure paths

**[R-OPS-030]** A journal record that cannot be parsed MUST NOT abort the
replay of the records around it. It MUST be reported and skipped, because the
alternative is a corrupt line stranding every earlier operation.

**[R-OPS-031]** An inverse whose target no longer exists MUST succeed rather
than fail. A user who deleted the file meowctl created has already achieved
what the inverse wanted.

**[R-OPS-032]** Journaling MUST fail the operation when the append fails. An
effect applied with no record of how to undo it is the state this component
exists to prevent.

## Parity with v0.1.0

The journal format, the seven implemented operation kinds, their inverse
payloads, and the three rollback outcomes all come from
`internal/rollback/rollback.go` and are reproduced exactly, including the JSON
field names, so a journal left by one binary is readable by the other.

Three things change. [R-OPS-017] journals `defaults_write` and `plist_set`,
which `v0.1.0` applies unjournaled; a `v0.1.0` binary replaying a `v0.2.0`
journal containing one of them will report "inverse not implemented" and carry
on, which is the same outcome it reaches today by having no record at all.

The second is structural. `v0.1.0` performs an effect in `internal/ctx` and
journals it through a separate `Append*` call, so the two can disagree; here
they are one value.

The third is what proves it. `v0.1.0` has no test that apply-then-undo restores
the tree, which is why [R-OPS-004] is written as an obligation on every variant
rather than on the ones somebody remembered to check.
