"""A package manager handler, which is what makes managers configuration."""

pm_name = "fakepm"

def interrogate(ctx):
    return {"installed": []}

def install_pkg(ctx, name, version, **kwargs):
    ctx.write_file(ctx.home + "/.config/corpus/pkg-" + name, version + "\n")

def uninstall_pkg(ctx, name, version, **kwargs):
    ctx.delete_file(ctx.home + "/.config/corpus/pkg-" + name)
