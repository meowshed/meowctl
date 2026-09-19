//! One suite, run against every implementation.
//!
//! The three are interchangeable behind a trait object, which is what lets
//! `meowctl-cli` choose one from the flags and nothing below it care; see
//! [R-FS-014]. A suite that ran against only the real filesystem would let the
//! other two drift, and the drift would show up as a dry run that plans
//! something the run does not do.
//!
//! `RealFs` is exercised inside a temporary directory, so these are the tests
//! that touch a disk. Everything else in the workspace uses `MemFs`.

// `clippy.toml` exempts tests from `expect_used`, but the exemption only
// recognises a function carrying `#[test]`. A helper in a test binary is test
// code by construction, so the whole file is exempt here rather than each
// helper separately.
#![allow(clippy::expect_used)]

use std::path::{Path, PathBuf};

use meowctl_fs::{Displaced, DryRunFs, Entry, FileSystem, FsError, MemFs, RealFs};

/// A filesystem to test, and the absolute root to work under.
struct Subject {
    name: &'static str,
    fs: Box<dyn FileSystem>,
    root: PathBuf,
    /// Whether this implementation can create a symlink here.
    ///
    /// `RealFs` refuses on Windows rather than guessing between a file link
    /// and a directory link, both of which need a privilege the machine may
    /// not grant. The in-memory implementations have no such constraint, so
    /// the difference is real and the tests say which side they are checking.
    symlinks: bool,
    /// Kept alive so the temporary directory outlives the test.
    _temp: Option<tempfile::TempDir>,
}

impl Subject {
    fn path(&self, rest: &str) -> PathBuf {
        self.root.join(rest)
    }
}

/// Every implementation, each with a root directory that already exists.
fn subjects() -> Vec<Subject> {
    let mem = MemFs::new();
    mem.create_dir_all(Path::new("/root"))
        .expect("seeding the memory root");

    let dry_backing = MemFs::new();
    dry_backing
        .create_dir_all(Path::new("/root"))
        .expect("seeding the dry-run root");

    let temp = tempfile::tempdir().expect("creating a temporary directory");
    let real_root = temp.path().to_path_buf();

    vec![
        Subject {
            name: "MemFs",
            fs: Box::new(mem),
            root: PathBuf::from("/root"),
            symlinks: true,
            _temp: None,
        },
        Subject {
            name: "DryRunFs",
            fs: Box::new(DryRunFs::new(Box::new(dry_backing))),
            root: PathBuf::from("/root"),
            symlinks: true,
            _temp: None,
        },
        Subject {
            name: "RealFs",
            fs: Box::new(RealFs::new()),
            root: real_root,
            symlinks: cfg!(unix),
            _temp: Some(temp),
        },
    ]
}

/// Runs `check` against every implementation, naming the one that failed.
fn for_each(check: impl Fn(&Subject)) {
    for subject in subjects() {
        check(&subject);
    }
}

/// [R-FS-012] a hook that writes a file and reads it back must take the same
/// branch under a dry run as it will for real, which means the dry run has to
/// remember what it pretended to write.
#[test]
fn a_write_is_visible_to_a_later_read() {
    for_each(|s| {
        let path = s.path("config");
        s.fs.write(&path, b"one").expect(s.name);
        assert_eq!(s.fs.read(&path).expect(s.name), b"one", "{}", s.name);

        s.fs.write(&path, b"two").expect(s.name);
        assert_eq!(s.fs.read(&path).expect(s.name), b"two", "{}", s.name);
    });
}

#[test]
fn an_append_extends_what_is_there() {
    for_each(|s| {
        let path = s.path("log");
        s.fs.append(&path, b"a\n").expect(s.name);
        s.fs.append(&path, b"b\n").expect(s.name);
        assert_eq!(s.fs.read(&path).expect(s.name), b"a\nb\n", "{}", s.name);
    });
}

/// [R-FS-033] a write into a directory that does not exist fails everywhere,
/// so a dry run does not promise something the run will refuse.
#[test]
fn a_write_into_a_missing_directory_fails_with_the_directory_named() {
    for_each(|s| {
        let path = s.path("absent/config");
        let err = s.fs.write(&path, b"x").expect_err(s.name);
        assert!(
            matches!(err, FsError::NoParent { .. }),
            "{}: {err:?}",
            s.name
        );
    });
}

/// The directory an earlier step created counts, or a plan stops halfway
/// through a component that would have worked.
#[test]
fn a_write_into_a_directory_this_run_created_succeeds() {
    for_each(|s| {
        let dir = s.path("made");
        assert!(s.fs.create_dir_all(&dir).expect(s.name), "{}", s.name);
        s.fs.write(&dir.join("file"), b"x").expect(s.name);
    });
}

/// [R-OPS-013] the inverse of a `mkdir` removes the directory only when
/// meowctl created it, so the answer here decides whether rollback deletes
/// something the user had.
#[test]
fn creating_a_directory_reports_whether_it_made_one() {
    for_each(|s| {
        let dir = s.path("new");
        assert!(s.fs.create_dir_all(&dir).expect(s.name), "{}", s.name);
        assert!(!s.fs.create_dir_all(&dir).expect(s.name), "{}", s.name);
    });
}

