package main

import (
	"os"

	"github.com/meowshed/meowctl/internal/cli"
)

func main() {
	if err := cli.RootCmd.Execute(); err != nil {
		os.Exit(1)
	}
}
