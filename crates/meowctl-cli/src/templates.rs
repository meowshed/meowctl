//! What `meowctl init` writes.
//!
//! Reproduced from the templates in `internal/cli/commands.go` byte for
//! byte. They are the first thing a user sees of the tool and they live in
//! their dotfiles repository afterwards, so a reworded comment is a diff in
//! a file they did not change.

/// `init.star`, the configuration's entry point.
pub const INIT_STAR: &str = r#"# init.star — dotfiles configuration
# See https://github.com/meowshed/meowctl for documentation.

# Declare the @stdlib source so components can reference it.
# source("@stdlib", "github://meowshed/meow-stdlib")

# Declare components. Each component corresponds to a <name>.star file
# in the components/ directory, or a stdlib component via @stdlib//.
#
# component("shell")
# component("git")
# component("@stdlib//components/zsh")
"#;

/// `local.star`, for what one machine adds.
pub const LOCAL_STAR: &str = r#"# local.star — machine-specific component additions (gitignored)
# Add components that should only apply to this machine.
#
# component("work-vpn")
"#;

/// `deps.local.mod`, for what one machine overrides.
pub const LOCAL_MOD: &str = r#"# deps.local.mod — machine-specific module declarations (gitignored)
# Declare additional or override deps for this machine only.
# Entries here shadow the same-named entries in deps.mod at runtime.
"#;

/// What a new configuration calls itself in `deps.mod`.
pub const MODULE_NAME: &str = "my-dotfiles";

/// And at what version.
pub const MODULE_VERSION: &str = "0.1.0";
