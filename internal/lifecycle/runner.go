// Package lifecycle implements the meowctl phase runner and component execution
// order for lifecycle operations (install, uninstall, verify, etc.).
package lifecycle

import (
	"fmt"
	"strings"

	"github.com/meowshed/meowctl/internal/rollback"
	"github.com/meowshed/meowctl/internal/state"
	"github.com/meowshed/meowctl/internal/tui"
)

// Phase identifies one of the 9 meowctl lifecycle phases.
type Phase string

// Phase constants define the 9 lifecycle phases in execution order.
const (
	PhaseBootstrap Phase = "bootstrap"
	PhaseInit      Phase = "init"
	PhaseInstall   Phase = "install"
	PhaseUpdate    Phase = "update"
	PhaseSetup     Phase = "setup"
	PhaseLogin     Phase = "login"
	PhaseShell     Phase = "shell"
	PhaseUninstall Phase = "uninstall"
	PhaseVerify    Phase = "verify"
)

// Phase sets define the ordered phases for each top-level command.
var (
	PhaseSetInstall   = []Phase{PhaseBootstrap, PhaseInit, PhaseInstall, PhaseSetup, PhaseLogin, PhaseShell}
	PhaseSetUpdate    = []Phase{PhaseUpdate, PhaseSetup, PhaseLogin, PhaseShell}
	PhaseSetUninstall = []Phase{PhaseUninstall}
	PhaseSetVerify    = []Phase{PhaseVerify}
)

// HookCaller executes a named lifecycle hook for one component.
// componentID is the component identifier; the implementation resolves the
// corresponding Starlark file path internally.
// hookName is the hook function name (e.g. "install", "uninstall").
// Returns nil if the hook is absent (optional hooks).
type HookCaller interface {
	CallHook(componentID, hookName string) error
}

// ComponentFailure records a hook execution error for one component.
type ComponentFailure struct {
	Component string
	Err       error
}

// PhaseError is returned when one or more components fail during a phase.
type PhaseError struct {
	Phase    Phase
	Failures []ComponentFailure
}

// Error implements the error interface.
func (e *PhaseError) Error() string {
	if len(e.Failures) == 1 {
		return fmt.Sprintf("phase %s: %s: %v", e.Phase, e.Failures[0].Component, e.Failures[0].Err)
	}
	msgs := make([]string, len(e.Failures))
	for i, f := range e.Failures {
		msgs[i] = fmt.Sprintf("%s: %v", f.Component, f.Err)
	}
	return fmt.Sprintf("phase %s: %d component(s) failed: %s",
		e.Phase, len(e.Failures), strings.Join(msgs, "; "))
}

// Runner executes lifecycle phases for an ordered list of components.
type Runner struct {
	// Order is the dependency-first list of component IDs to execute.
	Order []ComponentID
	// HookCaller executes individual hooks.
	Caller HookCaller
	// Stack is the rollback write-ahead log. May be nil for dry-run or verify phases.
	Stack *rollback.Stack
	// Sentinel tracks run progress. May be nil to skip state tracking.
	Sentinel *state.Manager
	// NoRollback disables automatic rollback on phase failure.
	NoRollback bool
	// Force skips the already-completed check and re-runs every component.
	Force bool
	// Verbose enables detailed output: phase transitions, commands, output.
	Verbose bool
	// Writer receives progress and log events. When nil a PlainWriter to
	// os.Stdout is used.
	Writer tui.Writer
}

// writer returns r.Writer, falling back to a plain stdout writer when nil.
func (r *Runner) writer() tui.Writer {
	if r.Writer != nil {
		return r.Writer
	}
	return tui.NewPlainWriter(nil) // os.Stdout set inside NewPlainWriter when nil
}

