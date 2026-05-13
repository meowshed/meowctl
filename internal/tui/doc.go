// Package tui provides TTY-aware progress output for meowctl commands.
//
// Use [New] to obtain a [Writer] appropriate for the output destination.
// On a TTY it returns a Bubble Tea-backed writer with per-component spinner
// rows; on a non-TTY (pipe, CI) it returns a plain-text writer.
package tui
