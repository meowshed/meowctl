// Package rollback implements the write-ahead log and inverse-op execution for
// the meowctl lifecycle engine. Each reversible ctx operation appends an OpRecord
// to the rollback journal before executing. On failure the stack is replayed in
// reverse order to restore the prior system state.
package rollback

import (
	"bufio"
	"encoding/json"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"strings"
	"sync"
)

// Kind constants identify the forward operation recorded in an OpRecord.
const (
	KindWriteFile    = "write_file"
	KindAppendFile   = "append_file"
	KindCopyFile     = "copy_file"
	KindSymlink      = "symlink"
	KindLinkFile     = "link_file"
	KindMkdir        = "mkdir"
	KindDownload     = "download"
	KindDefaultWrite = "defaults_write"
	KindPlistSet     = "plist_set"
)

// OpRecord is one entry in the write-ahead log. Inverse holds the
// operation-specific data needed to undo Kind.
type OpRecord struct {
	Seq       int             `json:"seq"`
	Phase     string          `json:"phase"`
	Component string          `json:"component"`
	Kind      string          `json:"kind"`
	Inverse   json.RawMessage `json:"inverse"`
}

// inverseWriteFile is stored when write_file creates a new file (no prior content).
type inverseWriteFile struct {
	Path string `json:"path"`
	// PriorContent is non-empty when the file existed before; empty means delete.
	PriorContent string `json:"prior_content,omitempty"`
	HadPrior     bool   `json:"had_prior"`
}

// inverseAppendFile is stored when append_file appends a marked block.
type inverseAppendFile struct {
	Path   string `json:"path"`
	Marker string `json:"marker"` // UUID used in BEGIN/END comment markers.
}

// inverseCopyFile deletes the destination.
type inverseCopyFile struct {
	Dst string `json:"dst"`
}

// inverseSymlink removes or restores the symlink at Dst.
// When HadPrior is true the symlink previously pointed to PriorTarget and
// rollback re-creates it; when false rollback simply removes the symlink.
type inverseSymlink struct {
	Dst         string `json:"dst"`
	PriorTarget string `json:"prior_target,omitempty"`
	HadPrior    bool   `json:"had_prior"`
}

// inverseLinkFile removes the symlink at Dst. When WasBackedUp is true the
// original file was renamed to BackupPath and rollback restores it; when false
// rollback simply removes the symlink.
type inverseLinkFile struct {
	Dst         string `json:"dst"`
	BackupPath  string `json:"backup_path,omitempty"`
	WasBackedUp bool   `json:"was_backed_up"`
}

// inverseMkdir removes the directory if meowctl created it.
type inverseMkdir struct {
	Path             string `json:"path"`
	CreatedByMeowctl bool   `json:"created_by_meowctl"`
}

// inverseDownload deletes the downloaded file, or restores prior content if
// the destination was overwritten.
type inverseDownload struct {
	Dst          string `json:"dst"`
	PriorContent string `json:"prior_content,omitempty"`
	HadPrior     bool   `json:"had_prior"`
}

// Stack is a write-ahead log backed by a file on disk. Append an op before
// executing it; call Truncate after a successful run. Stack is safe for
// concurrent use from a single process.
type Stack struct {
	mu   sync.Mutex
	path string
	seq  int
	f    *os.File
}

// Open opens or creates the rollback journal at path. Returns an error if the
// file cannot be opened.
func Open(path string) (*Stack, error) {
	if err := os.MkdirAll(filepath.Dir(path), 0o700); err != nil {
		return nil, fmt.Errorf("rollback: create dir: %w", err)
	}
	f, err := os.OpenFile(path, os.O_RDWR|os.O_CREATE|os.O_APPEND, 0o600) // #nosec G304
	if err != nil {
		return nil, fmt.Errorf("rollback: open journal: %w", err)
	}
	// Count existing records to determine next seq.
	seq, err := countLines(f)
	if err != nil {
		_ = f.Close()
		return nil, fmt.Errorf("rollback: count existing records: %w", err)
	}
	return &Stack{path: path, seq: seq, f: f}, nil
}

// Close closes the underlying file handle.
// Safe to call after a failed Truncate (s.f may be nil in that case).
func (s *Stack) Close() error {
	s.mu.Lock()
	defer s.mu.Unlock()
	if s.f == nil {
		return nil
	}
	return s.f.Close()
}

// Pending reports whether the journal contains any un-replayed records.
func (s *Stack) Pending() bool {
	s.mu.Lock()
	defer s.mu.Unlock()
	return s.seq > 0
}

