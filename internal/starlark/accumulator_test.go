package starlark

import "testing"

func TestLogicalName(t *testing.T) {
	tests := []struct {
		name string
		want string
	}{
		{"@stdlib//components/node", "node"},
		{"github://owner/repo//components/neovim", "neovim"},
		{"shell", "shell"},
		{"my-tool", "my-tool"},
		{"@stdlib//components/zsh/", "zsh"},
		{"components/foo/bar", "bar"},
	}
	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			d := ComponentDecl{Name: tc.name}
			got := d.LogicalName()
			if got != tc.want {
				t.Errorf("LogicalName(%q) = %q, want %q", tc.name, got, tc.want)
			}
		})
	}
}
