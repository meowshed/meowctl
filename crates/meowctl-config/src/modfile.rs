//! `deps.mod`, the machine-managed module manifest.
//!
//! Reading one is evaluating it, which [`meowctl_starlark`] does: `dep()`,
//! `module()` and `replace()` land in its accumulator. This module writes one,
//! byte for byte as `modfile.Write` does, because two binaries share a
//! configuration directory and a manifest that differs only in whitespace
//! shows up as a change in every `git diff` a user takes of their dotfiles.
//!
//! [`meowctl_starlark`]: https://docs.rs/meowctl-starlark

use std::fmt::Write as _;
use std::path::Path;

use meowctl_fs::FileSystem;

use crate::ConfigResult;

/// The header `v0.1.0` writes.
///
/// It names `meowctl.mod`, which the rename to `deps.mod` left behind.
/// Reproduced as it is, because the file is compared byte for byte and
/// correcting a stale comment costs a corpus rebaseline and buys nothing a
/// user reads; see [R-CONFIG-014].
const HEADER: &str = "# meowctl.mod — machine-managed module file. Do not edit by hand.\n\n";

/// One `dep()` statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dep {
    /// The module.
    pub name: String,
    /// Its version, for a registry module.
    pub version: String,
    /// Its source, for a GitHub module.
    pub source: String,
}

/// One `replace()` statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Replace {
    /// The module being replaced.
    pub name: String,
    /// A local directory to serve it from.
    pub path: String,
    /// A different remote source to fetch it from.
    pub source: String,
}

/// The configuration's own module identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module {
    /// Its name.
    pub name: String,
    /// Its version.
    pub version: String,
}

/// What a `deps.mod` says.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Modfile {
    /// The `module()` statement, when there is one.
    pub module: Option<Module>,
    /// Every `dep()`, in declaration order.
    pub deps: Vec<Dep>,
    /// Every `replace()`, in declaration order.
    pub replaces: Vec<Replace>,
}

impl Modfile {
    /// Refuses an entry carrying both of its two fields, or neither.
    ///
    /// `internal/modfile/modfile.go` refuses these while parsing, and this is
    /// where that happens now: `v0.1.0` has one `dep()` for `deps.mod` that
    /// validates and another for `init.star` that takes no `source`, and
    /// [R-CONFIG-010] merged them into one builtin. A manifest that names
    /// both leaves the resolver choosing between two answers; see
    /// [R-CONFIG-012].
    ///
    /// # Errors
    ///
    /// [`crate::ConfigError::Malformed`] naming the entry that is wrong.
    pub fn check(&self, path: &std::path::Path) -> crate::ConfigResult<()> {
        let wrong = |what: &str, name: &str, one: &str, other: &str, both: bool| {
            Err(crate::ConfigError::Malformed {
                path: path.to_path_buf(),
                reason: if both {
                    format!("{what}(\"{name}\"): {one} and {other} are mutually exclusive")
                } else {
                    format!("{what}(\"{name}\"): exactly one of {one} or {other} is required")
                },
            })
        };

        for dep in &self.deps {
            match (dep.version.is_empty(), dep.source.is_empty()) {
                (false, false) => return wrong("dep", &dep.name, "version", "source", true),
                (true, true) => return wrong("dep", &dep.name, "version", "source", false),
                _ => {}
            }
        }
        for replace in &self.replaces {
            match (replace.path.is_empty(), replace.source.is_empty()) {
                (false, false) => return wrong("replace", &replace.name, "path", "source", true),
                (true, true) => return wrong("replace", &replace.name, "path", "source", false),
                _ => {}
            }
        }
        Ok(())
    }

