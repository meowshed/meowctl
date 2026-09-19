# meowctl

[![CI](https://github.com/meowshed/meowctl/actions/workflows/ci.yml/badge.svg)](https://github.com/meowshed/meowctl/actions/workflows/ci.yml)

Dotfiles and dev environment manager powered by Starlark.

## Status

**0.1.0** — the Go implementation is feature-complete for the dotfiles
workflow: Starlark components, module resolution with MVS and lockfiles,
lifecycle phases with rollback. See [CHANGELOG.md](CHANGELOG.md).

0.2.0 is a full rewrite in Rust with a reworked architecture, in progress on
the `rust-rewrite` branch.

## Overview

meowctl is a single binary that manages dotfiles and developer environments using
[Starlark](https://github.com/bazelbuild/starlark) configuration files. It is the
Go-based successor to the `.meow` shell scripting system.

## Repositories

| Repo | Purpose |
|------|---------|
| [meowshed/meowctl](https://github.com/meowshed/meowctl) | This repo — Go binary |
| [meowshed/meowctl-stdlib](https://github.com/meowshed/meowctl-stdlib) | Standard library components (package managers, utilities) |
| [meowshed/meowctl-registry](https://github.com/meowshed/meowctl-registry) | Module registry index |

## Requirements

- Go 1.25+
- [mise](https://mise.jdx.dev) (recommended) or any Go toolchain

## Development

Install tools and run common tasks via [mise](https://mise.jdx.dev):

```sh
mise install          # install Go, golangci-lint, opencode
mise run build        # build bin/meowctl
mise run test         # run tests
mise run lint         # run golangci-lint
mise run install      # install to GOPATH/bin
mise run clean        # remove build artifacts
```

Without mise:

```sh
CGO_ENABLED=0 go build -o bin/meowctl ./cmd/meowctl
```

## License

Copyright meowctl contributors. Licensed under the [Apache License 2.0](LICENSE).
