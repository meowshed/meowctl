# Corpus configuration: exercises the paths the rewrite has to reproduce
# without needing a network or touching the machine.
#
# It covers a dependency graph with an explicit ordering hint, a platform
# select, a package manager declared as a component, file operations that
# journal an inverse, and a shell hook.

component("base")
component("tool")
component("fakepm")
component("packaged")
component("shellbits")
