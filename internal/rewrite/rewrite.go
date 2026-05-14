// Package rewrite provides targeted string-substitution utilities for meowctl.mod files.
// It performs surgical edits (e.g. bumping a dep version) without reformatting the file.
package rewrite

import (
	"fmt"
	"os"
	"regexp"
)

// SetDepVersion rewrites the version in a dep(url = "…@<version>") declaration
// for moduleName in the file at path.
//
// The substitution targets the first dep() line whose url contains moduleName
// followed by @<old-version>. It replaces the version token with newVersion.
//
// Returns an error if no matching dep declaration is found.
func SetDepVersion(path, moduleName, newVersion string) error {
	data, err := os.ReadFile(path) // #nosec G304 -- caller-controlled path; modfile is trusted config
	if err != nil {
		return fmt.Errorf("rewrite: read %s: %w", path, err)
	}

	// Pattern: dep(url = "…<moduleName>@<version>…")
	// We match the module name followed by @semver and replace only the version token.
	pat := regexp.MustCompile(`(dep\s*\([^)]*` + regexp.QuoteMeta(moduleName) + `@)[^\s"]+`)
	if !pat.Match(data) {
		return fmt.Errorf("rewrite: dep for module %q not found in %s", moduleName, path)
	}

	updated := pat.ReplaceAll(data, []byte(`${1}`+newVersion))
	return os.WriteFile(path, updated, 0o600) // #nosec G703 -- path is caller-controlled trusted modfile path
}
