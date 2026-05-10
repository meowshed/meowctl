// Package main is the entry point for the meowctl binary.
package main

import (
	"fmt"
	"os"

	"github.com/meowshed/meowctl/internal/cli"
)

func main() {
	if err := cli.RootCmd.Execute(); err != nil {
		fmt.Fprintf(os.Stderr, "error: %v\n", err)
		os.Exit(1)
	}
}
