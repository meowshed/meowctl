---
id: SPC-2400
artifact: spec
status: live
revised: 2026-09-27
checked-at:
states: [REQ-2401, REQ-2402, REQ-2403, REQ-2404, REQ-2405, REQ-2410, REQ-2411, REQ-2412, REQ-2413, REQ-2420, REQ-2421, REQ-2422, REQ-2430, REQ-2431, REQ-2432, REQ-2433, REQ-2434, REQ-2440, REQ-2441, REQ-2442, REQ-2443, REQ-2444, REQ-2445, REQ-2450, REQ-2451, REQ-2452, REQ-2453, REQ-2460, REQ-2461, REQ-2462, REQ-2463, REQ-2464, REQ-2500, REQ-2501, REQ-2502, REQ-2503, REQ-2504, REQ-2505, REQ-2506, REQ-2507, REQ-2508, REQ-2509, REQ-2510, REQ-2511, REQ-2512, REQ-2513, REQ-2514, REQ-2540]
---

# Module resolution

## Scope

This component resolves the module graph: version selection, fetching from the
registry or from GitHub, integrity verification, the on-disk cache, and local or
remote overrides (from docs/spec/module.md, high). It hands `meowctl-starlark` a
verified file for a URL and hands `meowctl-config` the resolution to lock (from
docs/spec/module.md, high). It lives in the `meowctl-module` crate, its design
is in `docs/design/0.2.0-rust-rewrite.md` §3, and its `v0.1.0` equivalents are
`internal/mvs/` and `internal/starlark/loader/` (from docs/spec/module.md,
high).

## Boundary

Resolution, fetching, caching, verification, and the resolved values written to
`deps.lock` (from docs/spec/module.md, high).

## Behaviour

### Version selection

Version selection is Minimal Version Selection over the dependency graph, the
algorithm in `internal/mvs/mvs.go`: for each module it selects the maximum
version any path requires [REQ-2401] (from docs/spec/module.md, high).

Versions compare by semver ordering, and the literal `none` orders below every
valid version [REQ-2402] (from docs/spec/module.md, high).

A resolution is deterministic [REQ-2404]: the same graph produces the same build
list in the same order on every run and every machine [REQ-2500] (from
docs/spec/module.md, high).

A cycle in the dependency graph terminates, because resolution doesn't re-expand
a module at a version it has already visited [REQ-2405] (from
docs/spec/module.md, high).

### Sources

A registry module resolves through the registry index, which lists each module's
versions, tarball URLs and integrity hashes [REQ-2410] (from
docs/spec/module.md, high).

A GitHub module is named `github:owner/repo@ref`, where the ref is a tag or a
branch [REQ-2411] (from docs/spec/module.md, high). It resolves to the commit
SHA the ref pointed to at resolution time [REQ-2501], and the SHA is recorded so
later fetches are reproducible [REQ-2502] (from docs/spec/module.md, high). Its
transitive dependencies are read from its own manifest and walked, so an
aggregate module brings its dependencies with it [REQ-2412] (from
docs/spec/module.md, high).

Fetching uses HTTPS [REQ-2413] and never shells out to `git` [REQ-2503] (from
docs/spec/module.md, high).

### Overrides

`replace(name, path)` serves every file for that module from the local directory
and skips fetch, cache and integrity verification [REQ-2420], and its lock entry
records that the module is replaced and where it points [REQ-2504] (from
docs/spec/module.md, high).

`replace(name, source)` resolves the module from the replacement source
[REQ-2421] and verifies its integrity as it would the original's [REQ-2505]
(from docs/spec/module.md, high).

An override in `deps.local.mod` wins over the same module in `deps.mod`
[REQ-2422] (from docs/spec/module.md, high).

### Integrity

Every fetched tarball is verified against the integrity hash from the index
before it is extracted [REQ-2430], and every extracted file is verified against
its recorded per-file hash before it is evaluated [REQ-2506] (from
docs/spec/module.md, high). The per-file hashes live in the cache beside the
extracted files and not in `deps.lock` [REQ-2506] (from docs/spec/module.md,
high).