// Append records op to the write-ahead log before the forward operation runs.
func (s *Stack) Append(phase, component, kind string, inverse any) error {
	s.mu.Lock()
	defer s.mu.Unlock()
	if s.f == nil {
		return fmt.Errorf("rollback: stack is broken (truncate failed); cannot append")
	}
	raw, err := json.Marshal(inverse)
	if err != nil {
		return fmt.Errorf("rollback: marshal inverse: %w", err)
	}
	rec := OpRecord{
		Seq:       s.seq,
		Phase:     phase,
		Component: component,
		Kind:      kind,
		Inverse:   raw,
	}
	line, err := json.Marshal(rec)
	if err != nil {
		return fmt.Errorf("rollback: marshal record: %w", err)
	}
	line = append(line, '\n')
	if _, err := s.f.Write(line); err != nil {
		return fmt.Errorf("rollback: write journal: %w", err)
	}
	s.seq++
	return nil
}

// Truncate removes all records from the journal after a successful run.
// It reopens the file without O_APPEND so that subsequent writes start at
// offset 0 on all platforms (Linux ignores seeks on O_APPEND file descriptors).
func (s *Stack) Truncate() error {
	s.mu.Lock()
	defer s.mu.Unlock()
	if err := s.f.Truncate(0); err != nil {
		return fmt.Errorf("rollback: truncate journal: %w", err)
	}
	// Reset seq immediately so that any future Append on a recovered stack uses
	// the correct starting counter, even if the reopen below fails.
	s.seq = 0
	// Close before reopen; if reopen fails, record the error and leave s.f nil
	// rather than holding a stale closed handle.
	if err := s.f.Close(); err != nil {
		s.f = nil
		return fmt.Errorf("rollback: close journal before reopen: %w", err)
	}
	s.f = nil
	// Reopen without O_APPEND so the write cursor is at the correct offset.
	f, err := os.OpenFile(s.path, os.O_RDWR|os.O_CREATE, 0o600) // #nosec G304 — path is the journal path set at Open time
	if err != nil {
		return fmt.Errorf("rollback: reopen journal after truncate: %w", err)
	}
	s.f = f
	return nil
}

// Execute reads all records from the journal, reverses them, and applies each
// inverse operation. Failures are collected; execution continues past them.
// Resets the in-memory seq counter so that subsequent Append calls use correct
// sequence numbers if the Stack is reused after a rollback.
// Returns a Result describing what succeeded and what failed.
func (s *Stack) Execute() Result {
	s.mu.Lock()
	defer s.mu.Unlock()
	records, skipped, skippedAt, err := readAll(s.f)
	if err != nil {
		return Result{Err: fmt.Errorf("rollback: read journal: %w", err)}
	}
	// Reverse LIFO.
	for i, j := 0, len(records)-1; i < j; i, j = i+1, j-1 {
		records[i], records[j] = records[j], records[i]
	}
	var result Result
	result.SkippedLines = skipped
	result.SkippedAt = skippedAt
	for _, rec := range records {
		if err := applyInverse(rec); err != nil {
			result.Failures = append(result.Failures, Failure{Record: rec, Err: err})
		}
	}
	// Reset seq so any subsequent Append (e.g. in a retry scenario) starts from 0.
	s.seq = 0
	return result
}

// Result summarises a rollback execution.
type Result struct {
	Failures []Failure
	// SkippedLines is the count of malformed journal lines skipped during replay.
	// A non-zero value means some ops may not have been reversed; the caller
	// should warn the operator.
	SkippedLines int
	// SkippedAt holds the 1-based line numbers of the malformed lines that were
	// skipped. Useful for directing the operator to specific positions in the
	// journal file.
	SkippedAt []int
	// Err is set when the journal could not even be read.
	Err error
}

// Failure records one op that could not be reversed.
type Failure struct {
	Record OpRecord
	Err    error
}

// applyInverse executes the inverse of one recorded op.
func applyInverse(rec OpRecord) error {
	switch rec.Kind {
	case KindWriteFile:
		return applyInverseWriteFile(rec.Inverse)
	case KindAppendFile:
		return applyInverseAppendFile(rec.Inverse)
	case KindCopyFile:
		return applyInverseCopyFile(rec.Inverse)
	case KindSymlink:
		return applyInverseSymlink(rec.Inverse)
	case KindLinkFile:
		return applyInverseLinkFile(rec.Inverse)
	case KindMkdir:
		return applyInverseMkdir(rec.Inverse)
	case KindDownload:
		return applyInverseDownload(rec.Inverse)
	case KindDefaultWrite, KindPlistSet:
		// macOS system-preference calls — inverse not yet implemented.
		// Log a warning so the operator knows the system state may not be
		// fully restored after a rollback.
		return fmt.Errorf("rollback: %s inverse not implemented; system preference may not be restored", rec.Kind)
	default:
		return fmt.Errorf("rollback: unknown op kind %q", rec.Kind)
	}
}

