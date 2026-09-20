//! [R-STAR-020] and [R-STAR-021]: what a `load()` argument means.
//!
//! The `init.star` convention is the part worth pinning. `RegistryLoader`'s
//! doc comment in `v0.1.0` says `@stdlib//components/apt` and
//! `@stdlib//components/apt.star` are equivalent, and `parseRegistryURL` a few
//! lines below it appends `/init.star` to the first. The code is what runs and
//! what every stdlib component depends on.

use meowctl_module::{LocalRoot, ModuleUrl};

#[test]
fn the_four_schemes_parse() {
    assert_eq!(
        ModuleUrl::parse("self//lib/helpers.star").expect("self"),
        ModuleUrl::Local {
            root: LocalRoot::Dotfiles,
            path: "lib/helpers.star".to_owned()
        }
    );
    assert_eq!(
        ModuleUrl::parse("user://modules/helpers.star").expect("user"),
        ModuleUrl::Local {
            root: LocalRoot::Config,
            path: "modules/helpers.star".to_owned()
        }
    );
    assert_eq!(
        ModuleUrl::parse("@stdlib//components/apt.star").expect("registry"),
        ModuleUrl::Registry {
            module: "stdlib".to_owned(),
            path: "components/apt.star".to_owned()
        }
    );
    assert_eq!(
        ModuleUrl::parse("github://meowshed/dotmeow@v1.2.3//init.star").expect("github"),
        ModuleUrl::GitHub {
            owner: "meowshed".to_owned(),
            repo: "dotmeow".to_owned(),
            reference: "v1.2.3".to_owned(),
            path: "init.star".to_owned()
        }
    );
}

/// [R-STAR-020] a path with no scheme is relative to the config directory,
/// which is the one form `v0.1.0`'s composite loader rejects outright.
#[test]
fn a_bare_path_resolves_against_the_config_directory() {
    assert_eq!(
        ModuleUrl::parse("components/zsh/init.star").expect("bare"),
        ModuleUrl::Local {
            root: LocalRoot::Config,
            path: "components/zsh/init.star".to_owned()
        }
    );
}

/// [R-STAR-021] a final segment with no `.` means the directory's
/// `init.star`, which is how every stdlib component is laid out.
#[test]
fn a_path_without_an_extension_means_the_directorys_init_star() {
    assert_eq!(
        ModuleUrl::parse("@stdlib//components/apt")
            .expect("registry")
            .path(),
        "components/apt/init.star"
    );
    assert_eq!(
        ModuleUrl::parse("self//lib").expect("self").path(),
        "lib/init.star"
    );
    assert_eq!(
        ModuleUrl::parse("@stdlib").expect("root").path(),
        "init.star"
    );
}

/// [R-STAR-021] and the same path with an extension is that file. The two are
/// not the same file, whatever the doc comment says.
#[test]
fn a_path_with_an_extension_is_taken_as_written() {
    assert_eq!(
        ModuleUrl::parse("@stdlib//components/apt.star")
            .expect("registry")
            .path(),
        "components/apt.star"
    );
}

/// `v0.1.0` takes a `github://` path exactly as written, and every URL in the
/// wild names a file.
#[test]
fn a_github_path_gets_no_init_star() {
    assert_eq!(
        ModuleUrl::parse("github://o/r@main//lib")
            .expect("github")
            .path(),
        "lib"
    );
}

#[test]
fn a_url_round_trips_through_display() {
    for raw in [
        "self//lib/helpers.star",
        "user://modules/helpers.star",
        "@stdlib//components/apt.star",
        "github://meowshed/dotmeow@v1.2.3//init.star",
    ] {
        let parsed = ModuleUrl::parse(raw).expect(raw);
        assert_eq!(parsed.to_string(), raw);
    }
}

/// Every refusal names what was wrong, because a `load()` that fails with
/// "invalid URL" leaves the author counting slashes.
#[test]
fn a_malformed_url_says_what_is_missing() {
    for (raw, expected) in [
        ("@", "module name"),
        ("@//path", "module name"),
        ("@name//", "path after"),
        ("github://o/r//init.star", "@ref"),
        ("github://or@main//init.star", "owner/repo"),
        ("github://o/r@main//", "path after"),
        ("self//", "no path"),
        ("self///etc/passwd", "cannot start with"),
        ("", "no path"),
    ] {
        let err = ModuleUrl::parse(raw).expect_err(raw);
        assert!(
            err.to_string().contains(expected),
            "{raw}: wanted {expected:?}, got {err}"
        );
    }
}
