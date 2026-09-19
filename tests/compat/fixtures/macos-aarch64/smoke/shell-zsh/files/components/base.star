"""The root of the graph: everything else orders after it."""

def install(ctx):
    ctx.mkdir(ctx.home + "/.config/corpus")
    ctx.write_file(ctx.home + "/.config/corpus/base.conf", "base = true\n")

def verify(ctx):
    if not ctx.file_exists(ctx.home + "/.config/corpus/base.conf"):
        fail("base.conf is missing")
