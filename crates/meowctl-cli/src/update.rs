//! Replacing the running binary with the one a release published.
//!
//! The one command that changes the tool rather than the machine, which is
//! why it runs alone and why nothing else calls it; see [R-CLI-075].
//!
//! The policy -- which asset, which hash, whether these bytes are it -- is in
//! `meowctl-release` and takes no effects. What is here is the fetching and
//! the replacing, because those need an `Http` and a `FileSystem` and this is
//! where they arrive.

use std::path::{Path, PathBuf};

use meowctl_release::{Release, verify};

use crate::run::Session;
use crate::{CliError, CliResult};

/// Where the latest release is described.
const LATEST: &str = "https://api.github.com/repos/meowshed/meowctl/releases/latest";

/// The variable that points `self-update` somewhere else.
///
/// For a fork that publishes its own releases, and for the tests, which
/// cannot ask GitHub for a release that does not exist yet; see
/// [R-CLI-076].
const RELEASES: &str = "MEOWCTL_RELEASES";

/// Updates the running binary, or says why it did not.
///
/// # Errors
///
/// Whatever the fetch, the verification, or the replacement failed with.
pub(super) fn self_update(session: &mut Session<'_>) -> CliResult<()> {
    let latest = session
        .environment
        .get(RELEASES)
        .cloned()
        .unwrap_or_else(|| LATEST.to_owned());

    session.say("checking for a newer release");
    let document = session
        .http
        .get(&latest)
        .map_err(|e| CliError::Module(format!("{latest}: {e}")))?;
    let release = Release::parse(&document).map_err(|e| CliError::Module(e.to_string()))?;

    // Nothing to do is the ordinary outcome, and saying so beats a command
    // that appears to have done something; see [R-CLI-072].
    if release.is(crate::version::NUMBER) {
        session.say(&format!("already at {}", release.tag));
        return Ok(());
    }

    let asset = release
        .asset_for(crate::version::TARGET)
        .map_err(|e| CliError::Module(e.to_string()))?;
    let checksums = release
        .checksums()
        .map_err(|e| CliError::Module(e.to_string()))?;

    // The checksums first. Fetching the binary before knowing whether it can
    // be verified is work that may have to be thrown away, and worse, it is
    // the order in which somebody eventually skips the check.
    session.say(&format!("fetching {} for {}", release.tag, asset.name));
    let published = session
        .http
        .get(&checksums.url)
        .map_err(|e| CliError::Module(format!("{}: {e}", checksums.url)))?;
    let published = String::from_utf8(published)
        .map_err(|_| CliError::Module(format!("{} is not text", meowctl_release::CHECKSUMS)))?;

    let bytes = session
        .http
        .get(&asset.url)
        .map_err(|e| CliError::Module(format!("{}: {e}", asset.url)))?;

    verify(&published, &asset.name, &bytes).map_err(|e| CliError::Module(e.to_string()))?;

    let running = std::env::current_exe()
        .map_err(|e| CliError::General(format!("this binary cannot be located: {e}")))?;
    replace(session, &running, &bytes)?;

    session.say(&format!(
        "updated to {}; run 'meowctl version' to confirm",
        release.tag
    ));
    Ok(())
}

/// Puts the new binary where the old one is.
///
/// Beside it, made executable, then renamed over: a partial write over the
/// binary leaves a machine with no working `meowctl` and no way to fetch one;
/// see [R-CLI-074].
fn replace(session: &Session<'_>, running: &Path, bytes: &[u8]) -> CliResult<()> {
    put_in_place(session.fs.as_ref(), running, bytes)
}

/// The same, against a filesystem of the caller's choosing.
///
/// Separate so the three steps can be checked without a real binary to
/// overwrite; see [R-CLI-074].
fn put_in_place(fs: &dyn meowctl_fs::FileSystem, running: &Path, bytes: &[u8]) -> CliResult<()> {
    let staged = staged_beside(running);

    fs.write(&staged, bytes)
        .map_err(|e| CliError::General(format!("{}: {e}", staged.display())))?;
    fs.set_executable(&staged, true)
        .map_err(|e| CliError::General(format!("{}: {e}", staged.display())))?;
    fs.rename(&staged, running).map_err(|e| {
        // The staged file is the only trace, and leaving it beside the binary
        // is worse than losing the reason it could not be moved.
        let _ = fs.remove(&staged);
        CliError::General(format!("{}: {e}", running.display()))
    })?;
    Ok(())
}

/// A name beside the running binary, so the rename stays on one filesystem.
fn staged_beside(running: &Path) -> PathBuf {
    let mut staged = running.to_path_buf();
    let name = running.file_name().map_or_else(
        || "meowctl".to_owned(),
        |n| n.to_string_lossy().into_owned(),
    );
    staged.set_file_name(format!(".{name}-{}.new", std::process::id()));
    staged
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use meowctl_fs::{FileSystem, MemFs};

    use super::*;

    /// [R-CLI-074] written beside, made executable, then renamed over. A
    /// partial write over the binary leaves a machine with no working
    /// `meowctl` and no way to fetch one.
    #[test]
    fn the_new_binary_arrives_whole_or_not_at_all() {
        let fs = MemFs::new();
        fs.create_dir_all(Path::new("/usr/local/bin"))
            .expect("the directory");
        let running = Path::new("/usr/local/bin/meowctl");
        fs.write(running, b"the old one").expect("the old binary");

        put_in_place(&fs, running, b"the new one").expect("the replacement");

        assert_eq!(fs.read(running).expect("read"), b"the new one");
        assert!(
            fs.entry(running).expect("ask").is_some_and(|e| matches!(
                e,
                meowctl_fs::Entry::File {
                    executable: true,
                    ..
                }
            )),
            "the new binary is not executable"
        );
    }

    /// [R-CLI-074] and nothing is left beside it, whichever way it went.
    #[test]
    fn the_staged_file_does_not_survive() {
        let fs = MemFs::new();
        fs.create_dir_all(Path::new("/usr/local/bin"))
            .expect("the directory");
        let running = Path::new("/usr/local/bin/meowctl");
        fs.write(running, b"the old one").expect("the old binary");

        put_in_place(&fs, running, b"the new one").expect("the replacement");

        let left: Vec<_> = fs
            .read_dir(Path::new("/usr/local/bin"))
            .expect("listing")
            .into_iter()
            .filter(|p| p.to_string_lossy().contains(".new"))
            .collect();
        assert!(left.is_empty(), "left behind: {left:?}");
    }

    /// [R-CLI-074] the staged name is beside the binary, so the rename that
    /// follows stays on one filesystem. A temporary directory elsewhere
    /// turns the rename into a copy that can fail halfway.
    #[test]
    fn the_staged_name_is_beside_the_binary() {
        let staged = staged_beside(Path::new("/usr/local/bin/meowctl"));
        assert_eq!(
            staged.parent(),
            Path::new("/usr/local/bin")
                .parent()
                .map(|_| Path::new("/usr/local/bin"))
        );
        assert_ne!(staged.file_name(), Some(std::ffi::OsStr::new("meowctl")));
    }
}
