//! What a dry run does that the others do not.
//!
//! The conformance suite proves the three implementations agree. These prove
//! the one thing the dry run must do differently: nothing.

// `clippy.toml` exempts tests from `expect_used`, but the exemption only
// recognises a function carrying `#[test]`. A helper in a test binary is test
// code by construction, so the whole file is exempt here rather than each
// helper separately.
#![allow(clippy::expect_used)]

use std::path::Path;

use meowctl_fs::{DryRunFs, FileSystem, Intent, MemFs};

/// A dry run over a filesystem seeded with one file, returning both so a test
/// can assert on what the underlying tree looks like afterwards.
fn dry_run() -> (DryRunFs, std::sync::Arc<MemFs>) {
    let backing = std::sync::Arc::new(MemFs::new());
    backing.seed("/home/u/.zshrc", "original\n");
    backing
        .create_dir_all(Path::new("/home/u"))
        .expect("seeding the home directory");
    (
        DryRunFs::new(Box::new(std::sync::Arc::clone(&backing))),
        backing,
    )
}

/// [R-FS-011] the guarantee the whole design rests on. `v0.1.0` made it by
/// checking a flag in every effectful method and missed one; here it is not
/// expressible.
#[test]
fn nothing_reaches_the_underlying_filesystem() {
    let (dry, backing) = dry_run();
    let before = backing.snapshot();

    dry.write(Path::new("/home/u/new"), b"x").expect("write");
    dry.append(Path::new("/home/u/.zshrc"), b"more\n")
        .expect("append");
    dry.create_dir_all(Path::new("/home/u/.config/nvim"))
        .expect("mkdir");
    dry.symlink(Path::new("/src"), Path::new("/home/u/link"), None)
        .expect("symlink");
    dry.copy(Path::new("/home/u/.zshrc"), Path::new("/home/u/copy"))
        .expect("copy");
    // Last, because the dry run remembers it: a copy after this would fail,
    // which is the behaviour a_removed_path_stays_removed_for_the_rest_of_the_run
    // checks on purpose.
    dry.remove(Path::new("/home/u/.zshrc")).expect("remove");

    assert_eq!(
        backing.snapshot(),
        before,
        "the dry run touched the filesystem"
    );
}

/// [R-FS-012] a read of a path the run wrote answers with what was written, so
/// a hook that writes then reads branches the same way it will for real.
#[test]
fn a_read_sees_what_this_run_would_have_written() {
    let (dry, _backing) = dry_run();

    dry.write(Path::new("/home/u/.zshrc"), b"replaced\n")
        .expect("write");
    assert_eq!(
        dry.read(Path::new("/home/u/.zshrc")).expect("read"),
        b"replaced\n"
    );
}

/// An append composes with what is already on disk, not with nothing.
#[test]
fn an_append_starts_from_the_real_contents() {
    let (dry, _backing) = dry_run();

    dry.append(Path::new("/home/u/.zshrc"), b"added\n")
        .expect("append");
    assert_eq!(
        dry.read(Path::new("/home/u/.zshrc")).expect("read"),
        b"original\nadded\n"
    );
}

/// A removed path is gone for the rest of the run, or a hook that deletes and
/// then tests takes a branch the real run will not.
#[test]
fn a_removed_path_stays_removed_for_the_rest_of_the_run() {
    let (dry, _backing) = dry_run();

    dry.remove(Path::new("/home/u/.zshrc")).expect("remove");
    assert!(!dry.exists(Path::new("/home/u/.zshrc")).expect("exists"));
    assert!(dry.read(Path::new("/home/u/.zshrc")).is_err());
}

/// Reads fall through, so a hook sees the machine as it is rather than as
/// empty.
#[test]
fn an_untouched_path_reads_through_to_the_real_filesystem() {
    let (dry, _backing) = dry_run();
    assert_eq!(
        dry.read(Path::new("/home/u/.zshrc")).expect("read"),
        b"original\n"
    );
}

/// The point of recording rather than discarding: the plan is what a dry run
/// exists to print.
#[test]
fn the_run_can_report_what_it_would_have_done() {
    let (dry, _backing) = dry_run();

    dry.write(Path::new("/home/u/a"), b"x").expect("write");
    dry.symlink(Path::new("/src"), Path::new("/home/u/b"), None)
        .expect("symlink");
    dry.remove(Path::new("/home/u/.zshrc")).expect("remove");

    let intents = dry.intents();
    assert_eq!(intents.len(), 3, "{intents:?}");
    assert!(
        intents
            .iter()
            .any(|(p, i)| p.ends_with("b") && matches!(i, Intent::Linked { .. })),
        "{intents:?}"
    );
    assert!(
        intents
            .iter()
            .any(|(p, i)| p.ends_with(".zshrc") && matches!(i, Intent::Removed)),
        "{intents:?}"
    );
}
