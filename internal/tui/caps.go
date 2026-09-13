package tui

import (
	"io"
	"os"
	"strings"

	"github.com/charmbracelet/colorprofile"
	"golang.org/x/term"
)

// Mode selects how output is rendered. Auto resolves to Live or Plain by
// inspecting the terminal; the other values force a specific renderer.
type Mode int

const (
	// ModeAuto picks Live or Plain from the detected terminal capabilities.
	ModeAuto Mode = iota
	// ModeLive forces the in-place renderer even if detection says otherwise.
	ModeLive
	// ModePlain forces one line per event and no cursor movement.
	ModePlain
)

// ParseMode maps the MEOWCTL_OUTPUT value (or a --output flag) to a Mode.
// Unrecognised values resolve to ModeAuto so a typo degrades to detection
// rather than to an error in the middle of an apply.
func ParseMode(s string) Mode {
	switch strings.ToLower(strings.TrimSpace(s)) {
	case "live":
		return ModeLive
	case "plain":
		return ModePlain
	default:
		return ModeAuto
	}
}

// Caps describes what the output destination can render. It is resolved once
// per command; a resize is picked up by re-reading Width at frame time.
type Caps struct {
	// TTY reports whether out is an interactive terminal.
	TTY bool
	// Live reports whether the in-place renderer may move the cursor.
	// False for pipes, dumb terminals, CI, and forced plain mode.
	Live bool
	// Unicode reports whether the locale advertises UTF-8. When false the
	// theme falls back to an ASCII symbol set.
	Unicode bool
	// Profile is the colour depth; SGR sequences are downsampled to it.
	Profile colorprofile.Profile
	// Width is the terminal width in columns, or 0 when unknown.
	Width int
	// fd is the file descriptor of out when it is a terminal, else -1.
	fd int
}

// DetectCaps resolves the capabilities of out under the given environment.
// env is in os.Environ() form so tests can supply their own.
//
// The live renderer is enabled only for a real terminal that is not dumb and
// not CI. That is deliberately conservative: cursor movement written into a
// log file or a CI transcript is unreadable, and a wrong guess there is much
// more annoying than a missing spinner.
func DetectCaps(out io.Writer, env []string, mode Mode) Caps {
	c := Caps{fd: -1, Profile: colorprofile.Detect(out, env)}

	if f, ok := out.(*os.File); ok {
		fd := int(f.Fd())
		if term.IsTerminal(fd) {
			c.TTY = true
			c.fd = fd
			if w, _, err := term.GetSize(fd); err == nil {
				c.Width = w
			}
		}
	}

	lookup := envLookup(env)
	termVar := lookup("TERM")
	dumb := termVar == "" || termVar == "dumb"
	// Most CI providers capture stdout into a log; treat them as non-live even
	// when they hand us a pty.
	ci := lookup("CI") != ""

	switch mode {
	case ModeLive:
		c.Live = true
	case ModePlain:
		c.Live = false
	default:
		c.Live = c.TTY && !dumb && !ci
	}

	c.Unicode = supportsUnicode(lookup)
	return c
}

// Size re-reads the terminal width, returning the last known value when the
// query fails. Called per frame so a resize is picked up without SIGWINCH.
func (c Caps) Size() int {
	if c.fd < 0 {
		return c.Width
	}
	if w, _, err := term.GetSize(c.fd); err == nil && w > 0 {
		return w
	}
	return c.Width
}

// Color reports whether SGR sequences survive downsampling.
func (c Caps) Color() bool {
	return c.Profile > colorprofile.Ascii
}

// envLookup returns a getenv-style function over an os.Environ() slice.
func envLookup(env []string) func(string) string {
	return func(key string) string {
		prefix := key + "="
		for i := len(env) - 1; i >= 0; i-- {
			if strings.HasPrefix(env[i], prefix) {
				return env[i][len(prefix):]
			}
		}
		return ""
	}
}

// supportsUnicode reports whether the locale advertises UTF-8. Terminals lie
// about a lot of things, but a non-UTF-8 locale reliably means box-drawing and
// braille characters will render as mojibake, so the ASCII set is used instead.
func supportsUnicode(lookup func(string) string) bool {
	for _, key := range []string{"LC_ALL", "LC_CTYPE", "LANG"} {
		v := strings.ToUpper(lookup(key))
		if v == "" {
			continue
		}
		return strings.Contains(v, "UTF-8") || strings.Contains(v, "UTF8")
	}
	// No locale set at all: assume a modern terminal on macOS/Linux, but stay
	// ASCII on Windows consoles, which historically default to a code page.
	return os.PathSeparator == '/'
}

// termSize returns the size of fd. Split out so the live renderer can query
// height per frame without importing x/term directly.
func termSize(fd int) (width, height int, err error) {
	return term.GetSize(fd)
}