func applyInverseWriteFile(raw json.RawMessage) error {
	var inv inverseWriteFile
	if err := json.Unmarshal(raw, &inv); err != nil {
		return err
	}
	if inv.HadPrior {
		return os.WriteFile(inv.Path, []byte(inv.PriorContent), 0o600) // #nosec G306
	}
	return deleteIfExists(inv.Path)
}

func applyInverseAppendFile(raw json.RawMessage) error {
	var inv inverseAppendFile
	if err := json.Unmarshal(raw, &inv); err != nil {
		return err
	}
	return removeMarkedBlock(inv.Path, inv.Marker)
}

func applyInverseCopyFile(raw json.RawMessage) error {
	var inv inverseCopyFile
	if err := json.Unmarshal(raw, &inv); err != nil {
		return err
	}
	return deleteIfExists(inv.Dst)
}

func applyInverseSymlink(raw json.RawMessage) error {
	var inv inverseSymlink
	if err := json.Unmarshal(raw, &inv); err != nil {
		return err
	}
	if inv.HadPrior {
		// Re-create the prior symlink. Remove whatever now sits at dst first
		// (either the new symlink or nothing).
		if err := deleteIfExists(inv.Dst); err != nil {
			return fmt.Errorf("rollback: remove new symlink %s: %w", inv.Dst, err)
		}
		if err := os.Symlink(inv.PriorTarget, inv.Dst); err != nil {
			return fmt.Errorf("rollback: restore symlink %s -> %s: %w", inv.Dst, inv.PriorTarget, err)
		}
		return nil
	}
	return deleteIfExists(inv.Dst)
}

func applyInverseLinkFile(raw json.RawMessage) error {
	var inv inverseLinkFile
	if err := json.Unmarshal(raw, &inv); err != nil {
		return err
	}
	if err := deleteIfExists(inv.Dst); err != nil {
		return fmt.Errorf("rollback: remove link_file symlink %s: %w", inv.Dst, err)
	}
	if inv.WasBackedUp {
		if err := os.Rename(inv.BackupPath, inv.Dst); err != nil {
			return fmt.Errorf("rollback: restore backup %s -> %s: %w", inv.BackupPath, inv.Dst, err)
		}
	}
	return nil
}

// applyInverseMkdir removes a directory that meowctl created. It uses
// os.Remove which only removes empty directories; if inner files were not
// rolled back first (LIFO invariant violated), Remove returns ENOTEMPTY.
func applyInverseMkdir(raw json.RawMessage) error {
	var inv inverseMkdir
	if err := json.Unmarshal(raw, &inv); err != nil {
		return err
	}
	if !inv.CreatedByMeowctl {
		return nil
	}
	if err := os.Remove(inv.Path); err != nil && !os.IsNotExist(err) {
		return fmt.Errorf("rollback: remove dir %s: %w (directory may not be empty — check for failed inner-op rollbacks)", inv.Path, err)
	}
	return nil
}

func applyInverseDownload(raw json.RawMessage) error {
	var inv inverseDownload
	if err := json.Unmarshal(raw, &inv); err != nil {
		return err
	}
	if inv.HadPrior {
		return os.WriteFile(inv.Dst, []byte(inv.PriorContent), 0o600) // #nosec G306
	}
	return deleteIfExists(inv.Dst)
}

// deleteIfExists removes path if it exists; no error if already absent.
func deleteIfExists(path string) error {
	err := os.Remove(path)
	if os.IsNotExist(err) {
		return nil
	}
	return err
}

// removeMarkedBlock removes the BEGIN/END block identified by marker from path.
// Returns an error if a BEGIN marker is found without a matching END marker,
// which would otherwise silently drop all lines after the truncated block.
func removeMarkedBlock(path, marker string) error {
	data, err := os.ReadFile(path) // #nosec G304
	if err != nil {
		if os.IsNotExist(err) {
			return nil
		}
		return err
	}
	begin := "# BEGIN meowctl:" + marker
	end := "# END meowctl:" + marker
	lines := strings.Split(string(data), "\n")
	out := make([]string, 0, len(lines))
	skip := false
	for _, line := range lines {
		if strings.TrimSpace(line) == begin {
			skip = true
			continue
		}
		if strings.TrimSpace(line) == end {
			skip = false
			continue
		}
		if !skip {
			out = append(out, line)
		}
	}
	// If skip is still true after scanning all lines, the BEGIN marker was
	// found but END was never encountered — the block is truncated. Returning
	// an error prevents silent data loss of all lines after the marker.
	if skip {
		return fmt.Errorf("rollback: unterminated block in %s (BEGIN meowctl:%s has no matching END)", path, marker)
	}
	// strings.Split on a newline-terminated file produces a trailing "" element,
	// which strings.Join preserves as a trailing newline — matching the original file.
	return os.WriteFile(path, []byte(strings.Join(out, "\n")), 0o600) // #nosec G306,G703
}

