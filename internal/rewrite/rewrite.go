// Package rewrite provides targeted string-substitution utilities for meowctl config files.
// It performs surgical edits (e.g. bumping a dep version, adding/removing components)
// without reformatting files.
package rewrite

import (
	"fmt"
	"os"
	"regexp"
	"strings"
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
	return os.WriteFile(path, updated, 0o600) // #nosec G703 -- path is caller-controlled trusted modfile path; not user input
}

// AppendComponent appends a component("<name>") declaration to the Starlark file at path.
// Creates the file if it does not exist.
func AppendComponent(path, name string) error {
	var existing string
	data, readErr := os.ReadFile(path) // #nosec G304
	if readErr != nil && !os.IsNotExist(readErr) {
		return fmt.Errorf("rewrite: read %s: %w", path, readErr)
	}
	if readErr == nil {
		existing = string(data)
	}

	// Ensure a newline separator.
	if len(existing) > 0 && existing[len(existing)-1] != '\n' {
		existing += "\n"
	}
	existing += fmt.Sprintf("component(%q)\n", name)
	return os.WriteFile(path, []byte(existing), 0o600) // #nosec G703
}

// RemoveComponent removes all component("<name>") lines from the Starlark file at path.
// Returns an error if the file does not exist or the component line is not found.
func RemoveComponent(path, name string) error {
	data, err := os.ReadFile(path) // #nosec G304
	if err != nil {
		return fmt.Errorf("rewrite: read %s: %w", path, err)
	}

	// Pattern: component("<name>") with optional whitespace, whole line.
	pat := regexp.MustCompile(`(?m)^[ \t]*component\(` + regexp.QuoteMeta(`"`+name+`"`) + `\)[ \t]*\n?`)
	if !pat.Match(data) {
		return fmt.Errorf("rewrite: component %q not found in %s", name, path)
	}

	updated := pat.ReplaceAll(data, nil)
	// Normalise trailing newline.
	result := strings.TrimRight(string(updated), "\n")
	if result != "" {
		result += "\n"
	}
	return os.WriteFile(path, []byte(result), 0o600) // #nosec G703
}
