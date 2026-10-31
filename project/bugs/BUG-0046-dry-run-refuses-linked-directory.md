---
id: BUG-0046
artifact: bug
status: approved
severity: minor
violates: REQ-1433
enters: implement
found: 2026-10-27
revised: 2026-10-31
---

# A dry run refuses to write into a directory reached through a symlink

`DryRunFs::directory_would_exist` in `crates/meowctl-fs/src/dry_run.rs` counts
only `Entry::Directory`, and `RealFs::entry` reports a symlink as
`Entry::Symlink`, so `apply --dry-run` fails at `state.toml` whenever the
configuration directory is a link to a dotfiles checkout.

## Reproduction

meowctl 0.2.0 at commit `fb70085`, aarch64-apple-darwin, with
`~/.config/meowctl` a symlink to `~/workspace/dotfiles`. Run:

```bash
meowctl apply --dry-run --format plain
```

## What the system does

```text
meowctl: /Users/u/.config/meowctl/state.toml: /Users/u/.config/meowctl/state.toml cannot be written: its directory /Users/u/.config/meowctl does not exist
```

It exits 3. The same command with `--config ~/workspace/dotfiles` prints the
plan, and a real `apply` writes `state.toml` through the link.

## What it should do, and why

[REQ-1433] says the dry run fails where the real run would fail for a reason it
can see. The real run writes through the link, so the dry run must too, and
still fail for a link to nothing.

## Triage

[REQ-1433] covers it, so it enters at implement. It is minor because
`--config` with the link's target works round it and no real run is affected.

## Closed by

Open.

## Tasks

- [ ] T-001 TSK-0046 follow a symlink when asking whether a directory exists,
  in `crates/meowctl-fs/src/dry_run.rs`
