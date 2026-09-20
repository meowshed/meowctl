//! [R-CONFIG-002], [R-CONFIG-062], and [R-CONFIG-063]: what every format
//! promises about how it reaches the disk.
//!
//! These three are properties of all the writers rather than of one, and the
//! atomic rename in [R-FS-003] is what provides them. What is checked here is
//! that each writer goes through the trait, so the guarantee holds for every
//! format rather than for the two `v0.1.0` happened to make atomic.

// `clippy.toml` exempts a function carrying `#[test]`; a helper in a test
// binary is test code by construction.
#![allow(clippy::expect_used)]

use std::path::Path;

use meowctl_config::{LockFile, Modfile, Sentinel};
use meowctl_fs::{FileSystem, FsError, FsResult, MemFs};

/// A filesystem that refuses every write, to stand in for a full disk.
#[derive(Debug)]
struct ReadOnlyFs(MemFs);

impl FileSystem for ReadOnlyFs {
    fn write(&self, path: &Path, _contents: &[u8]) -> FsResult<()> {
        Err(FsError::Io {
            operation: "writing",
            path: path.to_path_buf(),
            source: std::io::Error::other("the disk is full"),
        })
    }
    fn read(&self, path: &Path) -> FsResult<Vec<u8>> {
        self.0.read(path)
    }
    fn append(&self, path: &Path, contents: &[u8]) -> FsResult<()> {
        self.0.append(path, contents)
    }
    fn remove(&self, path: &Path) -> FsResult<()> {
        self.0.remove(path)
    }
    fn remove_dir_all(&self, path: &Path) -> FsResult<()> {
        self.0.remove_dir_all(path)
    }
    fn copy(&self, from: &Path, to: &Path) -> FsResult<()> {
        self.0.copy(from, to)
    }
    fn symlink(
        &self,
        target: &Path,
        link: &Path,
        backup: Option<&Path>,
    ) -> FsResult<meowctl_fs::Displaced> {
        self.0.symlink(target, link, backup)
    }
    fn read_link(&self, path: &Path) -> FsResult<std::path::PathBuf> {
        self.0.read_link(path)
    }
    fn remove_symlink(&self, path: &Path) -> FsResult<()> {
        self.0.remove_symlink(path)
    }
    fn create_dir_all(&self, path: &Path) -> FsResult<bool> {
        self.0.create_dir_all(path)
    }
    fn set_executable(&self, path: &Path, executable: bool) -> FsResult<()> {
        self.0.set_executable(path, executable)
    }
    fn read_dir(&self, path: &Path) -> FsResult<Vec<std::path::PathBuf>> {
        self.0.read_dir(path)
    }
    fn rename(&self, from: &Path, to: &Path) -> FsResult<()> {
        self.0.rename(from, to)
    }
    fn entry(&self, path: &Path) -> FsResult<Option<meowctl_fs::Entry>> {
        self.0.entry(path)
    }
}

fn seeded() -> MemFs {
    let fs = MemFs::new();
    fs.create_dir_all(Path::new("/cfg")).expect("config dir");
    fs
}

/// [R-CONFIG-062] a write that fails leaves the previous file, because
/// [R-FS-003] renames into place rather than truncating first. `v0.1.0` uses
/// a plain `os.WriteFile` for `deps.mod`, where a failure halfway through
/// leaves a manifest that no longer parses.
#[test]
fn a_failed_write_leaves_the_previous_file() {
    let backing = seeded();
    backing
        .write(
            Path::new("/cfg/deps.mod"),
            b"dep(\"stdlib\", version = \"1\")\n",
        )
        .expect("the previous manifest");

    let refusing = ReadOnlyFs(backing);
    let modfile = Modfile::default();
    assert!(
        modfile
            .write(&refusing, Path::new("/cfg/deps.mod"))
            .is_err(),
        "the write fails"
    );
    assert_eq!(
        refusing
            .read(Path::new("/cfg/deps.mod"))
            .expect("still there"),
        b"dep(\"stdlib\", version = \"1\")\n"
    );
}

/// [R-CONFIG-002] every format goes through the trait, so every format gets
/// the atomic write. The check is that each writer produces a file a reader
/// can read back, through a filesystem that is the only route to the disk.
#[test]
fn every_format_writes_through_the_filesystem() {
    let fs = seeded();

    LockFile::default()
        .write(&fs, Path::new("/cfg/deps.lock"))
        .expect("the lock writes");
    Modfile::default()
        .write(&fs, Path::new("/cfg/deps.mod"))
        .expect("the manifest writes");
    Sentinel::default()
        .write(&fs, Path::new("/cfg/state.toml"))
        .expect("the sentinel writes");

    for path in ["/cfg/deps.lock", "/cfg/deps.mod", "/cfg/state.toml"] {
        assert!(
            fs.exists(Path::new(path)).expect("asking"),
            "{path} was written"
        );
    }
}

/// [R-CONFIG-063] two runs writing the same file cannot interleave into
/// something that is neither. The rename is what gives this: a reader sees
/// one of the two complete files and never a mixture of both.
#[test]
fn two_writers_leave_one_whole_file_and_not_a_mixture() {
    let fs = std::sync::Arc::new(seeded());

    let first = LockFile {
        meta: meowctl_config::LockMeta {
            generated_by: "first".to_owned(),
            updated_at: "2026-09-20T00:00:00Z".to_owned(),
        },
        ..LockFile::default()
    };
    let second = LockFile {
        meta: meowctl_config::LockMeta {
            generated_by: "second".to_owned(),
            updated_at: "2026-09-20T00:00:01Z".to_owned(),
        },
        ..LockFile::default()
    };

    std::thread::scope(|scope| {
        for lock in [&first, &second] {
            let fs = std::sync::Arc::clone(&fs);
            scope.spawn(move || {
                for _ in 0..50 {
                    lock.write(fs.as_ref(), Path::new("/cfg/deps.lock"))
                        .expect("the lock writes");
                    let read = LockFile::read(fs.as_ref(), Path::new("/cfg/deps.lock"))
                        .expect("the lock reads back");
                    assert!(
                        read.meta.generated_by == "first" || read.meta.generated_by == "second",
                        "one writer's file, not a mixture: {:?}",
                        read.meta
                    );
                }
            });
        }
    });
}
