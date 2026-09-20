//! The shell integration snippets.
//!
//! Reproduced from `shellSnippets` in `internal/cli/shell.go` byte for byte.
//! A user's `~/.zshrc` evaluates this, so it is a machine interface and not
//! prose: a reworded comment is a diff in everybody's dotfiles; see
//! [R-CLI-021].

use crate::cli::Shell;

/// What to emit for a shell.
#[must_use]
pub const fn snippet(shell: Shell) -> &'static str {
    match shell {
        Shell::Bash => BASH,
        Shell::Zsh => ZSH,
        Shell::Fish => FISH,
        Shell::Posix => POSIX,
    }
}

const BASH: &str = r#"# meowctl shell integration for bash
# Add this to your ~/.bashrc:
#   eval "$(meowctl shell bash)"

meowctl_shell_init() {
    export MEOWCTL_SHELL=bash
    if [ -z "$_MEOWCTL_SHELL_DONE" ]; then
        export _MEOWCTL_SHELL_DONE=1
        eval "$(meowctl hook shell)"
    fi
}
meowctl_shell_init
"#;

const ZSH: &str = r#"# meowctl shell integration for zsh
# Add this to your ~/.zshrc:
#   eval "$(meowctl shell zsh)"

meowctl_shell_init() {
    export MEOWCTL_SHELL=zsh
    [[ -n "$_MEOWCTL_SHELL_DONE" ]] && return
    export _MEOWCTL_SHELL_DONE=1
    eval "$(meowctl hook shell)"
}
meowctl_shell_init
"#;

const FISH: &str = r#"# meowctl shell integration for fish
# Add this to your ~/.config/fish/config.fish:
#   meowctl shell fish | source

set -gx MEOWCTL_SHELL fish
if not set -q _MEOWCTL_SHELL_DONE
    set -gx _MEOWCTL_SHELL_DONE 1
    meowctl hook shell | source
end
"#;

const POSIX: &str = r#"# meowctl shell integration for POSIX sh
# Add this to your ~/.profile or ~/.shrc:
#   eval "$(meowctl shell posix)"

meowctl_shell_init() {
    export MEOWCTL_SHELL=posix
    if [ -z "$_MEOWCTL_SHELL_DONE" ]; then
        export _MEOWCTL_SHELL_DONE=1
        eval "$(meowctl hook shell)"
    fi
}
meowctl_shell_init
"#;