/// [R-FS-020] re-running `apply` re-links an existing symlink, and the caller
/// needs the previous target to journal an inverse that puts it back.
#[test]
fn replacing_a_symlink_reports_what_it_displaced() {
    for_each(|s| {
        if !s.symlinks {
            return;
        }
        let link = s.path("link");
        let first = s.path("first");
        let second = s.path("second");

        assert_eq!(
            s.fs.symlink(&first, &link, None).expect(s.name),
            Displaced::Nothing,
            "{}",
            s.name
        );
        assert_eq!(
            s.fs.symlink(&second, &link, None).expect(s.name),
            Displaced::Symlink { target: first },
            "{}",
            s.name
        );
        assert_eq!(s.fs.read_link(&link).expect(s.name), second, "{}", s.name);
    });
}

/// [R-FS-021] a user's own file must not disappear without a record.
#[test]
fn a_symlink_over_a_regular_file_is_refused_without_a_backup() {
    for_each(|s| {
        let link = s.path("dotfile");
        s.fs.write(&link, b"mine").expect(s.name);

        let err =
            s.fs.symlink(&s.path("target"), &link, None)
                .expect_err(s.name);
        assert!(
            matches!(err, FsError::WouldClobber { .. }),
            "{}: {err:?}",
            s.name
        );
    });
}

#[test]
fn a_symlink_over_a_regular_file_moves_it_aside_when_asked() {
    for_each(|s| {
        if !s.symlinks {
            return;
        }
        let link = s.path("dotfile");
        let backup = s.path("dotfile.backup");
        s.fs.write(&link, b"mine").expect(s.name);

        let displaced =
            s.fs.symlink(&s.path("target"), &link, Some(&backup))
                .expect(s.name);
        assert_eq!(
            displaced,
            Displaced::BackedUp {
                backup: backup.clone()
            },
            "{}",
            s.name
        );
        assert!(
            matches!(
                s.fs.entry(&link).expect(s.name),
                Some(Entry::Symlink { .. })
            ),
            "{}",
            s.name
        );
    });
}

/// [R-FS-022] a mistyped path must not delete something real.
/// The refusal is deliberate, and a test that skipped it would let the
/// platform behaviour change without anybody noticing.
#[cfg(not(unix))]
#[test]
fn creating_a_symlink_on_a_platform_without_them_says_so() {
    for_each(|s| {
        if s.symlinks {
            return;
        }
        let err =
            s.fs.symlink(&s.path("target"), &s.path("link"), None)
                .expect_err(s.name);
        assert!(
            err.to_string().contains("not supported"),
            "{}: {err}",
            s.name
        );
    });
}

#[test]
fn removing_a_symlink_refuses_a_regular_file() {
    for_each(|s| {
        let path = s.path("real");
        s.fs.write(&path, b"x").expect(s.name);

        let err = s.fs.remove_symlink(&path).expect_err(s.name);
        assert!(
            matches!(err, FsError::NotASymlink { .. }),
            "{}: {err:?}",
            s.name
        );
        assert!(s.fs.exists(&path).expect(s.name), "{}", s.name);
    });
}

/// [R-FS-032] a missing lock file is a clean slate and a missing `init.star`
/// is fatal, so the caller has to be able to tell them apart.
#[test]
fn a_missing_file_is_distinguishable_from_another_failure() {
    for_each(|s| {
        let err = s.fs.read(&s.path("absent")).expect_err(s.name);
        assert!(
            matches!(err, FsError::NotFound { .. }),
            "{}: {err:?}",
            s.name
        );
    });
}

/// [R-CTX-042] asking is how a component tests before it reads, so asking must
/// not be an error.
#[test]
fn asking_about_a_missing_path_is_not_an_error() {
    for_each(|s| {
        assert_eq!(
            s.fs.entry(&s.path("absent")).expect(s.name),
            None,
            "{}",
            s.name
        );
        assert!(!s.fs.exists(&s.path("absent")).expect(s.name), "{}", s.name);
    });
}

/// The caller wanted it gone and it is gone.
#[test]
fn removing_something_absent_succeeds() {
    for_each(|s| {
        s.fs.remove(&s.path("absent")).expect(s.name);
    });
}

#[test]
fn a_copy_reproduces_the_contents() {
    for_each(|s| {
        let from = s.path("from");
        let to = s.path("to");
        s.fs.write(&from, b"contents").expect(s.name);
        s.fs.copy(&from, &to).expect(s.name);
        assert_eq!(s.fs.read(&to).expect(s.name), b"contents", "{}", s.name);
    });
}

#[test]
fn every_error_names_its_path() {
    for_each(|s| {
        let path = s.path("absent");
        let err = s.fs.read(&path).expect_err(s.name);
        assert_eq!(err.path(), path, "{}", s.name);
        assert!(err.to_string().contains("absent"), "{}: {err}", s.name);
    });
}
