package ctx

// ShellCtxAllowList is the set of ctx attributes available to shell.star
// scripts. shell.star may not use mutating file operations — it must have no
// persistent side effects beyond the shell code it emits via ctx.emit().
var ShellCtxAllowList = []string{
	"emit",
	"file_exists",
	"list_dir",
	"platform",
	"read_file",
	"run",
	"shell",
	"state_dir",
}
