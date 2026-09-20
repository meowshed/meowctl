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

/// Journals one operation and returns the record it wrote.
fn record_for(forward: &Op, inverse: &Op) -> serde_json::Value {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut journal = journal_at(&dir);
    journal
        .append("install", "c", forward, inverse)
        .expect("append");
    let text = std::fs::read_to_string(dir.path().join("rollback.jsonl")).expect("read");
    serde_json::from_str(text.trim()).expect("parse")
}

/// [R-OPS-020] every journalled variant, not just the first one.
///
/// The record is what a *different binary* replays, so each variant's payload
/// is an interface. Mutation testing is what found this: the arms that carry
/// the prior value -- the content, the target, the backup, the setting --
/// could each be deleted and only `write_file` had a test that noticed.
#[test]
fn every_journalled_variant_writes_the_payload_v0_1_0_reads() {
    let path = PathBuf::from("/home/u/.zshrc");
    let other = PathBuf::from("/home/u/source");

    // A download that replaced a file carries what was there.
    let download = record_for(
        &Op::Download {
            path: path.clone(),
            contents: b"fetched\n".to_vec(),
        },
        &Op::WriteFile {
            path: path.clone(),
            contents: b"original\n".to_vec(),
        },
    );
    assert_eq!(download["kind"], "download");
    assert_eq!(download["inverse"]["dst"], "/home/u/.zshrc");
    assert_eq!(download["inverse"]["prior_content"], "original\n");
    assert_eq!(download["inverse"]["had_prior"], true);

    // And one that created a file says so, or the replay would write an
    // empty file where there was none.
    let fresh = record_for(
        &Op::Download {
            path: path.clone(),
            contents: b"fetched\n".to_vec(),
        },
        &Op::Remove { path: path.clone() },
    );
    assert_eq!(fresh["inverse"]["had_prior"], false);

    // A symlink that re-pointed an existing one carries the old target.
    let symlink = record_for(
        &Op::Symlink {
            target: other.clone(),
            link: path.clone(),
        },
        &Op::Symlink {
            target: PathBuf::from("/home/u/was-here"),
            link: path.clone(),
        },
    );
    assert_eq!(symlink["kind"], "symlink");
    assert_eq!(symlink["inverse"]["dst"], "/home/u/.zshrc");
    assert_eq!(symlink["inverse"]["prior_target"], "/home/u/was-here");
    assert_eq!(symlink["inverse"]["had_prior"], true);

    // A link_file that moved the user's file aside carries where it went.
    let backup = PathBuf::from("/home/u/.zshrc.backup");
    let linked = record_for(
        &Op::LinkFile {
            target: other.clone(),
            link: path.clone(),
            backup: backup.clone(),
        },
        &Op::RestoreBackup {
            backup: backup.clone(),
            link: path.clone(),
        },
    );
    assert_eq!(linked["kind"], "link_file");
    assert_eq!(linked["inverse"]["backup_path"], "/home/u/.zshrc.backup");
    assert_eq!(linked["inverse"]["was_backed_up"], true);

    // A defaults write carries the value to put back, and its type: `-bool`
    // and `-int` are different settings with the same string.
    let defaults = record_for(
        &Op::DefaultsWrite {
            domain: "com.apple.dock".to_owned(),
            key: "autohide".to_owned(),
            value: "1".to_owned(),
            value_type: "-bool".to_owned(),
        },
        &Op::DefaultsWrite {
            domain: "com.apple.dock".to_owned(),
            key: "autohide".to_owned(),
            value: "0".to_owned(),
            value_type: "-bool".to_owned(),
        },
    );
    assert_eq!(defaults["kind"], "defaults_write");
    assert_eq!(defaults["inverse"]["value"], "0");
    assert_eq!(defaults["inverse"]["value_type"], "-bool");

    // And a plist set carries the value alone.
    let plist = record_for(
        &Op::PlistSet {
            path: PathBuf::from("/home/u/Library/Preferences/x.plist"),
            key: "Enabled".to_owned(),
            value: "true".to_owned(),
        },
        &Op::PlistSet {
            path: PathBuf::from("/home/u/Library/Preferences/x.plist"),
            key: "Enabled".to_owned(),
            value: "false".to_owned(),
        },
    );
    assert_eq!(plist["kind"], "plist_set");
    assert_eq!(plist["inverse"]["key"], "Enabled");
    assert_eq!(plist["inverse"]["value"], "false");
}

