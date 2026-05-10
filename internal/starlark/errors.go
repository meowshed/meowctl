// Package starlark provides a Starlark evaluator for meowctl configuration files.
package starlark

import (
	"fmt"

	gostarlark "go.starlark.net/starlark"
)

// ParseError is returned when a Starlark file contains syntax errors.
type ParseError struct {
	Err error
}

func (e *ParseError) Error() string {
	return fmt.Sprintf("parse error: %v", e.Err)
}

func (e *ParseError) Unwrap() error {
	return e.Err
}

// EvalError is returned when a runtime error occurs during file body evaluation.
type EvalError struct {
	Err      *gostarlark.EvalError
	Filename string
}

func (e *EvalError) Error() string {
	return fmt.Sprintf("eval error in %s: %v", e.Filename, e.Err)
}

func (e *EvalError) Unwrap() error {
	return e.Err
}

// HookError is returned when a runtime error occurs during a lifecycle hook invocation.
type HookError struct {
	Err      *gostarlark.EvalError
	HookName string
	Filename string
}

func (e *HookError) Error() string {
	return fmt.Sprintf("hook error in %s[%s]: %v", e.Filename, e.HookName, e.Err)
}

func (e *HookError) Unwrap() error {
	return e.Err
}
