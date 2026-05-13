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

// ComponentStartMsg is sent to the Bubble Tea model when a component begins.
type ComponentStartMsg struct{ Name string }

// ComponentDoneMsg is sent to the Bubble Tea model when a component finishes.
type ComponentDoneMsg struct {
	Name string
	Err  error
}

// logMsg carries a free-form log line to append below the component rows.
type logMsg struct{ line string }

// btModel is the internal Bubble Tea model for BubbleTeaWriter.
type btModel struct {
	rows []*componentRow
	logs []string
}

// Init starts spinner ticks for any rows already in running state (none at
// construction time, but included for correctness).
func (m btModel) Init() tea.Cmd {
	var cmds []tea.Cmd
	for _, row := range m.rows {
		if row.state == rowRunning {
			row := row
			cmds = append(cmds, func() tea.Msg { return row.spinner.Tick() })
		}
	}
	return tea.Batch(cmds...)
}

// Update handles incoming messages and returns the updated model and commands.
func (m btModel) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case ComponentStartMsg:
		for _, row := range m.rows {
			if row.name == msg.Name {
				row.state = rowRunning
				row.spinner = spinner.New(spinner.WithSpinner(spinner.MiniDot),
					spinner.WithStyle(lipgloss.NewStyle().Foreground(lipgloss.Color("33"))))
				return m, func() tea.Msg { return row.spinner.Tick() }
			}
		}
		// Component not yet registered — add it.
		s := spinner.New(spinner.WithSpinner(spinner.MiniDot),
			spinner.WithStyle(lipgloss.NewStyle().Foreground(lipgloss.Color("33"))))
		row := &componentRow{name: msg.Name, state: rowRunning, spinner: s}
		m.rows = append(m.rows, row)
		return m, func() tea.Msg { return s.Tick() }

	case ComponentDoneMsg:
		for _, row := range m.rows {
			if row.name == msg.Name {
				if msg.Err != nil {
					row.state = rowFailed
					row.err = msg.Err
				} else {
					row.state = rowOK
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
		for _, row := range m.rows {
			if row.state == rowRunning {
				newSpinner, cmd := row.spinner.Update(msg)
				row.spinner = newSpinner
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
			sb.WriteString(lipgloss.NewStyle().Faint(true).Render("  … "+row.name) + "\n")
		case rowRunning:
			sb.WriteString("  " + row.spinner.View() + " " + row.name + "\n")
		case rowOK:
			ok := lipgloss.NewStyle().Foreground(lipgloss.Color("10")).Render("✓")
			sb.WriteString("  " + ok + " " + row.name + "\n")
		case rowFailed:
			fail := lipgloss.NewStyle().Foreground(lipgloss.Color("9")).Render("✗")
			sb.WriteString("  " + fail + " " + row.name + ": " + row.err.Error() + "\n")
		}
	}
	for _, line := range m.logs {
		sb.WriteString(line)
	}
	return tea.NewView(sb.String())
}

// BubbleTeaWriter runs a Bubble Tea v2 program inline (no alt-screen) and
// renders per-component spinner rows with Lip Gloss styling.
// Use [New] to obtain a BubbleTeaWriter; it is returned when stdout is a TTY.
type BubbleTeaWriter struct {
	program *tea.Program
	done    chan struct{}
	once    sync.Once
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
			// Non-fatal: output is best-effort in TUI mode.
			_, _ = fmt.Fprintf(out, "meowctl: tui: %v\n", err)
		}
	}()
	return w
}

// ComponentStart marks a component as in-progress.
func (w *BubbleTeaWriter) ComponentStart(name string) {
	w.program.Send(ComponentStartMsg{Name: name})
}

// ComponentDone marks a component as completed.
func (w *BubbleTeaWriter) ComponentDone(name string, err error) {
	w.program.Send(ComponentDoneMsg{Name: name, Err: err})
}

// Log emits a formatted log line.
func (w *BubbleTeaWriter) Log(format string, args ...any) {
	w.program.Send(logMsg{line: fmt.Sprintf(format, args...)})
}

// Close shuts down the Bubble Tea program and waits for it to exit.
func (w *BubbleTeaWriter) Close() error {
	w.once.Do(func() {
		w.program.Quit()
		<-w.done
	})
	return nil
}
