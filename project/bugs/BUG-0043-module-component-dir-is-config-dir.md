---
id: BUG-0043
artifact: bug
status: approved
severity: critical
violates: REQ-2803
enters: implement
found: 2026-10-25
revised: 2026-10-25
---

# A module component's `component_dir` is the configuration's own `components/` directory

`ConfigSources::source` at `crates/meowctl-cli/src/run/commands.rs:222` gives a
module component `<config>/components/<logical name>` as its `component_dir`,
so `link_file` and `render_file` in a dotmeow or meowctl-stdlib component read
from a directory that isn't the component's and usually doesn't exist.

## Reproduction

meowctl at commit `8df9f84` on `fix/link-file-source`, which reads
`link_file`'s `src` from `component_dir` (BUG-0042), aarch64-apple-darwin,
with dotmeow 0.3.32 and a configuration at `~/.config/meowctl` that has no
`components/git-config/`. Run:

```bash
meowctl apply @dotmeow//components/git-config
```

On `main`, `link_file` refuses the relative `src` first, so the defect shows
only as the wrong `ctx.component_dir`.

## What the system does

The apply succeeds and replaces the working links:

```text
~/.config/git/config -> ~/.config/meowctl/components/git-config/config
```

That target doesn't exist, so git loses its configuration. The component's
files are in the module cache, at
`~/.cache/meowctl/modules/dotmeow/0.3.32/components/git-config/`.

## What it should do, and why

[REQ-2803] says `component_dir` is the component's own source directory, and
for a module component that is the directory the loader read its `init.star`
from: the cache directory for a registry or GitHub module, or the checkout for
a replaced one.

## Triage

[REQ-2803] covers it, so it enters at implement. It is critical because, once
BUG-0042 is fixed, an apply silently replaces a user's working links with
broken ones, and before that fix the same wrong directory makes every module
`render_file` read from the wrong place.

## Closed by

`a_module_component_lives_where_its_file_was_read` in
`crates/meowctl-cli/src/run/commands.rs` holds the directory as a regression
check, with the two loader tests in `crates/meowctl-module/tests/fetching.rs`.

## Tasks

- [x] T-001 TSK-0041 take a module component's directory from where the
  loader read it, in `crates/meowctl-cli/src/run/commands.rs` and
  `crates/meowctl-module/src/loader.rs`
  (the three directory tests pass, and every stage exits 0)
