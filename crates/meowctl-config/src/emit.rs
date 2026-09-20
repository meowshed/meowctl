//! A TOML writer that matches what `v0.1.0` emits, byte for byte.
//!
//! The Rust `toml` crate writes valid TOML that looks nothing like
//! `BurntSushi/toml`'s: no indentation, different key ordering, different
//! handling of empty tables. Both are correct TOML and neither is wrong, but
//! two binaries share a configuration directory during the rewrite, and a lock
//! file that differs only in whitespace still shows up as a change in every
//! `git diff` a user takes of their dotfiles; see [R-CONFIG-023].
//!
//! So the layout is reproduced rather than approximated. What `v0.1.0` does,
//! read from a file it wrote:
//!
//! - two spaces of indentation per level of table nesting;
//! - a blank line before each top-level table, and none before a nested one;
//! - map keys sorted, struct fields in declaration order;
//! - a key quoted only when it contains something other than a letter, a
//!   digit, a hyphen or an underscore;
//! - an empty map emitted as nothing at all, not as an empty table.
//!
//! This writer covers the shapes these schemas use and no more. A general
//! emitter that matched another library everywhere would be a much larger
//! commitment for no more benefit.

use std::fmt::Write as _;

/// A value in the subset these files use.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// A string, quoted and escaped.
    String(String),
    /// An integer.
    Integer(i64),
    /// A boolean.
    Boolean(bool),
    /// An RFC 3339 instant, written unquoted as TOML spells a datetime.
    Datetime(String),
    /// A table, whose entries keep the order they were inserted in.
    Table(Vec<(String, Value)>),
    /// An array of tables, written as `[[name]]` blocks.
    ArrayOfTables(Vec<Value>),
}

impl Value {
    /// A table from pairs, dropping any whose value is `None`.
    ///
    /// The `None` case is how `omitempty` is expressed: a field that is absent
    /// rather than empty.
    #[must_use]
    pub fn table(pairs: Vec<(&str, Option<Value>)>) -> Value {
        Value::Table(
            pairs
                .into_iter()
                .filter_map(|(k, v)| v.map(|v| (k.to_owned(), v)))
                .collect(),
        )
    }

    /// A string value, or `None` when it is empty.
    #[must_use]
    pub fn optional_string(s: &str) -> Option<Value> {
        (!s.is_empty()).then(|| Value::String(s.to_owned()))
    }

    /// Whether this is a table with no entries, which is emitted as nothing.
    fn is_empty_table(&self) -> bool {
        match self {
            Value::Table(entries) => entries.is_empty(),
            Value::ArrayOfTables(items) => items.is_empty(),
            _ => false,
        }
    }

    /// Whether this value lives on a `key = value` line rather than in a table
    /// of its own.
    fn is_inline(&self) -> bool {
        !matches!(self, Value::Table(_) | Value::ArrayOfTables(_))
    }
}

/// Renders a top-level table.
#[must_use]
pub fn to_string(root: &Value) -> String {
    let mut out = String::new();
    let Value::Table(entries) = root else {
        return out;
    };

    // Bare keys first, then tables, which is what a struct with scalar fields
    // followed by map fields produces.
    for (key, value) in entries.iter().filter(|(_, v)| v.is_inline()) {
        let _ = writeln!(out, "{} = {}", render_key(key), render_scalar(value));
    }

    for (key, value) in entries.iter().filter(|(_, v)| !v.is_inline()) {
        if value.is_empty_table() {
            continue;
        }
        if !out.is_empty() {
            out.push('\n');
        }
        write_table(&mut out, std::slice::from_ref(key), value, 0);
    }
    out
}