/// [R-OPS-024] nothing applied is `failed`, not `partial`.
///
/// The two are what a user reads to decide whether to finish by hand, and
/// `partial` where nothing was undone sends them looking for work that was
/// never done. Mutation testing found this: `applied > 0` could become
/// `applied >= 0` and every test still passed.
#[test]
fn a_replay_where_nothing_applied_reports_failed() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("rollback.jsonl");
    std::fs::write(
        &path,
        concat!(
            r#"{"seq":1,"phase":"install","component":"a","kind":"nonsense","inverse":{}}"#,
            "\n",
        ),
    )
    .expect("write");

    let journal = Journal::open(&path).expect("open");
    let fs = seeded();
    let exec = ScriptedExecutor::new([]);
    let mut events = |_| {};

    let outcome = replay(&journal, &fs, &exec, &mut events).expect("replay");
    assert_eq!(outcome.outcome, Outcome::Failed);
    assert_eq!(outcome.applied, 0);
    assert_eq!(outcome.failures.len(), 1);
}

/// [R-OPS-025] truncating a journal that is not there succeeds and resets the
/// sequence, because a first run has no journal and still has to be able to
/// finish.
#[test]
fn truncating_a_journal_that_was_never_written_succeeds() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut journal = journal_at(&dir);

    assert!(journal.is_empty(), "a fresh journal holds nothing");
    journal.truncate().expect("truncating nothing succeeds");
    assert!(journal.is_empty());
}

/// [R-OPS-030] the sequence number in an unreadable-record error is the line
/// it was on, because that is what the reader opens the file to find.
#[test]
fn an_unreadable_record_is_numbered_by_its_line() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("rollback.jsonl");
    std::fs::write(
        &path,
        concat!(
            r#"{"seq":1,"phase":"install","component":"a","kind":"symlink","inverse":{"dst":"/home/u/link","had_prior":false}}"#,
            "\n",
            "{ not json\n",
            "{ also not json\n",
        ),
    )
    .expect("write");

    let journal = Journal::open(&path).expect("open");
    let (_, broken) = journal.records().expect("records");
    assert_eq!(broken.len(), 2);
    assert!(broken[0].to_string().contains('2'), "{}", broken[0]);
    assert!(broken[1].to_string().contains('3'), "{}", broken[1]);
}

/// [R-OPS-025] a journal holds something once something is appended, which is
/// what `interrupted_run` reads to report a run that stopped partway.
#[test]
fn a_journal_holds_something_once_something_is_appended() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut journal = journal_at(&dir);
    assert!(journal.is_empty());

    journal
        .append(
            "install",
            "c",
            &Op::WriteFile {
                path: PathBuf::from("/home/u/.zshrc"),
                contents: b"x".to_vec(),
            },
            &Op::Remove {
                path: PathBuf::from("/home/u/.zshrc"),
            },
        )
        .expect("append");

    assert!(!journal.is_empty(), "the journal still reports nothing");
    journal.truncate().expect("truncate");
    assert!(journal.is_empty(), "truncating did not reset the sequence");
}

/// [R-OPS-025] a journal file that was never written reads as empty rather
/// than failing, because a first run has none and still has to start.
#[test]
fn a_journal_that_is_not_there_reads_as_empty() {
    let dir = tempfile::tempdir().expect("tempdir");
    let journal = Journal::open(dir.path().join("never-written.jsonl")).expect("open");

    let (records, broken) = journal.records().expect("records");
    assert!(records.is_empty(), "{records:?}");
    assert!(broken.is_empty(), "{broken:?}");
}

/// [R-OPS-020] the kind strings are what a `v0.1.0` journal carries, so a
/// record written here replays there and one written there replays here.
#[test]
fn the_kind_strings_are_the_ones_v0_1_0_writes() {
    use meowctl_ops::OpKind;

    for (kind, written) in [
        (OpKind::WriteFile, "write_file"),
        (OpKind::AppendFile, "append_file"),
        (OpKind::CopyFile, "copy_file"),
        (OpKind::Symlink, "symlink"),
        (OpKind::LinkFile, "link_file"),
        (OpKind::Mkdir, "mkdir"),
        (OpKind::Download, "download"),
        (OpKind::DefaultsWrite, "defaults_write"),
        (OpKind::PlistSet, "plist_set"),
    ] {
        assert_eq!(kind.as_str(), written);
        assert_eq!(String::from(kind), written);
        assert_eq!(kind.to_string(), written);
    }
}

/// [R-OPS-024] and [R-CONFIG-044]: the outcome strings are what `state.toml`
/// records, so a user reading the file and a user reading the screen see the
/// same word.
#[test]
fn the_outcome_strings_are_the_ones_the_sentinel_records() {
    assert_eq!(Outcome::Ok.as_str(), "ok");
    assert_eq!(Outcome::Partial.as_str(), "partial");
    assert_eq!(Outcome::Failed.as_str(), "failed");
}
