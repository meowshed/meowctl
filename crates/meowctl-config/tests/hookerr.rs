//! [R-CONFIG-064]: the flag a failed runtime hook leaves behind.
//!
//! Small, and load-bearing. `meowctl hook shell` runs on every shell spawn
//! and reports nothing, so this file is the only place a user's broken
//! configuration is recorded; see [R-CLI-062].

// `clippy.toml` exempts a function carrying `#[test]`; a helper in a test
// binary is test code by construction.
#![allow(clippy::expect_used)]

use std::path::Path;

use meowctl_config::HookError;
use meowctl_fs::{FileSystem, MemFs};

const FLAG: &str = "/config/.hook-error";

/// A filesystem with the configuration directory already there.
fn configured() -> MemFs {
    let fs = MemFs::new();
    fs.create_dir_all(Path::new("/config"))
        .expect("the directory");
    fs
}

fn flag() -> HookError {
    HookError {
        at: "2026-09-20T18:11:19Z".to_owned(),
        reason: "broken failed in shell: fail: deliberate".to_owned(),
    }
}

/// [R-CONFIG-064] the timestamp is the first line and the failure the second.
#[test]
fn the_timestamp_leads_and_the_reason_follows() {
    let fs = configured();
    flag().write(&fs, Path::new(FLAG)).expect("the write");

    let text = String::from_utf8(fs.read(Path::new(FLAG)).expect("the read")).expect("utf-8");
    assert_eq!(
        text,
        "2026-09-20T18:11:19Z\nbroken failed in shell: fail: deliberate\n"
    );
}

/// [R-CONFIG-064] what was written reads back, so `doctor` renders what
/// `hook` recorded rather than an approximation of it.
#[test]
fn what_was_written_reads_back() {
    let fs = configured();
    flag().write(&fs, Path::new(FLAG)).expect("the write");

    assert_eq!(HookError::read(&fs, Path::new(FLAG)), Some(flag()));
}

/// [R-CONFIG-064] a reason spanning several lines survives, because a
/// Starlark traceback is the usual case.
#[test]
fn a_reason_of_several_lines_survives() {
    let fs = configured();
    let long = HookError {
        at: "2026-09-20T18:11:19Z".to_owned(),
        reason: "Traceback:\n  File <builtin>\n  * broken:2, in shell".to_owned(),
    };
    long.write(&fs, Path::new(FLAG)).expect("the write");

    assert_eq!(HookError::read(&fs, Path::new(FLAG)), Some(long));
}

/// [R-CONFIG-064] each failure replaces the last. A log would grow on every
/// shell spawn, and what a user needs is the reason it is broken now.
#[test]
fn a_second_failure_replaces_the_first() {
    let fs = configured();
    flag().write(&fs, Path::new(FLAG)).expect("the first");
    let second = HookError {
        at: "2026-09-21T09:00:00Z".to_owned(),
        reason: "something else".to_owned(),
    };
    second.write(&fs, Path::new(FLAG)).expect("the second");

    assert_eq!(HookError::read(&fs, Path::new(FLAG)), Some(second));
}

/// [R-CLI-063] a run in which nothing failed removes it.
#[test]
fn clearing_removes_the_flag() {
    let fs = configured();
    flag().write(&fs, Path::new(FLAG)).expect("the write");
    assert!(HookError::present(&fs, Path::new(FLAG)));

    HookError::clear(&fs, Path::new(FLAG)).expect("the clear");
    assert!(!HookError::present(&fs, Path::new(FLAG)));
    assert_eq!(HookError::read(&fs, Path::new(FLAG)), None);
}

/// [R-CLI-063] absence is the success case, so clearing one that is not
/// there succeeds. Every shell spawn that works takes this path.
#[test]
fn clearing_a_flag_that_is_not_there_succeeds() {
    let fs = configured();
    HookError::clear(&fs, Path::new(FLAG)).expect("the clear");
}
