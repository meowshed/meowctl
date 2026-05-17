package cli

import (
	"fmt"

	"github.com/spf13/cobra"
)

// shellSnippets maps shell name → integration snippet.
var shellSnippets = map[string]string{
	"bash": `# meowctl shell integration for bash
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
`,
	"zsh": `# meowctl shell integration for zsh
# Add this to your ~/.zshrc:
#   eval "$(meowctl shell zsh)"

meowctl_shell_init() {
    export MEOWCTL_SHELL=zsh
    [[ -n "$_MEOWCTL_SHELL_DONE" ]] && return
    export _MEOWCTL_SHELL_DONE=1
    eval "$(meowctl hook shell)"
}
meowctl_shell_init
`,
	"fish": `# meowctl shell integration for fish
# Add this to your ~/.config/fish/config.fish:
#   meowctl shell fish | source

set -gx MEOWCTL_SHELL fish
if not set -q _MEOWCTL_SHELL_DONE
    set -gx _MEOWCTL_SHELL_DONE 1
    meowctl hook shell | source
end
`,
	"posix": `# meowctl shell integration for POSIX sh
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
`,
}

func newShellCmd() *cobra.Command {
	return &cobra.Command{
		Use:       "shell <shell>",
		Short:     "Emit shell integration code for the given shell",
		Args:      cobra.ExactArgs(1),
		ValidArgs: []string{"bash", "zsh", "fish", "posix"},
		RunE: func(cmd *cobra.Command, args []string) error {
			shell := args[0]
			snippet, ok := shellSnippets[shell]
			if !ok {
				return exitErrorf(ExitUsage, "unsupported shell %q (supported: bash, zsh, fish, posix)", shell)
			}
			_, err := fmt.Fprint(cmd.OutOrStdout(), snippet)
			return err
		},
	}
}
