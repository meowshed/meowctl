package cli

import (
	"fmt"

	"github.com/spf13/cobra"
)

// shellSnippets maps shell name → integration snippet.
var shellSnippets = map[string]string{
	"bash": `# meowctl shell integration for bash
# Add this to your ~/.bashrc or ~/.bash_profile:
#   eval "$(meowctl shell bash)"

meowctl_shell_init() {
    export MEOWCTL_SHELL=bash
}
meowctl_shell_init
`,
	"zsh": `# meowctl shell integration for zsh
# Add this to your ~/.zshrc:
#   eval "$(meowctl shell zsh)"

meowctl_shell_init() {
    export MEOWCTL_SHELL=zsh
}
meowctl_shell_init
`,
	"fish": `# meowctl shell integration for fish
# Add this to your ~/.config/fish/config.fish:
#   meowctl shell fish | source

set -gx MEOWCTL_SHELL fish
`,
}

func newShellCmd() *cobra.Command {
	return &cobra.Command{
		Use:       "shell <shell>",
		Short:     "Emit shell integration code for the given shell",
		Args:      cobra.ExactArgs(1),
		ValidArgs: []string{"bash", "zsh", "fish"},
		RunE: func(_ *cobra.Command, args []string) error {
			shell := args[0]
			snippet, ok := shellSnippets[shell]
			if !ok {
				return exitErrorf(ExitUsage, "unsupported shell %q (supported: bash, zsh, fish)", shell)
			}
			fmt.Print(snippet)
			return nil
		},
	}
}