// countLines counts valid JSON lines in f by scanning from the start.
// Only lines that successfully unmarshal as OpRecord are counted, so corrupt
// or partial trailing lines do not inflate the seq counter.
// Note: if the journal contains corrupt mid-file lines, seq will equal the
// number of valid records, not the maximum Seq value. Subsequent Append calls
// may assign a Seq number already used by a record that was skipped. Seq is
// therefore a logical record counter, not a guaranteed-unique identifier.
func countLines(f *os.File) (int, error) {
	if _, err := f.Seek(0, io.SeekStart); err != nil {
		return 0, err
	}
	n := 0
	sc := bufio.NewScanner(f)
	for sc.Scan() {
		line := sc.Text()
		if line == "" {
			continue
		}
		var rec OpRecord
		if json.Unmarshal([]byte(line), &rec) == nil {
			n++
		}
	}
	if err := sc.Err(); err != nil {
		return 0, err
	}
	// Leave file pointer at end for subsequent appends.
	if _, err := f.Seek(0, io.SeekEnd); err != nil {
		return 0, err
	}
	return n, nil
}

// readAll reads all valid OpRecords from the file from the beginning.
// Malformed or partial lines are skipped — consistent with countLines — so a
// partially-written trailing record does not abort the entire rollback replay.
// Returns the records, the count of skipped lines, and a slice of the 1-based
// line numbers that were skipped so callers can warn the operator with position info.
func readAll(f *os.File) ([]OpRecord, int, []int, error) {
	if _, err := f.Seek(0, io.SeekStart); err != nil {
		return nil, 0, nil, err
	}
	var records []OpRecord
	var skippedAt []int
	lineNum := 0 // counts every line including blanks; reported positions match text-editor line numbers
	sc := bufio.NewScanner(f)
	for sc.Scan() {
		line := sc.Text()
		lineNum++
		if line == "" {
			// blank lines advance lineNum so malformed-line positions remain accurate
			continue
		}
		var rec OpRecord
		if json.Unmarshal([]byte(line), &rec) == nil {
			records = append(records, rec)
		} else {
			skippedAt = append(skippedAt, lineNum)
		}
	}
	return records, len(skippedAt), skippedAt, sc.Err()
}

// AppendWriteFile records a write_file inverse op before execution.
// priorContent is empty and hadPrior false when the file did not exist.
func (s *Stack) AppendWriteFile(phase, component, path, priorContent string, hadPrior bool) error {
	return s.Append(phase, component, KindWriteFile, inverseWriteFile{
		Path:         path,
		PriorContent: priorContent,
		HadPrior:     hadPrior,
	})
}

// AppendAppendFile records an append_file inverse op before execution.
func (s *Stack) AppendAppendFile(phase, component, path, marker string) error {
	return s.Append(phase, component, KindAppendFile, inverseAppendFile{
		Path:   path,
		Marker: marker,
	})
}

// AppendCopyFile records a copy_file inverse op before execution.
func (s *Stack) AppendCopyFile(phase, component, dst string) error {
	return s.Append(phase, component, KindCopyFile, inverseCopyFile{Dst: dst})
}

// AppendSymlink records a symlink inverse op before execution.
// priorTarget is the target the existing symlink pointed to (empty string when
// no prior symlink existed). hadPrior must be true when an existing symlink is
// being replaced so that rollback can restore the original link.
func (s *Stack) AppendSymlink(phase, component, dst, priorTarget string, hadPrior bool) error {
	return s.Append(phase, component, KindSymlink, inverseSymlink{
		Dst:         dst,
		PriorTarget: priorTarget,
		HadPrior:    hadPrior,
	})
}

// AppendLinkFile records a link_file inverse op before execution.
// backupPath is the path the original file was renamed to (empty when no
// backup was made). wasBackedUp must be true when an existing regular file was
// backed up so that rollback can restore it.
func (s *Stack) AppendLinkFile(phase, component, dst, backupPath string, wasBackedUp bool) error {
	return s.Append(phase, component, KindLinkFile, inverseLinkFile{
		Dst:         dst,
		BackupPath:  backupPath,
		WasBackedUp: wasBackedUp,
	})
}

// AppendMkdir records a mkdir inverse op before execution.
func (s *Stack) AppendMkdir(phase, component, path string, createdByMeowctl bool) error {
	return s.Append(phase, component, KindMkdir, inverseMkdir{
		Path:             path,
		CreatedByMeowctl: createdByMeowctl,
	})
}

// AppendDownload records a download inverse op before execution.
// priorContent and hadPrior capture any pre-existing file at dst so that
// rollback can restore it rather than simply deleting the downloaded file.
func (s *Stack) AppendDownload(phase, component, dst, priorContent string, hadPrior bool) error {
	return s.Append(phase, component, KindDownload, inverseDownload{
		Dst:          dst,
		PriorContent: priorContent,
		HadPrior:     hadPrior,
	})
}
