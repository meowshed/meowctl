// Package tui renders meowctl's terminal output.
//
// Two entry points, one design system:
//
//   - [Writer] renders lifecycle progress. [New] returns [LiveWriter] on a
//     capable terminal and [PlainWriter] everywhere else.
//   - [Printer] renders command output — plans, results, tables, diagnostics.
//
// Both draw their glyphs, colours and indentation from [Theme], so a status
// means the same thing in every command, and both degrade through the same
// [Caps] detection: a pipe or CI log loses motion, a monochrome terminal or
// NO_COLOR loses colour, a non-UTF-8 locale falls back to ASCII glyphs, and
// nothing loses information.
//
// The live renderer never puts the terminal into raw mode. Lifecycle hooks
// shell out to commands that may need the real terminal, so instead of owning
// it the renderer erases its region and stands down for the duration — see
// [LiveWriter] and Capabilities.SuspendOutput in internal/ctx.
package tui
