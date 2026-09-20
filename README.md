# meowctl

[![CI](https://github.com/meowshed/meowctl/actions/workflows/ci.yml/badge.svg)](https://github.com/meowshed/meowctl/actions/workflows/ci.yml)

Dotfiles and dev environment manager powered by Starlark.

## What it does

You describe a machine as a set of components, each a Starlark file that says
what to install and what to link. meowctl resolves the modules those components
come from, orders them by their declared dependencies, and runs them through
the lifecycle phases. A failure rolls back what the run had done.

```python
# components/bat-config.star
after = ["@stdlib//bundles/modern-shell"]

pkg("bat")

def install(ctx):
    d = ctx.home + "/.config/bat"
    ctx.mkdir(d)
    ctx.link_file("config", d + "/config")

def uninstall(ctx):
    ctx.remove_symlink(ctx.home + "/.config/bat/config")

def shell(ctx):
    if ctx.shell == "fish":
        ctx.emit("set -gx BAT_CONFIG_PATH ~/.config/bat/config")
    else:
        ctx.emit("export BAT_CONFIG_PATH=~/.config/bat/config")
```

```sh
meowctl init            # scaffold a configuration
meowctl add bat-config  # declare a component and install it
meowctl apply           # bring the machine to what the configuration says
meowctl apply -n        # say what that would do, and touch nothing
meowctl status          # what the last run did
```

`meowctl --help` lists the rest.

## Status

**0.2.0** — Rust, and the version to use. It replaced the Go implementation
released as `v0.1.0`, which is gone from this tree and still available at the
`v0.1.0` tag.

The rewrite keeps the Starlark API, the command surface, and every config and
lock format byte for byte. A configuration `v0.1.0` applied applies here. What
changed is the inside and the terminal output: see
[CHANGELOG.md](CHANGELOG.md) for the summary and
[`docs/design/0.2.0-rust-rewrite.md`](docs/design/0.2.0-rust-rewrite.md) for
why each change is a rewrite rather than a refactor.

## Repositories

| Repo | Purpose |
| --- | --- |
| [meowshed/meowctl](https://github.com/meowshed/meowctl) | This repo — the binary |
| [meowshed/meowctl-stdlib](https://github.com/meowshed/meowctl-stdlib) | Standard library components: package managers, tools, bundles |
| [meowshed/meowctl-registry](https://github.com/meowshed/meowctl-registry) | Module registry index |

## Building

[mise](https://mise.jdx.dev) installs the pinned toolchain and every tool the
tasks need:

```sh
mise install
mise run build        # cargo build --workspace
mise run test         # cargo nextest run --workspace
mise run all          # everything CI checks
```

`mise tasks` lists the rest. Without mise, a stable Rust toolchain and
`cargo build` are enough to get a binary; the gate needs `cargo-nextest`,
`cargo-deny`, `cargo-machete`, and `markdownlint-cli2`.

## Documentation

[`docs/README.md`](docs/README.md) indexes every document. The two starting
points are [`docs/design/`](docs/design) for the architecture and
[`docs/spec/`](docs/spec) for the normative behaviour, one file per crate.

## License

Copyright meowctl contributors. Licensed under the
[Apache License 2.0](LICENSE).
