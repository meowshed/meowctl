package rollback_test

import (
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/meowshed/meowctl/internal/rollback"
)

func openStack(t *testing.T) (*rollback.Stack, string) {
	t.Helper()
	dir := t.TempDir()
	path := filepath.Join(dir, "rollback.jsonl")
	s, err := rollback.Open(path)
	if err != nil {
		t.Fatalf("Open: %v", err)
	}
	t.Cleanup(func() { _ = s.Close() })
	return s, dir
}

func TestRollback_WriteFile_New(t *testing.T) {
	s, dir := openStack(t)
	path := filepath.Join(dir, "file.txt")

	// Record + execute write.
	if err := s.AppendWriteFile("install", "test", path, "", false); err != nil {
		t.Fatalf("AppendWriteFile: %v", err)
	}
	if err := os.WriteFile(path, []byte("hello"), 0o600); err != nil {
		t.Fatal(err)
	}

	// Rollback should delete the file.
	result := s.Execute()
	if result.Err != nil {
		t.Fatalf("Execute: %v", result.Err)
	}
	if len(result.Failures) > 0 {
		t.Fatalf("unexpected failures: %v", result.Failures)
	}
	if _, err := os.Stat(path); !os.IsNotExist(err) {
		t.Errorf("expected file to be deleted after rollback")
	}
}

func TestRollback_WriteFile_Existing(t *testing.T) {
	s, dir := openStack(t)
	path := filepath.Join(dir, "file.txt")
	if err := os.WriteFile(path, []byte("original"), 0o600); err != nil {
		t.Fatal(err)
	}

	if err := s.AppendWriteFile("install", "test", path, "original", true); err != nil {
		t.Fatalf("AppendWriteFile: %v", err)
	}
	if err := os.WriteFile(path, []byte("modified"), 0o600); err != nil {
		t.Fatal(err)
	}

	result := s.Execute()
	if result.Err != nil || len(result.Failures) > 0 {
		t.Fatalf("rollback failed: err=%v failures=%v", result.Err, result.Failures)
	}
	got, _ := os.ReadFile(path) // #nosec G304
	if string(got) != "original" {
		t.Errorf("expected original content restored, got %q", got)
	}
}

func TestRollback_AppendFile_Marker(t *testing.T) {
	s, dir := openStack(t)
	path := filepath.Join(dir, "rc.sh")
	// Create file with some pre-existing content.
	existing := "# existing\n"
	if err := os.WriteFile(path, []byte(existing), 0o600); err != nil {
		t.Fatal(err)
	}
	marker := "test-uuid-1234"

	if err := s.AppendAppendFile("install", "test", path, marker); err != nil {
		t.Fatalf("AppendAppendFile: %v", err)
	}
	// Simulate the append.
	block := "\n# BEGIN meowctl:" + marker + "\nexport FOO=bar\n# END meowctl:" + marker + "\n"
	f, _ := os.OpenFile(path, os.O_APPEND|os.O_WRONLY, 0o600) // #nosec G304
	_, _ = f.WriteString(block)
	_ = f.Close()

	result := s.Execute()
	if result.Err != nil || len(result.Failures) > 0 {
		t.Fatalf("rollback failed: err=%v failures=%v", result.Err, result.Failures)
	}
	got, _ := os.ReadFile(path) // #nosec G304
	if strings.Contains(string(got), "BEGIN meowctl:"+marker) {
		t.Errorf("marker block should have been removed; file content:\n%s", got)
	}
	if !strings.Contains(string(got), "# existing") {
		t.Errorf("pre-existing content should remain; file content:\n%s", got)
	}
}

func TestRollback_Symlink(t *testing.T) {
	s, dir := openStack(t)
	src := filepath.Join(dir, "src.txt")
	dst := filepath.Join(dir, "link")
	if err := os.WriteFile(src, []byte("x"), 0o600); err != nil {
		t.Fatal(err)
	}

	if err := s.AppendSymlink("install", "test", dst, "", false); err != nil {
		t.Fatalf("AppendSymlink: %v", err)
	}
	if err := os.Symlink(src, dst); err != nil {
		t.Fatal(err)
	}

	result := s.Execute()
	if result.Err != nil || len(result.Failures) > 0 {
		t.Fatalf("rollback failed: %v %v", result.Err, result.Failures)
	}
	if _, err := os.Lstat(dst); !os.IsNotExist(err) {
		t.Errorf("expected symlink to be removed")
	}
}

func TestRollback_Mkdir_CreatedByMeowctl(t *testing.T) {
	s, dir := openStack(t)
	path := filepath.Join(dir, "newdir")

	if err := s.AppendMkdir("install", "test", path, true); err != nil {
		t.Fatalf("AppendMkdir: %v", err)
	}
	if err := os.Mkdir(path, 0o750); err != nil { //nolint:gosec
		t.Fatal(err)
	}

	result := s.Execute()
	if result.Err != nil || len(result.Failures) > 0 {
		t.Fatalf("rollback failed: %v %v", result.Err, result.Failures)
	}
	if _, err := os.Stat(path); !os.IsNotExist(err) {
		t.Errorf("expected dir to be removed")
	}
}

func TestRollback_Mkdir_NotCreatedByMeowctl(t *testing.T) {
	s, dir := openStack(t)
	// Pre-existing dir should not be removed.
	path := filepath.Join(dir, "existing")
	if err := os.Mkdir(path, 0o750); err != nil { //nolint:gosec
		t.Fatal(err)
	}

	if err := s.AppendMkdir("install", "test", path, false); err != nil {
		t.Fatalf("AppendMkdir: %v", err)
	}

	result := s.Execute()
	if result.Err != nil || len(result.Failures) > 0 {
		t.Fatalf("rollback failed: %v %v", result.Err, result.Failures)
	}
	if _, err := os.Stat(path); err != nil {
		t.Errorf("pre-existing dir should NOT have been removed: %v", err)
	}
}

func TestRollback_CopyFile(t *testing.T) {
	s, dir := openStack(t)
	src := filepath.Join(dir, "src.txt")
	dst := filepath.Join(dir, "dst.txt")
	if err := os.WriteFile(src, []byte("content"), 0o600); err != nil {
		t.Fatal(err)
	}

	if err := s.AppendCopyFile("install", "test", dst); err != nil {
		t.Fatalf("AppendCopyFile: %v", err)
	}
	// Simulate the copy.
	data, _ := os.ReadFile(src)                            // #nosec G304
	if err := os.WriteFile(dst, data, 0o600); err != nil { // #nosec G703
		t.Fatal(err)
	}

	result := s.Execute()
	if result.Err != nil || len(result.Failures) > 0 {
		t.Fatalf("rollback failed: err=%v failures=%v", result.Err, result.Failures)
	}
	if _, err := os.Stat(dst); !os.IsNotExist(err) {
		t.Errorf("expected dst to be deleted after rollback")
	}
}

func TestRollback_Truncate(t *testing.T) {
	s, dir := openStack(t)
	path := filepath.Join(dir, "f")
	_ = s.AppendWriteFile("install", "test", path, "", false)
	if !s.Pending() {
		t.Fatal("expected pending after append")
	}
	if err := s.Truncate(); err != nil {
		t.Fatalf("Truncate: %v", err)
	}
	if s.Pending() {
		t.Error("expected no pending after truncate")
	}
}
