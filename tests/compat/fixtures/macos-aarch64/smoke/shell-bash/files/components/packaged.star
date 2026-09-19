"""Declares a package, dispatched to the handler in fakepm."""

after = ["fakepm", "base"]

def install(ctx):
    pkg("widget", version = "1.2.3", manager = "fakepm")
