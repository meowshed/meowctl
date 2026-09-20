//! The journal, and what it has to stay compatible with.
//!
//! Two binaries share a configuration directory during the rewrite, so a
//! journal written by one has to replay under the other. The field names here
//! are read from `internal/rollback/rollback.go`.

// `clippy.toml` exempts tests from `expect_used`, but only a function carrying
// `#[test]`. A helper in a test binary is test code by construction.
#![allow(clippy::expect_used)]

use std::path::{Path, PathBuf};

use meowctl_exec::ScriptedExecutor;
use meowctl_fs::{FileSystem, MemFs};
use meowctl_ops::{Journal, Op, Outcome, inverse_from, replay};

fn journal_at(dir: &tempfile::TempDir) -> Journal {
    Journal::open(dir.path().join("rollback.jsonl")).expect("open")
}

fn seeded() -> MemFs {
    let fs = MemFs::new();
    fs.create_dir_all(Path::new("/home/u")).expect("home");
    fs.write(Path::new("/home/u/.zshrc"), b"original\n")
        .expect("seed");
    fs
}

/// [R-OPS-020] a record written here is a record `v0.1.0` can read, so the
/// keys are the ones its `inverse*` structs declare.
#[test]
fn a_record_uses_the_field_names_v0_1_0_declares() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut journal = journal_at(&dir);

    journal
        .append(
            "install",
            "neovim",
            &Op::WriteFile {
                path: PathBuf::from("/home/u/.zshrc"),
                contents: b"new\n".to_vec(),
            },
            &Op::WriteFile {
                path: PathBuf::from("/home/u/.zshrc"),
                contents: b"original\n".to_vec(),
            },
        )
        .expect("append");

    let text = std::fs::read_to_string(dir.path().join("rollback.jsonl")).expect("read");
    let record: serde_json::Value = serde_json::from_str(text.trim()).expect("parse");

    assert_eq!(record["seq"], 1);
    assert_eq!(record["phase"], "install");
    assert_eq!(record["component"], "neovim");
    assert_eq!(record["kind"], "write_file");
    assert_eq!(record["inverse"]["path"], "/home/u/.zshrc");
    assert_eq!(record["inverse"]["prior_content"], "original\n");
    assert_eq!(record["inverse"]["had_prior"], true);
}

/// A journal `v0.1.0` wrote replays here, which is the half of compatibility
/// that a round trip through our own writer cannot prove.
#[test]
fn a_v0_1_0_journal_replays() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("rollback.jsonl");

    // Written by hand in the shape `internal/rollback/rollback.go` emits,
    // including the fields this implementation does not write.
    std::fs::write(
        &path,
        concat!(
            r#"{"seq":1,"phase":"install","component":"zsh","kind":"write_file","inverse":{"path":"/home/u/.zshrc","prior_content":"original\n","had_prior":true}}"#,
            "\n",
            r#"{"seq":2,"phase":"install","component":"zsh","kind":"mkdir","inverse":{"path":"/home/u/.config","created_by_meowctl":true}}"#,
            "\n",
            r#"{"seq":3,"phase":"install","component":"zsh","kind":"symlink","inverse":{"dst":"/home/u/link","had_prior":false}}"#,
            "\n",
        ),
    )
    .expect("write");

    let journal = Journal::open(&path).expect("open");
    assert_eq!(journal.len(), 3);
    assert!(journal.is_pending());

    let (records, broken) = journal.records().expect("records");
    assert!(broken.is_empty(), "{broken:?}");
    assert_eq!(records.len(), 3);

    assert!(matches!(
        inverse_from(&records[0]).expect("inverse"),
        Op::WriteFile { .. }
    ));
    assert!(matches!(
        inverse_from(&records[1]).expect("inverse"),
        Op::RemoveDir { .. }
    ));
    assert!(matches!(
        inverse_from(&records[2]).expect("inverse"),
        Op::Remove { .. }
    ));
}

/// [R-OPS-022] undoing a mkdir before the writes inside it would fail, so the
/// order is the reverse of the order things happened.
#[test]
fn a_replay_runs_backwards() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut journal = journal_at(&dir);
    let fs = seeded();
    let exec = ScriptedExecutor::new([]);
    let mut events = |_| {};

    let mkdir = Op::Mkdir {
        path: PathBuf::from("/home/u/.config/nvim"),
    };
    let write = Op::WriteFile {
        path: PathBuf::from("/home/u/.config/nvim/init.lua"),
        contents: b"-- hi\n".to_vec(),
    };

    for op in [&mkdir, &write] {
        let inverse = op.inverse(&fs, &exec).expect("inverse");
        journal
            .append("install", "nvim", op, &inverse)
            .expect("append");
        op.apply(&fs, &exec, &mut events).expect("apply");
    }

    let before = seeded().snapshot();
    let outcome = replay(&journal, &fs, &exec, &mut events).expect("replay");

    assert_eq!(outcome.outcome, Outcome::Ok, "{:?}", outcome.failures);
    assert_eq!(outcome.applied, 2);
    assert_eq!(fs.snapshot(), before, "the replay did not restore the tree");
}

