//! Changing a Starlark file without disturbing the rest of it.
//!
//! `meowctl add`, `meowctl remove` and `meowctl dep upgrade` rewrite files
//! that belong to the user: their comments, their grouping, their blank lines.
//! A round trip through a parsed model and a printer would lose all of it, so
//! the edit is made to the *text*, at the position the parse reports; see
//! [R-CONFIG-050].
//!
//! `v0.1.0` uses regular expressions, and documents what that costs: a
//! `dep()` written `version` before `name` is not matched, so
//! `meowctl dep upgrade` reports "not found" for a declaration that is plainly
//! there. Parsing has no such condition; see [R-CONFIG-051].
//!
//! Parsing is not evaluating. This module reads the syntax to find a
//! statement; what a declaration *means* is [`meowctl_starlark`]'s, and the
//! dialect is shared with it — see [`DIALECT`].
//!
//! [`meowctl_starlark`]: https://docs.rs/meowctl-starlark

use starlark_syntax::syntax::ast::{ArgumentP, AstStmt, ExprP, StmtP};
use starlark_syntax::syntax::module::AstModule;
use starlark_syntax::syntax::top_level_stmts::top_level_stmts;

use crate::ConfigError;

/// The dialect both this module and the evaluator parse with.
///
/// Named in two crates on purpose. A file one accepts and the other rejects
/// would mean `meowctl add` succeeding on a configuration that then fails to
/// apply, and a test runs one file through both to keep them honest. The
/// duplication is one line; the alternative is an edge between two crates that
/// otherwise share nothing.
#[must_use]
pub fn dialect() -> starlark_syntax::dialect::Dialect {
    starlark_syntax::dialect::Dialect::Extended
}

/// A call statement at the top level of a file, with where it sits.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Call {
    /// The function.
    name: String,
    /// The first positional argument, when it is a string literal.
    first_positional: Option<String>,
    /// Named string arguments.
    named: Vec<(String, String)>,
    /// Where the statement begins, as a byte offset.
    begin: usize,
    /// Where it ends, as a byte offset.
    end: usize,
}

/// Parses a file and returns its top-level calls.
fn calls(name: &str, source: &str) -> Result<Vec<Call>, ConfigError> {
    let ast = AstModule::parse(name, source.to_owned(), &dialect()).map_err(|e| {
        ConfigError::Malformed {
            path: name.into(),
            reason: e.to_string(),
        }
    })?;

    let mut found = Vec::new();
    for statement in top_level_stmts(ast.statement()) {
        let Some(call) = as_call(statement) else {
            continue;
        };
        found.push(call);
    }
    Ok(found)
}

/// Reads one statement as a call, when that is what it is.
fn as_call(statement: &AstStmt) -> Option<Call> {
    let StmtP::Expression(expr) = &statement.node else {
        return None;
    };
    let ExprP::Call(function, arguments) = &expr.node else {
        return None;
    };
    let ExprP::Identifier(name) = &function.node else {
        return None;
    };

    let mut first_positional = None;
    let mut named = Vec::new();
    for argument in &arguments.args {
        match &argument.node {
            ArgumentP::Positional(value) => {
                if first_positional.is_none() {
                    first_positional = string_of(&value.node);
                }
            }
            ArgumentP::Named(key, value) => {
                if let Some(text) = string_of(&value.node) {
                    named.push((key.node.clone(), text));
                }
            }
            ArgumentP::Args(_) | ArgumentP::KwArgs(_) => {}
        }
    }

    Some(Call {
        name: name.node.ident.clone(),
        first_positional,
        named,
        begin: statement.span.begin().get() as usize,
        end: statement.span.end().get() as usize,
    })
}

/// The value of a string literal, when the expression is one.
fn string_of<P>(expr: &ExprP<P>) -> Option<String>
where
    P: starlark_syntax::syntax::ast::AstPayload,
{
    match expr {
        ExprP::Literal(starlark_syntax::syntax::ast::AstLiteral::String(s)) => Some(s.node.clone()),
        _ => None,
    }
}

