# meowctl

Declarative dotfiles and dev environment manager powered by Starlark.

## Status

Early development. Not ready for use.

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

- Go 1.24+
- macOS, Linux, or WSL

## Building

```sh
task build
```

Or without Task:

```sh
CGO_ENABLED=0 go build -o meowctl ./cmd/meowctl
```

## License

Copyright meowctl contributors. Licensed under the [Apache License 2.0](LICENSE).
