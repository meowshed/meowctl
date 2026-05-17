// Package lock defines the meowctl lock file schema and types.
// The lock file is a TOML document stored at ~/.config/meowctl/meowctl.lock.
// It records the exact resolved state of all modules, GitHub file pins, and
// package manager installations so that subsequent runs are reproducible.
package lock

// LockFile is the top-level structure of meowctl.lock.
//
//nolint:revive // "LockFile" stutters but "File" is too generic at call sites.
type LockFile struct {
	Meta     LockMeta                   `toml:"meta"`
	Modules  map[string]ModuleEntry     `toml:"modules"`
	GitHub   map[string]GitHubEntry     `toml:"github-modules"`
	Packages map[string]ManagerPackages `toml:"packages"`
}

// LockMeta holds file-level metadata written on every update.
//
//nolint:revive // "LockMeta" stutters but "Meta" conflicts with the LockFile.Meta field name.
type LockMeta struct {
	// GeneratedBy is the meowctl version that last wrote this file.
	GeneratedBy string `toml:"generated-by"`
	// UpdatedAt is an RFC 3339 timestamp of the last write.
	UpdatedAt string `toml:"updated-at"`
}

// ModuleEntry records the resolved state of one registry or GitHub module.
type ModuleEntry struct {
	// Version is the resolved semver version selected by MVS (registry deps only).
	Version string `toml:"version"`
	// Source is the canonical URL of the source tarball.
	Source string `toml:"source"`
	// Integrity is the W3C SRI hash of the tarball (e.g. "sha384-<base64>").
	Integrity string `toml:"integrity"`
	// Files maps each extracted file path (relative to module root) to its
	// individual SRI hash, enabling per-file integrity verification.
	Files map[string]string `toml:"files"`
	// CommitSHA is the full Git commit SHA resolved at sync time for GitHub deps.
	// For registry deps this is empty. For GitHub deps (source = "github:..."),
	// this is the commit the tag or branch resolved to, ensuring reproducible fetches.
	CommitSHA string `toml:"commit-sha,omitempty"`
	// Replaced is true when this module is replaced by a local path via replace().
	// When true, Source and Integrity are empty; Path holds the local directory.
	Replaced bool `toml:"replaced,omitempty"`
	// Path is the local filesystem path when Replaced is true.
	Path string `toml:"path,omitempty"`
}

// GitHubEntry records a pinned GitHub file reference resolved from a ref to a commit SHA.
type GitHubEntry struct {
	// Commit is the full SHA-1 commit hash the ref resolved to at lock time.
	Commit string `toml:"commit"`
	// Integrity is the W3C SRI hash of the fetched file content.
	Integrity string `toml:"integrity"`
}

// ManagerPackages groups all package entries for one package manager component.
// The map key is the component name (e.g. "homebrew", "apt").
type ManagerPackages map[string]PackageEntry

// PackageEntry records the installed state of one package under a manager.
type PackageEntry struct {
	// Requested is the version constraint specified in the component manifest.
	Requested string `toml:"requested"`
	// Installed is the exact version string reported by the package manager after install.
	Installed string `toml:"installed"`
	// Note carries an optional human-readable remark (e.g. "pinned: CVE-2025-1234").
	Note string `toml:"note,omitempty"`
}
