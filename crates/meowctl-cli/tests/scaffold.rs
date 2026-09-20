//! What `init` writes, byte for byte.
//!
//! The scaffold is the first thing a user sees of the tool and it lives in
//! their dotfiles repository afterwards, so a reworded comment is a diff in a
//! file they did not change. `v0.1.0` produced these exact bytes.

// `clippy.toml` exempts a function carrying `#[test]`; a helper in a test
// binary is test code by construction.
#![allow(clippy::expect_used)]

use std::path::Path;

use meowctl_config::{Layout, Modfile};
use meowctl_fs::{FileSystem as _, MemFs};

/// The `deps.mod` a new configuration gets, rendered the way `init` renders
/// it.
fn scaffolded_modfile() -> String {
    Modfile {
        module: Some(meowctl_config::Module {
            name: "my-dotfiles".to_owned(),
            version: "0.1.0".to_owned(),
        }),
        ..Modfile::default()
    }
    .render()
}

/// The exact file `v0.1.0`'s `meowctl init` writes.
#[test]
fn the_scaffolded_modfile_is_the_one_v0_1_0_writes() {
    assert_eq!(
        scaffolded_modfile(),
        "# meowctl.mod — machine-managed module file. Do not edit by hand.\n\
         \n\
         module(\n    name = \"my-dotfiles\",\n    version = \"0.1.0\",\n)\n\n"
    );
}

/// The three templates are quoted from `internal/cli/commands.go`, so a test
/// that only checked they were non-empty would catch nothing. These check the
/// shape a user reads: the file says what it is and shows an example.
#[test]
fn each_template_names_itself_and_shows_an_example() {
    for (name, text, example) in [
        ("init.star", meowctl_cli::templates::INIT_STAR, "component("),
        (
            "local.star",
            meowctl_cli::templates::LOCAL_STAR,
            "component(",
        ),
        (
            "deps.local.mod",
            meowctl_cli::templates::LOCAL_MOD,
            "shadow",
        ),
    ] {
        assert!(text.starts_with(&format!("# {name}")), "{name}: {text}");
        assert!(text.contains(example), "{name}: {text}");
        assert!(text.ends_with('\n'), "{name} ends with a newline");
    }
}

/// A layout puts every file where `v0.1.0` puts it: a configuration is shared
/// between the two binaries during the cutover.
#[test]
fn the_layout_names_the_files_v0_1_0_names() {
    let layout = Layout::new("/cfg");
    let named: Vec<String> = [
        layout.entry(),
        layout.local_entry(),
        layout.modfile(),
        layout.local_modfile(),
        layout.lock(),
        layout.local_lock(),
        layout.state(),
        layout.installed(),
        layout.components(),
    ]
    .iter()
    .map(|path| {
        path.file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default()
    })
    .collect();

    assert_eq!(
        named,
        [
            "init.star",
            "local.star",
            "deps.mod",
            "deps.local.mod",
            "deps.lock",
            "deps.local.lock",
            "state.toml",
            "installed.lock",
            "components",
        ]
    );
}

/// A configuration directory with nothing in it is not one, and saying so is
/// what tells a user to run `init` rather than sending them after a missing
/// file; see [R-CLI-050].
#[test]
fn an_empty_directory_is_not_a_configuration() {
    let fs = MemFs::new();
    fs.create_dir_all(Path::new("/cfg")).expect("the directory");
    let layout = Layout::new("/cfg");
    assert!(layout.check(&fs).is_err());

    fs.write(&layout.entry(), b"# empty\n").expect("the entry");
    assert!(layout.check(&fs).is_ok());
}
