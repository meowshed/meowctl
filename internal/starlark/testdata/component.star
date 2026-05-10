"""Fixture component file for integration tests.

Exercises all six predeclared builtins (module, component, pkg, dep, select, platform)
and defines two lifecycle hooks (install, setup) that access ctx.

The provides kwarg on component() exercises the extra-kwargs path in builtinComponent.
The dep() URL is recorded without resolution — no loader is configured in the test evaluator.
"""

module(name = "fixture", version = "1.0.0")

component(name = "git", provides = ["vcs"])

pkg(manager = "apt", name = "git")
pkg(manager = "brew", name = "git")

dep(url = "self//lib.star")

_p = platform()

_path = select(
    cases = {
        "//platform:linux": "linux-path",
        "//platform:macos": "macos-path",
        "//conditions:default": "fallback-path",
    },
)

def install(ctx):
    ctx.log(msg = "install called")

def setup(ctx):
    ctx.log(msg = "setup called")
