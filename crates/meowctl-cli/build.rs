//! What the binary reports about itself.
//!
//! The commit and the build date are captured here rather than patched in by
//! linker flags, so `meowctl version` reports what was actually built and a
//! build with no `-ldflags` does not report `dev/unknown/unknown`; see
//! [R-CLI-013].

use std::process::Command;

fn main() {
    // Rerun when the checked-out commit moves, and no more often.
    println!("cargo:rerun-if-changed=../../.git/HEAD");
    println!("cargo:rerun-if-env-changed=MEOWCTL_COMMIT");
    println!("cargo:rerun-if-env-changed=MEOWCTL_BUILD_DATE");

    println!("cargo:rustc-env=MEOWCTL_COMMIT={}", commit());
    println!("cargo:rustc-env=MEOWCTL_BUILD_DATE={}", build_date());
    println!(
        "cargo:rustc-env=MEOWCTL_TARGET={}",
        std::env::var("TARGET").unwrap_or_else(|_| "unknown".to_owned())
    );
}

/// The short commit, or what the environment says, or `unknown`.
///
/// A release build sets `MEOWCTL_COMMIT` because the tarball it builds from
/// has no `.git`.
fn commit() -> String {
    if let Ok(given) = std::env::var("MEOWCTL_COMMIT") {
        return given;
    }
    Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .filter(|out| out.status.success())
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .map_or_else(|| "unknown".to_owned(), |text| text.trim().to_owned())
}

/// The date of that commit, which is reproducible, rather than the date of
/// the build, which is not.
fn build_date() -> String {
    if let Ok(given) = std::env::var("MEOWCTL_BUILD_DATE") {
        return given;
    }
    Command::new("git")
        .args(["log", "-1", "--format=%cs"])
        .output()
        .ok()
        .filter(|out| out.status.success())
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .map_or_else(|| "unknown".to_owned(), |text| text.trim().to_owned())
}
