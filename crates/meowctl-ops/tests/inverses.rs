//! Applying an operation and then its inverse restores what was there.
//!
//! The property test is the one that matters. A per-variant suite checks the
//! variants somebody thought of; the failure this crate exists to prevent is a
//! variant added later whose inverse is subtly wrong, and only a test over the
//! whole set catches that; see [R-OPS-004].

// `clippy.toml` exempts tests from `expect_used`, but only a function carrying
// `#[test]`. A helper in a test binary is test code by construction.
#![allow(clippy::expect_used)]

use std::path::{Path, PathBuf};

use meowctl_exec::ScriptedExecutor;
use meowctl_fs::{FileSystem, MemFs};
use meowctl_ops::Op;
use proptest::prelude::*;

/// A filesystem with a home directory and one file already in it, so the
/// "prior content existed" and "it did not" cases both come up.
fn seeded() -> MemFs {
    let fs = MemFs::new();
    fs.create_dir_all(Path::new("/home/u")).expect("home");
    fs.write(Path::new("/home/u/.zshrc"), b"original\n")
        .expect("seed");
    fs.write(Path::new("/home/u/source"), b"source\n")
        .expect("seed");
    fs
}

fn nothing_runs() -> ScriptedExecutor {
    ScriptedExecutor::new([])
}

/// Applies an operation and its inverse, and returns whether the tree came
/// back to where it started.
fn round_trips(op: &Op) -> Result<(), String> {
    let fs = seeded();
    let exec = nothing_runs();
    let mut events = |_| {};

    let before = fs.snapshot();
    let inverse = op
        .inverse(&fs, &exec)
        .map_err(|e| format!("computing the inverse of {:?}: {e}", op.kind()))?;
    op.apply(&fs, &exec, &mut events)
        .map_err(|e| format!("applying {:?}: {e}", op.kind()))?;
    inverse
        .apply(&fs, &exec, &mut events)
        .map_err(|e| format!("undoing {:?}: {e}", op.kind()))?;

    let after = fs.snapshot();
    if before == after {
        return Ok(());
    }
    Err(format!(
        "{:?} did not restore the tree\n  before: {before:?}\n  after:  {after:?}",
        op.kind()
    ))
}

/// The filesystem operations, with both the "it was there" and "it was not"
/// cases for every one that has them.
fn filesystem_ops() -> impl Strategy<Value = Op> {
    let existing = PathBuf::from("/home/u/.zshrc");
    let fresh = PathBuf::from("/home/u/fresh");
    let source = PathBuf::from("/home/u/source");

    prop_oneof![
        Just(Op::WriteFile {
            path: existing.clone(),
            contents: b"new\n".to_vec()
        }),
        Just(Op::WriteFile {
            path: fresh.clone(),
            contents: b"new\n".to_vec()
        }),
        Just(Op::Download {
            path: existing.clone(),
            contents: b"fetched\n".to_vec()
        }),
        Just(Op::Download {
            path: fresh.clone(),
            contents: b"fetched\n".to_vec()
        }),
        Just(Op::AppendFile {
            path: existing.clone(),
            contents: "added\n".to_owned(),
            marker: "test".to_owned(),
        }),
        Just(Op::AppendFile {
            path: fresh.clone(),
            contents: "added\n".to_owned(),
            marker: "test".to_owned(),
        }),
        Just(Op::CopyFile {
            from: source.clone(),
            to: fresh.clone()
        }),
        Just(Op::Symlink {
            target: source.clone(),
            link: fresh.clone()
        }),
        Just(Op::Mkdir {
            path: PathBuf::from("/home/u/.config/nvim")
        }),
        Just(Op::Mkdir {
            path: PathBuf::from("/home/u")
        }),
        Just(Op::LinkFile {
            target: source,
            link: existing,
            backup: PathBuf::from("/home/u/.zshrc.backup"),
        }),
    ]
}

proptest! {
    /// [R-OPS-004] every filesystem variant, applied and undone.
    #[test]
    fn apply_then_undo_restores_the_tree(op in filesystem_ops()) {
        if let Err(why) = round_trips(&op) {
            prop_assert!(false, "{why}");
        }
    }
}

