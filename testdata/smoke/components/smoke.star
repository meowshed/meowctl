# Smoke fixture: minimal dotfiles component for meowctl install/uninstall dry-run tests.

def install(ctx):
    ctx.log("smoke: install phase")
    ctx.mkdir("/tmp/meowctl-smoke-test")
    ctx.write_file("/tmp/meowctl-smoke-test/hello.txt", "hello from smoke\n")
    ctx.symlink("/tmp/meowctl-smoke-test/hello.txt", "/tmp/meowctl-smoke-test/hello-link.txt")
    ctx.append_file("/tmp/meowctl-smoke-test/rc.sh", "export SMOKE=1")

def uninstall(ctx):
    ctx.log("smoke: uninstall phase")
    ctx.remove_symlink("/tmp/meowctl-smoke-test/hello-link.txt")
    ctx.delete_file("/tmp/meowctl-smoke-test/hello.txt")

def verify(ctx):
    ctx.log("smoke: verify phase")
    exists = ctx.file_exists("/tmp/meowctl-smoke-test/hello.txt")
    if not exists:
        fail("hello.txt missing")