/// [R-OPS-030] one corrupt line must not strand every earlier operation.
#[test]
fn a_corrupt_line_is_reported_and_the_rest_still_replay() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("rollback.jsonl");
    std::fs::write(
        &path,
        concat!(
            r#"{"seq":1,"phase":"install","component":"a","kind":"write_file","inverse":{"path":"/home/u/.zshrc","prior_content":"original\n","had_prior":true}}"#,
            "\n",
            "{ this is not json\n",
            r#"{"seq":3,"phase":"install","component":"c","kind":"symlink","inverse":{"dst":"/home/u/link","had_prior":false}}"#,
            "\n",
        ),
    )
    .expect("write");

    let journal = Journal::open(&path).expect("open");
    let (records, broken) = journal.records().expect("records");
    assert_eq!(records.len(), 2, "the good records were lost");
    assert_eq!(broken.len(), 1);
    assert!(
        broken[0].to_string().contains("unreadable"),
        "{}",
        broken[0]
    );
}

/// [R-OPS-024] `partial` is the outcome a user most needs to see, and
/// [R-OPS-023]: the inverse that failed did not strand the one after it. The
/// failing record is the later one, and replay runs backwards, so the run
/// reached the good one only by continuing past the bad.
#[test]
fn a_replay_that_half_works_reports_partial() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("rollback.jsonl");
    std::fs::write(
        &path,
        concat!(
            r#"{"seq":1,"phase":"install","component":"a","kind":"symlink","inverse":{"dst":"/home/u/link","had_prior":false}}"#,
            "\n",
            r#"{"seq":2,"phase":"install","component":"b","kind":"nonsense","inverse":{}}"#,
            "\n",
        ),
    )
    .expect("write");

    let journal = Journal::open(&path).expect("open");
    let fs = seeded();
    let exec = ScriptedExecutor::new([]);
    let mut events = |_| {};

    let outcome = replay(&journal, &fs, &exec, &mut events).expect("replay");
    assert_eq!(outcome.outcome, Outcome::Partial);
    assert_eq!(outcome.applied, 1);
    assert_eq!(outcome.failures.len(), 1);
}

/// [R-OPS-025] every run would otherwise replay the last one's operations.
#[test]
fn truncating_empties_the_journal() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut journal = journal_at(&dir);

    journal
        .append(
            "install",
            "a",
            &Op::Mkdir {
                path: PathBuf::from("/tmp/x"),
            },
            &Op::RemoveDir {
                path: PathBuf::from("/tmp/x"),
                until: PathBuf::from("/tmp"),
            },
        )
        .expect("append");
    assert!(journal.is_pending());

    journal.truncate().expect("truncate");
    assert!(!journal.is_pending());
    assert!(journal.is_empty());
}

/// An operation that is only ever an inverse never reaches the journal: a
/// journal records the forward operation and how to undo it, not the undo.
#[test]
fn an_inverse_only_operation_is_not_journaled() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut journal = journal_at(&dir);

    journal
        .append(
            "install",
            "a",
            &Op::Remove {
                path: PathBuf::from("/tmp/x"),
            },
            &Op::Nothing,
        )
        .expect("append");

    assert!(journal.is_empty());
}

/// [R-OPS-021] and [R-OPS-003]: a crash between the record and the effect
/// must leave a journal
/// that replays a no-op, so the record is on disk before the effect runs.
#[test]
fn a_record_is_on_disk_before_its_operation_runs() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("rollback.jsonl");
    let mut journal = Journal::open(&path).expect("open");

    journal
        .append(
            "install",
            "a",
            &Op::WriteFile {
                path: PathBuf::from("/home/u/.zshrc"),
                contents: b"new\n".to_vec(),
            },
            &Op::Remove {
                path: PathBuf::from("/home/u/.zshrc"),
            },
        )
        .expect("append");

    // Read from a fresh handle: the bytes are on disk, not in a buffer this
    // process happens to hold.
    let reopened = Journal::open(&path).expect("reopen");
    assert_eq!(reopened.len(), 1);
}