/// [R-OPS-010] undoing a write to a file that existed restores it. Always
/// deleting would destroy the user's original.
#[test]
fn undoing_a_write_restores_prior_content() {
    let fs = seeded();
    let exec = nothing_runs();
    let mut events = |_| {};

    let op = Op::WriteFile {
        path: PathBuf::from("/home/u/.zshrc"),
        contents: b"replaced\n".to_vec(),
    };
    let inverse = op.inverse(&fs, &exec).expect("inverse");
    op.apply(&fs, &exec, &mut events).expect("apply");
    assert_eq!(
        fs.read(Path::new("/home/u/.zshrc")).expect("read"),
        b"replaced\n"
    );

    inverse.apply(&fs, &exec, &mut events).expect("undo");
    assert_eq!(
        fs.read(Path::new("/home/u/.zshrc")).expect("read"),
        b"original\n"
    );
}

/// [R-OPS-010] and undoing a write to a file that did not exist removes it.
#[test]
fn undoing_a_write_to_a_new_file_removes_it() {
    let fs = seeded();
    let exec = nothing_runs();
    let mut events = |_| {};

    let op = Op::WriteFile {
        path: PathBuf::from("/home/u/new"),
        contents: b"x".to_vec(),
    };
    let inverse = op.inverse(&fs, &exec).expect("inverse");
    op.apply(&fs, &exec, &mut events).expect("apply");
    inverse.apply(&fs, &exec, &mut events).expect("undo");

    assert!(!fs.exists(Path::new("/home/u/new")).expect("exists"));
}

/// [R-OPS-011] a file edited between apply and undo keeps the edit, which is
/// why the inverse removes a marked block rather than truncating.
#[test]
fn undoing_an_append_keeps_an_edit_made_since() {
    let fs = seeded();
    let exec = nothing_runs();
    let mut events = |_| {};

    let op = Op::AppendFile {
        path: PathBuf::from("/home/u/.zshrc"),
        contents: "export MEOWCTL=1\n".to_owned(),
        marker: "abc".to_owned(),
    };
    let inverse = op.inverse(&fs, &exec).expect("inverse");
    op.apply(&fs, &exec, &mut events).expect("apply");

    // The user edits the file afterwards.
    let edited = format!(
        "{}\n# mine\n",
        String::from_utf8_lossy(&fs.read(Path::new("/home/u/.zshrc")).expect("read")).trim_end()
    );
    fs.write(Path::new("/home/u/.zshrc"), edited.as_bytes())
        .expect("edit");

    inverse.apply(&fs, &exec, &mut events).expect("undo");
    let after = fs.read(Path::new("/home/u/.zshrc")).expect("read");
    let after = String::from_utf8_lossy(&after);

    assert!(after.contains("original"), "{after:?}");
    assert!(
        after.contains("# mine"),
        "the user's edit was lost: {after:?}"
    );
    assert!(!after.contains("MEOWCTL"), "the block survived: {after:?}");
}

/// [R-OPS-011] a component re-running with the same marker replaces its own
/// block rather than appending a second copy.
#[test]
fn appending_twice_with_one_marker_leaves_one_block() {
    let fs = seeded();
    let exec = nothing_runs();
    let mut events = |_| {};

    let op = Op::AppendFile {
        path: PathBuf::from("/home/u/.zshrc"),
        contents: "line\n".to_owned(),
        marker: "same".to_owned(),
    };
    op.apply(&fs, &exec, &mut events).expect("first");
    op.apply(&fs, &exec, &mut events).expect("second");

    let after = fs.read(Path::new("/home/u/.zshrc")).expect("read");
    let after = String::from_utf8_lossy(&after);
    assert_eq!(after.matches("BEGIN meowctl same").count(), 1, "{after:?}");
}

/// [R-OPS-013] removing a directory meowctl did not create would delete
/// `~/.config` because a component put a file in it.
#[test]
fn undoing_a_mkdir_leaves_a_directory_that_already_existed() {
    let fs = seeded();
    let exec = nothing_runs();
    let mut events = |_| {};

    let op = Op::Mkdir {
        path: PathBuf::from("/home/u"),
    };
    let inverse = op.inverse(&fs, &exec).expect("inverse");
    assert_eq!(inverse, Op::Nothing);

    op.apply(&fs, &exec, &mut events).expect("apply");
    inverse.apply(&fs, &exec, &mut events).expect("undo");
    assert!(fs.exists(Path::new("/home/u")).expect("exists"));
}