/// Writes one table and everything under it.
fn write_table(out: &mut String, path: &[String], value: &Value, depth: usize) {
    let indent = "  ".repeat(depth);
    let header = path
        .iter()
        .map(|k| render_key(k))
        .collect::<Vec<_>>()
        .join(".");

    match value {
        Value::Table(entries) => {
            let _ = writeln!(out, "{indent}[{header}]");
            for (key, child) in entries.iter().filter(|(_, v)| v.is_inline()) {
                let _ = writeln!(
                    out,
                    "{}{} = {}",
                    "  ".repeat(depth + 1),
                    render_key(key),
                    render_scalar(child)
                );
            }
            for (key, child) in entries.iter().filter(|(_, v)| !v.is_inline()) {
                if child.is_empty_table() {
                    continue;
                }
                let mut child_path = path.to_vec();
                child_path.push(key.clone());
                write_table(out, &child_path, child, depth + 1);
            }
        }
        Value::ArrayOfTables(items) => {
            for item in items {
                let _ = writeln!(out, "{indent}[[{header}]]");
                if let Value::Table(entries) = item {
                    for (key, child) in entries.iter().filter(|(_, v)| v.is_inline()) {
                        let _ = writeln!(
                            out,
                            "{}{} = {}",
                            "  ".repeat(depth + 1),
                            render_key(key),
                            render_scalar(child)
                        );
                    }
                }
            }
        }
        _ => {}
    }
}

/// A key, quoted when it needs to be.
fn render_key(key: &str) -> String {
    let bare = !key.is_empty()
        && key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if bare {
        key.to_owned()
    } else {
        format!("\"{}\"", escape(key))
    }
}

/// A scalar, as TOML spells it.
fn render_scalar(value: &Value) -> String {
    match value {
        Value::String(s) => format!("\"{}\"", escape(s)),
        Value::Integer(n) => n.to_string(),
        Value::Boolean(b) => b.to_string(),
        // Unquoted: a TOML datetime is its own type, and quoting it would make
        // it a string that a reader has to parse again.
        Value::Datetime(s) => s.clone(),
        Value::Table(_) | Value::ArrayOfTables(_) => String::new(),
    }
}

/// Escapes what a TOML basic string must escape.
fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04X}", c as u32);
            }
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bare_key_is_not_quoted_and_a_path_is() {
        assert_eq!(render_key("generated-by"), "generated-by");
        assert_eq!(render_key("schema_version"), "schema_version");
        assert_eq!(render_key("github.com/o/r"), "\"github.com/o/r\"");
        assert_eq!(
            render_key("components/zsh/init.star"),
            "\"components/zsh/init.star\""
        );
    }

    /// A datetime is its own TOML type, and quoting it would make a reader
    /// parse a string back into one.
    #[test]
    fn a_datetime_is_not_quoted() {
        assert_eq!(
            render_scalar(&Value::Datetime("2026-09-19T14:38:45.972634Z".to_owned())),
            "2026-09-19T14:38:45.972634Z"
        );
    }

    /// An empty map is nothing, not an empty table: `v0.1.0` omits it, and a
    /// stray `[packages]` header would be a difference in every lock file.
    #[test]
    fn an_empty_table_is_omitted() {
        let root = Value::Table(vec![
            (
                "kept".to_owned(),
                Value::Table(vec![("a".to_owned(), Value::Integer(1))]),
            ),
            ("dropped".to_owned(), Value::Table(vec![])),
        ]);
        let rendered = to_string(&root);
        assert!(rendered.contains("[kept]"), "{rendered}");
        assert!(!rendered.contains("dropped"), "{rendered}");
    }

    #[test]
    fn nesting_indents_by_two_spaces_per_level() {
        let root = Value::Table(vec![(
            "modules".to_owned(),
            Value::Table(vec![(
                "stdlib".to_owned(),
                Value::Table(vec![
                    ("version".to_owned(), Value::String("1.0".to_owned())),
                    (
                        "files".to_owned(),
                        Value::Table(vec![("a.star".to_owned(), Value::String("h".to_owned()))]),
                    ),
                ]),
            )]),
        )]);

        assert_eq!(
            to_string(&root),
            "[modules]\n  [modules.stdlib]\n    version = \"1.0\"\n    [modules.stdlib.files]\n      \"a.star\" = \"h\"\n"
        );
    }

    #[test]
    fn a_string_escapes_what_toml_requires() {
        assert_eq!(escape("a\"b\\c\nd"), "a\\\"b\\\\c\\nd");
    }
}
