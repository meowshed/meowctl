"""Depends on base, and branches on the platform."""

after = ["base"]

GREETING = select({
    "//platform:macos": "hello from macos",
    "//platform:linux": "hello from linux",
    "//conditions:default": "hello",
})

def install(ctx):
    ctx.write_file(ctx.home + "/.config/corpus/tool.conf", GREETING + "\n")
    ctx.append_file(
        ctx.home + "/.config/corpus/base.conf",
        "tool = true\n",
        marker = "corpus-tool",
    )

def verify(ctx):
    ctx.read_file(ctx.home + "/.config/corpus/tool.conf")
