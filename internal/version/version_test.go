package version_test

import (
	"runtime"
	"strings"
	"testing"

	"github.com/meowshed/meowctl/internal/version"
)

func TestString_DefaultValues(t *testing.T) {
	s := version.String()

	if !strings.HasPrefix(s, "meowctl ") {
		t.Errorf("version string must start with 'meowctl ': got %q", s)
	}
	if !strings.Contains(s, runtime.GOOS) {
		t.Errorf("version string must contain GOOS %q: got %q", runtime.GOOS, s)
	}
	if !strings.Contains(s, runtime.GOARCH) {
		t.Errorf("version string must contain GOARCH %q: got %q", runtime.GOARCH, s)
	}
	if !strings.Contains(s, "commit") {
		t.Errorf("version string must contain 'commit': got %q", s)
	}
	if !strings.Contains(s, "built") {
		t.Errorf("version string must contain 'built': got %q", s)
	}
}

func TestString_CustomValues(t *testing.T) {
	// NOTE: Version, Commit, BuildDate are exported package-level vars required
	// for ldflags injection. Direct mutation is safe here because tests run
	// sequentially (CGO_ENABLED=0 prevents -race). If parallelism is added in
	// future, replace with a version.New(v, commit, date) constructor instead.
	orig := version.Version
	origCommit := version.Commit
	origDate := version.BuildDate
	defer func() {
		version.Version = orig
		version.Commit = origCommit
		version.BuildDate = origDate
	}()

	version.Version = "1.2.3"
	version.Commit = "abc1234"
	version.BuildDate = "2026-01-01"

	s := version.String()
	expected := "meowctl 1.2.3 (" + runtime.GOOS + "/" + runtime.GOARCH + ", commit abc1234, built 2026-01-01)"
	if s != expected {
		t.Errorf("expected %q, got %q", expected, s)
	}
}