// RunPhaseSet runs each phase in phases sequentially. On the first phase failure
// it triggers rollback (unless NoRollback is true) and returns the error.
func (r *Runner) RunPhaseSet(phaseSetName string, phases []Phase) error {
	if r.Sentinel != nil {
		if err := r.Sentinel.RecordRunStart(phaseSetName); err != nil {
			// Non-fatal: log and continue.
			r.writer().Log("meowctl: warning: could not write state: %v\n", err)
		}
	}

	var runErr error
	for _, phase := range phases {
		if err := r.RunPhase(phase); err != nil {
			runErr = err
			break
		}
	}

	if runErr != nil {
		var rb state.RolledBack
		if !r.NoRollback && r.Stack != nil {
			// Rollback enabled and a stack is open — attempt reverse.
			rb = r.executeRollback()
		} else {
			// No rollback attempted: either --no-rollback flag or dry-run (Stack==nil).
			rb = state.RolledBackNone
		}
		if r.Sentinel != nil {
			_ = r.Sentinel.RecordRunEnd(false, rb)
		}
		return runErr
	}

	// Success path.
	if r.Stack != nil {
		if err := r.Stack.Truncate(); err != nil {
			// A truncate failure leaves stale records in the journal; the next
			// run might attempt to replay them against an already-clean system.
			// Treat this as a hard error so the caller can alert the user.
			r.writer().Log("meowctl: error: could not truncate rollback journal: %v\n", err)
			return fmt.Errorf("rollback journal truncate: %w", err)
		}
	}
	if r.Sentinel != nil {
		_ = r.Sentinel.RecordRunEnd(true, state.RolledBackNone)
	}
	return nil
}

// RunPhase executes one phase for all components in order.
// Fail-fast: if any component fails, execution stops immediately and the error
// is returned. Rollback of the current phase-set is triggered by RunPhaseSet.
// Sentinel state is only recorded when the entire phase succeeds.
func (r *Runner) RunPhase(phase Phase) error {
	hookName := string(phase)
	if r.Verbose {
		r.writer().Log("==> phase: %s\n", hookName)
	}

	for _, id := range r.Order {
		// Skip components that have already been completed for this phase,
		// unless --force is set.
		if !r.Force && r.Sentinel != nil && r.Sentinel.IsCompleted(hookName, id) {
			r.writer().ComponentSkipped(id)
			continue
		}
		r.writer().ComponentStart(id)
		hookErr := r.Caller.CallHook(id, hookName)
		r.writer().ComponentDone(id, hookErr)
		if hookErr != nil {
			return &PhaseError{Phase: phase, Failures: []ComponentFailure{{Component: id, Err: hookErr}}}
		}
	}

	// Record completed components only after the phase is fully clean.
	if r.Sentinel != nil {
		for _, id := range r.Order {
			if err := r.Sentinel.RecordComponent(string(phase), id); err != nil {
				r.writer().Log("meowctl: warning: could not record component state: %v\n", err)
			}
		}
	}
	return nil
}

// executeRollback runs the rollback stack and returns the outcome status.
func (r *Runner) executeRollback() state.RolledBack {
	result := r.Stack.Execute()
	if result.Err != nil {
		r.writer().Log("meowctl: error: rollback journal unreadable: %v\n", result.Err)
		return state.RolledBackFailed
	}
	if result.SkippedLines > 0 {
		r.writer().Log("meowctl: warning: rollback skipped %d malformed journal line(s) at lines %v; some ops may not have been reversed\n", result.SkippedLines, result.SkippedAt)
	}
	if len(result.Failures) == 0 {
		// Warning already logged above if SkippedLines > 0; treat as OK since
		// all parseable ops were reversed successfully.
		return state.RolledBackOK
	}
	r.writer().Log("meowctl: warning: rollback completed with %d failure(s):\n", len(result.Failures))
	for _, f := range result.Failures {
		r.writer().Log("  [%s] %s/%s: %v\n", f.Record.Kind, f.Record.Phase, f.Record.Component, f.Err)
	}
	return state.RolledBackPartial
}
