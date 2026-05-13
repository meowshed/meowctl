// Package main is the entry point for the meowctl binary.
package main

import (
	"errors"
	"fmt"
	"os"

	"github.com/meowshed/meowctl/internal/cli"
)

func main() {
	if err := cli.RootCmd.Execute(); err != nil {
		fmt.Fprintf(os.Stderr, "error: %v\n", err)
		var exitErr *cli.ExitError
		if errors.As(err, &exitErr) {
			os.Exit(exitErr.Code)
		}
		os.Exit(1)
	}
}
