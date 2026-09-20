//! What the binary reports about itself.
//!
//! From the build rather than from variables a linker patched, so a build
//! with no flags reports what it is instead of `dev/unknown/unknown`; see
//! [R-CLI-013].

/// The version, the target, the commit, and the build date.
///
/// The shape `internal/version/version.go` produces, with the Rust target
/// triple where Go writes `<goos>/<goarch>`: the triple is what identifies a
/// Rust build, and writing `macos/aarch64` would name neither toolchain's
/// convention.
pub const STRING: &str = concat!(
    "meowctl ",
    env!("CARGO_PKG_VERSION"),
    " (",
    env!("MEOWCTL_TARGET"),
    ", commit ",
    env!("MEOWCTL_COMMIT"),
    ", built ",
    env!("MEOWCTL_BUILD_DATE"),
    ")"
);

/// The same string, for a caller that wants it owned.
#[must_use]
pub fn string() -> String {
    STRING.to_owned()
}

/// Just the version, for comparing against a release tag.
pub const NUMBER: &str = env!("CARGO_PKG_VERSION");

/// The target triple this build is for, which names the asset it updates
/// from; see [R-CLI-073].
pub const TARGET: &str = env!("MEOWCTL_TARGET");
