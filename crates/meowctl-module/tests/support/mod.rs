//! Building the archives the fetching tests fetch.
//!
//! Shared by several test binaries, each of which needs some of it. Rust
//! compiles the module separately into each one, so a helper another binary
//! uses looks dead here.
#![allow(dead_code)]
// `clippy.toml` exempts a function carrying `#[test]`; a helper in a test
// binary is test code by construction.
#![allow(clippy::expect_used)]

use std::io::Write as _;

use flate2::Compression;
use flate2::write::GzEncoder;

/// One entry to put in a tarball.
pub struct Entry {
    /// The path inside the archive.
    pub path: String,
    /// What the file holds.
    pub contents: &'static str,
    /// Whether the entry carries the executable bit.
    pub executable: bool,
}

impl Entry {
    /// A regular file.
    pub fn file(path: &str, contents: &'static str) -> Entry {
        Entry {
            path: path.to_owned(),
            contents,
            executable: false,
        }
    }

    /// A file the archive marks executable.
    pub fn script(path: &str, contents: &'static str) -> Entry {
        Entry {
            path: path.to_owned(),
            contents,
            executable: true,
        }
    }

    /// The same entry, one directory down.
    fn under(&self, root: &str) -> Entry {
        Entry {
            path: format!("{root}/{}", self.path),
            contents: self.contents,
            executable: self.executable,
        }
    }
}

/// A gzipped tar holding these entries, the way a release tarball is built.
///
/// # Panics
///
/// If the archive cannot be written, which would be a bug in the test.
#[must_use]
pub fn tarball(entries: &[Entry]) -> Vec<u8> {
    let mut builder = tar::Builder::new(GzEncoder::new(Vec::new(), Compression::fast()));
    for entry in entries {
        let mut header = tar::Header::new_gnu();
        header.set_size(entry.contents.len() as u64);
        header.set_mode(if entry.executable { 0o755 } else { 0o644 });
        header.set_cksum();
        builder
            .append_data(&mut header, &entry.path, entry.contents.as_bytes())
            .expect("appending to the test archive");
    }
    builder
        .into_inner()
        .expect("finishing the test archive")
        .finish()
        .expect("compressing the test archive")
}

/// A gzipped tar with everything under one directory, the way
/// `github.com/<owner>/<repo>/archive/<ref>.tar.gz` serves a repository.
#[must_use]
pub fn github_tarball(root: &str, entries: &[Entry]) -> Vec<u8> {
    let nested: Vec<Entry> = entries.iter().map(|entry| entry.under(root)).collect();
    tarball(&nested)
}

/// An archive whose one entry climbs out of the module root with `..`.
///
/// `tar::Builder` refuses to write such a path, which is the right default and
/// is not what a hostile archive does, so the name field of a valid header is
/// overwritten afterwards and its checksum recomputed. Everything meowctl
/// fetches is remote input, and this is the archive that check exists for; see
/// [R-MODULE-033].
///
/// # Panics
///
/// If the archive cannot be written.
#[must_use]
pub fn escaping_tarball() -> Vec<u8> {
    let mut builder = tar::Builder::new(Vec::new());
    let payload = b"pwned";
    let mut header = tar::Header::new_gnu();
    header.set_size(payload.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    builder
        .append_data(&mut header, "placeholder", &payload[..])
        .expect("appending to the test archive");
    let mut raw = builder.into_inner().expect("finishing the test archive");

    overwrite_name(&mut raw[..512], b"../../escaped");

    let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
    encoder
        .write_all(&raw)
        .expect("compressing the test archive");
    encoder.finish().expect("compressing the test archive")
}

/// An archive whose one entry names an absolute path.
///
/// # Panics
///
/// If the archive cannot be written.
#[must_use]
pub fn absolute_path_tarball() -> Vec<u8> {
    let mut builder = tar::Builder::new(GzEncoder::new(Vec::new(), Compression::fast()));
    let payload = b"pwned";
    let mut header = tar::Header::new_gnu();
    header.set_size(payload.len() as u64);
    header.set_mode(0o644);
    header
        .set_path_absolute("/escaped")
        .expect("setting an absolute path");
    header.set_cksum();
    builder
        .append(&header, &payload[..])
        .expect("appending to the test archive");
    builder
        .into_inner()
        .expect("finishing the test archive")
        .finish()
        .expect("compressing the test archive")
}

/// Replaces a tar header's name and repairs its checksum.
///
/// The name occupies the first 100 bytes and the checksum bytes 148 to 156,
/// computed over the whole header with the checksum field read as spaces.
fn overwrite_name(header: &mut [u8], name: &[u8]) {
    header[..100].fill(0);
    header[..name.len()].copy_from_slice(name);
    header[148..156].fill(b' ');
    let sum: u32 = header.iter().map(|b| u32::from(*b)).sum();
    let encoded = format!("{sum:06o}\0 ");
    header[148..156].copy_from_slice(encoded.as_bytes());
}
