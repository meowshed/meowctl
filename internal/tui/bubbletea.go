package tui

import (
	"fmt"
	"io"
	"strings"
	"sync"

	"charm.land/bubbles/v2/spinner"
	tea "charm.land/bubbletea/v2"
	"charm.land/lipgloss/v2"
)

// Package-level styles to avoid allocating on every View() call.
var (
	stylePending = lipgloss.NewStyle().Faint(true)
	styleOK      = lipgloss.NewStyle().Foreground(lipgloss.Color("10"))
	styleFail    = lipgloss.NewStyle().Foreground(lipgloss.Color("9"))
	styleSpinner = lipgloss.NewStyle().Foreground(lipgloss.Color("33"))
)

// rowState represents the display state of a component row.
type rowState int

const (
	rowPending rowState = iota
	rowRunning
	rowOK
	rowFailed
)

// componentRow holds per-component render state.
type componentRow struct {
	name    string
	state   rowState
	spinner spinner.Model
	err     error
}

// componentStartMsg is sent to the Bubble Tea model when a component begins.
type componentStartMsg struct{ name string }

// componentDoneMsg is sent to the Bubble Tea model when a component finishes.
type componentDoneMsg struct {
	name string
	err  error
}

// logMsg carries a free-form log line to append below the component rows.
type logMsg struct{ line string }

// btModel is the internal Bubble Tea model for BubbleTeaWriter.
type btModel struct {
	rows []*componentRow
	logs []string
}

// Init starts spinner ticks for rows in running state on program startup.
// Guards against future pre-populated rows.
func (m btModel) Init() tea.Cmd {
	var cmds []tea.Cmd
	for _, row := range m.rows {
		if row.state == rowRunning {
			cmds = append(cmds, row.spinner.Tick)
		}
	}
	return tea.Batch(cmds...)
}

// Update handles incoming messages and returns the updated model and commands.
// btModel uses pointer-typed rows so index-based mutation is safe across the
// value-receiver copy: all mutations go through the pointer, never the slice
// header itself.
func (m btModel) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case componentStartMsg:
		for i, row := range m.rows {
			if row.name == msg.name {
				s := spinner.New(
					spinner.WithSpinner(spinner.MiniDot),
					spinner.WithStyle(styleSpinner),
				)
				m.rows[i].state = rowRunning
				m.rows[i].spinner = s
				return m, s.Tick
			}
		}
		// Component not yet registered — add it.
		s := spinner.New(
			spinner.WithSpinner(spinner.MiniDot),
			spinner.WithStyle(styleSpinner),
		)
		m.rows = append(m.rows, &componentRow{name: msg.name, state: rowRunning, spinner: s})
		return m, s.Tick

	case componentDoneMsg:
		for i, row := range m.rows {
			if row.name == msg.name {
				if msg.err != nil {
					m.rows[i].state = rowFailed
					m.rows[i].err = msg.err
				} else {
					m.rows[i].state = rowOK
				}
				return m, nil
			}
		}
		return m, nil

	case logMsg:
		m.logs = append(m.logs, msg.line)
		return m, nil

	case spinner.TickMsg:
		var cmds []tea.Cmd
		for i, row := range m.rows {
			if row.state == rowRunning {
				newSpinner, cmd := m.rows[i].spinner.Update(msg)
				m.rows[i].spinner = newSpinner
				if cmd != nil {
					cmds = append(cmds, cmd)
				}
			}
		}
		return m, tea.Batch(cmds...)

	default:
		return m, nil
	}
}

// View renders the current component rows and any log lines.
func (m btModel) View() tea.View {
	var sb strings.Builder
	for _, row := range m.rows {
		switch row.state {
		case rowPending:
			sb.WriteString(stylePending.Render("  … "+row.name) + "\n")
		case rowRunning:
			sb.WriteString("  " + row.spinner.View() + " " + row.name + "\n")
		case rowOK:
			sb.WriteString("  " + styleOK.Render("✓") + " " + row.name + "\n")
		case rowFailed:
			sb.WriteString("  " + styleFail.Render("✗") + " " + row.name + ": " + row.err.Error() + "\n")
		}
	}
	for _, line := range m.logs {
		if !strings.HasSuffix(line, "\n") {
			sb.WriteString(line + "\n")
		} else {
			sb.WriteString(line)
		}
	}
	return tea.NewView(sb.String())
}

// BubbleTeaWriter runs a Bubble Tea v2 program inline (no alt-screen) and
// renders per-component spinner rows with Lip Gloss styling.
// Use [New] to obtain a BubbleTeaWriter; it is returned when stdout is a TTY.
type BubbleTeaWriter struct {
	program *tea.Program
	// done is closed by the program goroutine when p.Run() returns.
	done chan struct{}
	// once ensures Close() is idempotent.
	once   sync.Once
	runErr error
}

func newBubbleTeaWriter(out io.Writer) *BubbleTeaWriter {
	m := btModel{}
	p := tea.NewProgram(m, tea.WithOutput(out), tea.WithInput(nil))
	w := &BubbleTeaWriter{
		program: p,
		done:    make(chan struct{}),
	}
	go func() {
		defer close(w.done)
		if _, err := p.Run(); err != nil {
			w.runErr = err
		}
	}()
	return w
}

// ComponentStart marks a component as in-progress.
func (w *BubbleTeaWriter) ComponentStart(name string) {
	w.program.Send(componentStartMsg{name: name})
}

// ComponentDone marks a component as completed.
func (w *BubbleTeaWriter) ComponentDone(name string, err error) {
	w.program.Send(componentDoneMsg{name: name, err: err})
}

// Log emits a formatted log line. The format string must include any desired
// trailing newline (consistent with the [Writer] interface contract).
func (w *BubbleTeaWriter) Log(format string, args ...any) {
	w.program.Send(logMsg{line: fmt.Sprintf(format, args...)})
}

// Close shuts down the Bubble Tea program, waits for it to exit, and returns
// any error from the program run. Safe to call even if no messages were sent.
// Concurrent calls are safe; only the first call performs the shutdown.
func (w *BubbleTeaWriter) Close() error {
	var err error
	w.once.Do(func() {
		w.program.Quit()
		<-w.done
		err = w.runErr
	})
	return err
}