    /// Renders the file exactly as `modfile.Write` renders it.
    ///
    /// The layout is not ours: the header, `module()` across four lines with
    /// indented keyword arguments, each `dep()` on one line, a blank line
    /// after the dependency block, and each `replace()` on one line.
    #[must_use]
    pub fn render(&self) -> String {
        let mut out = String::from(HEADER);

        if let Some(module) = &self.module {
            let _ = write!(
                out,
                "module(\n    name = {},\n    version = {},\n)\n\n",
                quote(&module.name),
                quote(&module.version)
            );
        }

        for dep in &self.deps {
            // Source and version are exclusive, and a dep carrying neither is
            // rejected before it reaches here; see [R-CONFIG-012].
            if dep.source.is_empty() {
                let _ = writeln!(
                    out,
                    "dep(name = {}, version = {})",
                    quote(&dep.name),
                    quote(&dep.version)
                );
            } else {
                let _ = writeln!(
                    out,
                    "dep(name = {}, source = {})",
                    quote(&dep.name),
                    quote(&dep.source)
                );
            }
        }
        if !self.deps.is_empty() {
            out.push('\n');
        }

        for replace in &self.replaces {
            if replace.source.is_empty() {
                let _ = writeln!(
                    out,
                    "replace(name = {}, path = {})",
                    quote(&replace.name),
                    quote(&replace.path)
                );
            } else {
                let _ = writeln!(
                    out,
                    "replace(name = {}, source = {})",
                    quote(&replace.name),
                    quote(&replace.source)
                );
            }
        }

        out
    }

    /// Writes the file.
    ///
    /// Atomically, unlike `modfile.Write`, which uses a plain write: a
    /// `deps.mod` truncated by a crash during `meowctl dep add` leaves a
    /// configuration that does not parse; see [R-CONFIG-002].
    ///
    /// # Errors
    ///
    /// Whatever the filesystem fails with.
    pub fn write(&self, fs: &dyn FileSystem, path: &Path) -> ConfigResult<()> {
        fs.write(path, self.render().as_bytes())?;
        Ok(())
    }
}

/// A Go `%q`-style quoted string.
///
/// The manifest is written by `fmt.Fprintf` with `%q`, which escapes what Go
/// escapes. Module names and versions are plain text, so the two agree for
/// everything that reaches here; the escaping is written out anyway, because a
/// path in a `replace()` can contain anything a filesystem allows.
fn quote(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\x{:02x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [R-CONFIG-014] the layout `modfile.Write` produces, which a user's
    /// `git diff` will otherwise show as a change.
    #[test]
    fn a_manifest_renders_exactly_as_v0_1_0_renders_it() {
        let modfile = Modfile {
            module: Some(Module {
                name: "my-dotfiles".to_owned(),
                version: "0.1.0".to_owned(),
            }),
            deps: vec![
                Dep {
                    name: "stdlib".to_owned(),
                    version: "0.2.17".to_owned(),
                    source: String::new(),
                },
                Dep {
                    name: "plug".to_owned(),
                    version: String::new(),
                    source: "github:o/r@v1".to_owned(),
                },
            ],
            replaces: vec![Replace {
                name: "stdlib".to_owned(),
                path: "/local/checkout".to_owned(),
                source: String::new(),
            }],
        };

        assert_eq!(
            modfile.render(),
            concat!(
                "# meowctl.mod — machine-managed module file. Do not edit by hand.\n",
                "\n",
                "module(\n",
                "    name = \"my-dotfiles\",\n",
                "    version = \"0.1.0\",\n",
                ")\n",
                "\n",
                "dep(name = \"stdlib\", version = \"0.2.17\")\n",
                "dep(name = \"plug\", source = \"github:o/r@v1\")\n",
                "\n",
                "replace(name = \"stdlib\", path = \"/local/checkout\")\n",
            )
        );
    }

    /// The blank line after the dependency block is only there when there is a
    /// block, which is what `modfile.Write` does.
    #[test]
    fn an_empty_manifest_is_the_header_alone() {
        assert_eq!(Modfile::default().render(), HEADER);
    }

    /// [R-CONFIG-013] keyword order is `name` first, which is what the
    /// regular-expression rewriter in `v0.1.0` requires and what keeps the two
    /// binaries writing the same bytes.
    #[test]
    fn a_dep_writes_its_name_first() {
        let modfile = Modfile {
            deps: vec![Dep {
                name: "a".to_owned(),
                version: "1.0.0".to_owned(),
                source: String::new(),
            }],
            ..Modfile::default()
        };
        assert!(
            modfile
                .render()
                .contains("dep(name = \"a\", version = \"1.0.0\")"),
            "{}",
            modfile.render()
        );
    }

    /// A `replace()` can point at any path a filesystem allows.
    #[test]
    fn a_path_with_a_quote_in_it_is_escaped() {
        assert_eq!(quote("a\"b"), "\"a\\\"b\"");
        assert_eq!(quote("c:\\dir"), "\"c:\\\\dir\"");
    }
}