Extraction keeps the executable bit from the tar entry and writes the on-disk
mode as `0o644` or `0o755` [REQ-2432] (from docs/spec/module.md, high).

Extraction drops a single top-level directory when every entry in the archive is
inside one [REQ-2434], and drops nothing otherwise [REQ-2509] (from
docs/spec/module.md, high). So a GitHub archive holding
`repo-<commit>/MODULE.meow` and a release tarball holding `MODULE.meow` at its
root both extract with `MODULE.meow` at the module root (from
docs/spec/module.md, high).

### The cache

A module is cached under the cache directory keyed by name and resolved version,
or by commit SHA for a GitHub module [REQ-2440] (from docs/spec/module.md,
high).

A cached module whose per-file hashes match the record written at extraction is
used without a network request [REQ-2441] (from docs/spec/module.md, high). A
cached module whose files don't match the record is re-fetched [REQ-2442], and
so is a cached module with no record at all, such as one a `v0.1.0` binary
extracted [REQ-2510] (from docs/spec/module.md, high).

Resolution works offline when every module in the lock is cached and verified
[REQ-2443] (from docs/spec/module.md, high).

A dry run populates the cache [REQ-2445] (from docs/spec/module.md, high).

The cache record sits inside the cache directory under a name that can't be
loaded as a module file [REQ-2444], and it covers the tarball's integrity as
well as the per-file hashes [REQ-2511] (from docs/spec/module.md, high). The
tarball hash stays in the `.sri` sidecar in the format `v0.1.0` reads, and the
per-file hashes go in a second file beside it, which `v0.1.0` ignores [REQ-2512]
(from docs/spec/module.md, high).

### Locking

A module already in the lock is used at the locked version without running
selection again; selection runs when a module is absent or when the caller asks
to ignore the lock [REQ-2450] (from docs/spec/module.md, high).

Syncing writes the full resolution: version, source, integrity and, where it
applies, commit SHA [REQ-2451] (from docs/spec/module.md, high).

Syncing `deps.mod` and `deps.local.mod` produces a lock file for each
[REQ-2452], and a module in both resolves independently in each [REQ-2513] (from
docs/spec/module.md, high).

An upgrade clears the locked version for the named modules and re-resolves them,
and leaves the rest of the lock untouched [REQ-2453] (from docs/spec/module.md,
high).

## Failure paths

### Version selection

A dependency declaring a version that isn't valid semver fails resolution,
naming the module and the version, and is neither skipped nor treated as `none`
[REQ-2403] (from docs/spec/module.md, high).

A requirement no version satisfies fails, naming the module and the constraint
[REQ-2462] (from docs/spec/module.md, high).

### Registry index

A registry index that can't be fetched fails with the module code [REQ-2460] and
says whether the network, the status code or the parse failed [REQ-2514] (from
docs/spec/module.md, high).

A module named in `deps.mod` but absent from the index is reported by name,
distinct from a version of it that doesn't exist [REQ-2461] (from
docs/spec/module.md, high).

### GitHub sources

A GitHub ref that doesn't resolve to a commit fails with the module code, naming
the repository and the ref, and reports the missing ref apart from an
unreachable or rate-limited API [REQ-2540].

### Integrity and extraction

A hash mismatch fails with the module code [REQ-2431], names the module, the
expected hash and the actual one [REQ-2507], and leaves no extracted files in
the cache [REQ-2508] (from docs/spec/module.md, high).

Extraction rejects an entry whose path escapes the module root [REQ-2433] (from
docs/spec/module.md, high).

A fetch interrupted partway leaves neither a partial tarball nor a partially
extracted module that a later run would treat as cached [REQ-2463] (from
docs/spec/module.md, high). Extraction writes beside the cache entry and renames
it into place, so the entry's directory exists only once it is complete
[REQ-2463] (from docs/spec/module.md, high).

When the cache record can't be written, the fetch fails with the module code and
names the path, and the cache holds no entry for that key, because the entry's
directory appears only by the rename [REQ-2463] (from
crates/meowctl-module/src/cache.rs:208-270, high).

### Overrides

A `replace` pointing at a path that doesn't exist fails naming the path and
doesn't fall back to fetching the original [REQ-2464] (from docs/spec/module.md,
high).
