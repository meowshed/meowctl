//! Every component in the real standard library evaluates.
//!
//! This is the check M0 could not do, because it needed the builtin set. A
//! synthetic configuration exercises what its author thought of; 233 files
//! written by somebody else exercise what they actually wrote, and it is what
//! found that `json` had to be a module rather than a function.
//!
//! Skipped when the repositories are not beside this one, so a machine that
//! has only this checkout still runs the suite. The compat corpus is where
//! this becomes a gate.

// `clippy.toml` exempts tests from `expect_used`, but only a function carrying
// `#[test]`. A helper in a test binary is test code by construction.
#![allow(clippy::expect_used)]
// A skipped test says why on stderr, which is where a test runner shows it.
// The print lint exists to protect the live terminal region; a test binary has
// none.
#![allow(clippy::print_stderr)]

use std::path::{Path, PathBuf};

use meowctl_starlark::{Evaluator, NoLoader, Platform};

/// A sibling checkout, when there is one.
fn sibling(name: &str) -> Option<PathBuf> {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace = here.parent()?.parent()?.parent()?;
    let path = workspace.join(name);
    path.is_dir().then_some(path)
}

/// Evaluates every `.star` file under a directory, returning what failed.
fn evaluate_tree(root: &Path) -> (usize, Vec<(PathBuf, String)>) {
    let loader = NoLoader;
    let evaluator = Evaluator::new(
        Platform {
            os: "macos".to_owned(),
            ..Platform::default()
        },
        &loader,
    );

    let mut evaluated = 0usize;
    let mut failed = Vec::new();

    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().is_none_or(|e| e != "star") {
                continue;
            }
            let source = std::fs::read_to_string(&path).expect("reading a component");
            match evaluator.evaluate(&path.display().to_string(), &source) {
                Ok(_) => evaluated += 1,
                Err(e) => failed.push((path, e.to_string())),
            }
        }
    }
    (evaluated, failed)
}

/// [R-STAR-001] the predeclared set has to be enough for what people have
/// already written.
#[test]
fn every_standard_library_component_evaluates() {
    let Some(stdlib) = sibling("meowctl-stdlib") else {
        eprintln!("skipping: meowctl-stdlib is not beside this checkout");
        return;
    };

    let (evaluated, failed) = evaluate_tree(&stdlib);
    assert!(
        failed.is_empty(),
        "{} of {} files failed:\n{}",
        failed.len(),
        evaluated + failed.len(),
        failed
            .iter()
            .take(5)
            .map(|(p, e)| format!("  {}\n    {}", p.display(), e.lines().next().unwrap_or("")))
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert!(
        evaluated > 100,
        "expected a full standard library, got {evaluated}"
    );
}

/// A real configuration, which uses the builtins differently from a library of
/// components.
#[test]
fn a_real_configuration_evaluates() {
    let Some(dotmeow) = sibling("dotmeow") else {
        eprintln!("skipping: dotmeow is not beside this checkout");
        return;
    };

    let (evaluated, failed) = evaluate_tree(&dotmeow);
    assert!(
        failed.is_empty(),
        "{} files failed:\n{}",
        failed.len(),
        failed
            .iter()
            .take(5)
            .map(|(p, e)| format!("  {}\n    {}", p.display(), e.lines().next().unwrap_or("")))
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert!(
        evaluated > 5,
        "expected a real configuration, got {evaluated}"
    );
}