/// Adds a `component("name")` declaration to a file.
///
/// Appended, because there is no position that is more right than the end and
/// a user who groups their components can move it. A declaration that is
/// already there is left alone rather than duplicated, which is what
/// `AppendComponent` in `internal/rewrite/rewrite.go` does not check.
///
/// # Errors
///
/// [`ConfigError::Malformed`] when the file does not parse.
pub fn add_component(file: &str, source: &str, name: &str) -> Result<String, ConfigError> {
    if has_component(file, source, name)? {
        return Ok(source.to_owned());
    }

    let mut out = source.to_owned();
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(&format!("component({})\n", quote(name)));
    Ok(out)
}

/// Whether a file already declares this component.
///
/// # Errors
///
/// [`ConfigError::Malformed`] when the file does not parse.
pub fn has_component(file: &str, source: &str, name: &str) -> Result<bool, ConfigError> {
    Ok(calls(file, source)?
        .iter()
        .any(|c| c.name == "component" && component_name(c).as_deref() == Some(name)))
}

/// Removes a `component("name")` declaration.
///
/// The statement's line goes, and nothing else: a comment above it belongs to
/// whoever wrote it, and guessing that it describes the component would delete
/// a section header along with one entry.
///
/// # Errors
///
/// [`ConfigError::Malformed`] when the file does not parse, or
/// [`ConfigError::Missing`] when there is no such declaration — reported
/// rather than written as a no-op; see [R-CONFIG-052].
pub fn remove_component(file: &str, source: &str, name: &str) -> Result<String, ConfigError> {
    let target = calls(file, source)?
        .into_iter()
        .find(|c| c.name == "component" && component_name(c).as_deref() == Some(name))
        .ok_or_else(|| ConfigError::Malformed {
            path: file.into(),
            reason: format!("no component({}) declaration to remove", quote(name)),
        })?;

    Ok(splice_line(source, target.begin, target.end))
}

/// Changes the version in a `dep()` declaration.
///
/// # Errors
///
/// [`ConfigError::Malformed`] when the file does not parse or when no `dep()`
/// names this module.
pub fn set_dep_version(
    file: &str,
    source: &str,
    module: &str,
    version: &str,
) -> Result<String, ConfigError> {
    let target = calls(file, source)?
        .into_iter()
        .find(|c| {
            c.name == "dep"
                && c.named
                    .iter()
                    .any(|(key, value)| key == "name" && value == module)
        })
        .ok_or_else(|| ConfigError::Malformed {
            path: file.into(),
            reason: format!("no dep() declaration for {module} to change"),
        })?;

    // Rendered from what the declaration said rather than from a template, so
    // a `dep()` carrying a source keeps it.
    let mut rendered = format!("dep(name = {}", quote(module));
    for (key, value) in &target.named {
        if key == "name" {
            continue;
        }
        let value = if key == "version" { version } else { value };
        rendered.push_str(&format!(", {key} = {}", quote(value)));
    }
    if !target.named.iter().any(|(key, _)| key == "version") {
        rendered.push_str(&format!(", version = {}", quote(version)));
    }
    rendered.push(')');

    let mut out = String::with_capacity(source.len());
    out.push_str(&source[..target.begin]);
    out.push_str(&rendered);
    out.push_str(&source[target.end..]);
    Ok(out)
}

/// The name a `component()` call declares, positionally or by keyword.
fn component_name(call: &Call) -> Option<String> {
    call.first_positional.clone().or_else(|| {
        call.named
            .iter()
            .find(|(key, _)| key == "name")
            .map(|(_, value)| value.clone())
    })
}

/// Removes the statement between two offsets, and the newline that ended it.
///
/// Taking the trailing newline is what keeps a removal from leaving a blank
/// line where the declaration was.
fn splice_line(source: &str, begin: usize, end: usize) -> String {
    let mut end = end;
    if source[end..].starts_with("\r\n") {
        end += 2;
    } else if source[end..].starts_with('\n') {
        end += 1;
    }

    let mut out = String::with_capacity(source.len());
    out.push_str(&source[..begin]);
    out.push_str(&source[end..]);
    out
}

/// A Starlark string literal.
fn quote(text: &str) -> String {
    let escaped = text.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}