/// [R-OPS-014] a component that re-points an existing symlink must leave the
/// previous one behind, not nothing.
#[test]
fn undoing_a_symlink_restores_the_previous_target() {
    let fs = seeded();
    let exec = nothing_runs();
    let mut events = |_| {};

    Op::Symlink {
        target: PathBuf::from("/first"),
        link: PathBuf::from("/home/u/link"),
    }
    .apply(&fs, &exec, &mut events)
    .expect("first link");

    let op = Op::Symlink {
        target: PathBuf::from("/second"),
        link: PathBuf::from("/home/u/link"),
    };
    let inverse = op.inverse(&fs, &exec).expect("inverse");
    op.apply(&fs, &exec, &mut events).expect("apply");
    inverse.apply(&fs, &exec, &mut events).expect("undo");

    assert_eq!(
        fs.read_link(Path::new("/home/u/link")).expect("read link"),
        PathBuf::from("/first")
    );
}

/// [R-OPS-015] the backup is the user's original file, and not restoring it is
/// data loss.
#[test]
fn undoing_a_link_restores_the_file_it_moved_aside() {
    let fs = seeded();
    let exec = nothing_runs();
    let mut events = |_| {};

    let op = Op::LinkFile {
        target: PathBuf::from("/home/u/source"),
        link: PathBuf::from("/home/u/.zshrc"),
        backup: PathBuf::from("/home/u/.zshrc.backup"),
    };
    let inverse = op.inverse(&fs, &exec).expect("inverse");
    op.apply(&fs, &exec, &mut events).expect("apply");
    inverse.apply(&fs, &exec, &mut events).expect("undo");

    assert_eq!(
        fs.read(Path::new("/home/u/.zshrc")).expect("read"),
        b"original\n"
    );
}

/// [R-OPS-031] the user who deleted the file already achieved what the inverse
/// wanted.
#[test]
fn an_inverse_whose_target_is_gone_succeeds() {
    let fs = seeded();
    let exec = nothing_runs();
    let mut events = |_| {};

    Op::Remove {
        path: PathBuf::from("/home/u/never-existed"),
    }
    .apply(&fs, &exec, &mut events)
    .expect("remove");
}

/// [R-OPS-017] a failed run must not leave system preferences changed with no
/// record of what they were, which is what `v0.1.0` does by not journaling
/// these at all.
#[test]
fn a_macos_default_records_its_prior_value() {
    let fs = seeded();
    let exec = ScriptedExecutor::new([meowctl_exec::ScriptedRun::ok(
        "defaults read com.apple.dock autohide",
        "0\n",
    )]);

    let op = Op::DefaultsWrite {
        domain: "com.apple.dock".to_owned(),
        key: "autohide".to_owned(),
        value: "1".to_owned(),
        value_type: "-bool".to_owned(),
    };
    let inverse = op.inverse(&fs, &exec).expect("inverse");
    assert_eq!(
        inverse,
        Op::DefaultsWrite {
            domain: "com.apple.dock".to_owned(),
            key: "autohide".to_owned(),
            value: "0".to_owned(),
            value_type: "-bool".to_owned(),
        }
    );
}

/// [R-OPS-017] with no prior value the inverse deletes the key rather than
/// writing an empty one, which would leave the machine in a state it was never
/// in.
#[test]
fn a_macos_default_with_no_prior_value_is_undone_by_deleting_it() {
    let fs = seeded();
    let exec = ScriptedExecutor::new([meowctl_exec::ScriptedRun::fails(
        "defaults read com.apple.dock autohide",
        1,
        "does not exist\n",
    )]);

    let op = Op::DefaultsWrite {
        domain: "com.apple.dock".to_owned(),
        key: "autohide".to_owned(),
        value: "1".to_owned(),
        value_type: "-bool".to_owned(),
    };
    let inverse = op.inverse(&fs, &exec).expect("inverse");
    match inverse {
        Op::DefaultsWrite { value_type, .. } => assert_eq!(value_type, "-delete"),
        other => panic!("expected a delete, got {other:?}"),
    }
}
