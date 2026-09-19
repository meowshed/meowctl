"""Contributes to the shell environment, which runs on every shell spawn."""

after = ["base"]

def shell(ctx):
    ctx.add_path(ctx.home + "/.config/corpus/bin")
    ctx.emit("set -gx CORPUS_SHELL " + ctx.shell)
