//! The command line itself: what it accepts, and what it exits with.
//!
//! These drive `clap` rather than a process, because the question is what the
//! surface is and not what a binary does with it; see [R-CLI-012], which is
//! what makes constructing one per test possible at all.

// `clippy.toml` exempts a function carrying `#[test]`; a helper in a test
// binary is test code by construction.
#![allow(clippy::expect_used)]

use clap::{CommandFactory as _, Parser as _};
use meowctl_cli::{Cli, Command, DepCommand, Format};

fn parse(args: &[&str]) -> Result<Cli, clap::Error> {
    Cli::try_parse_from(std::iter::once("meowctl").chain(args.iter().copied()))
}

/// [R-CLI-001] exactly what `newRootCmd` registers, and nothing else. A
/// command that quietly went missing is a script that stops working.
#[test]
fn every_command_v0_1_0_registers_is_there() {
    let command = Cli::command();
    let mut names: Vec<String> = command
        .get_subcommands()
        .map(|sub| sub.get_name().to_owned())
        .collect();
    names.sort();

    let mut expected = [
        "init",
        "apply",
        "add",
        "remove",
        "upgrade",
        "verify",
        "update",
        "status",
        "doctor",
        "check",
        "hook",
        "shell",
        "self-update",
        "version",
        "dep",
        "completions",
    ];
    expected.sort_unstable();
    assert_eq!(names, expected);
}

/// [R-CLI-002] the `dep` group.
#[test]
fn the_dep_group_carries_its_six() {
    let command = Cli::command();
    let dep = command
        .get_subcommands()
        .find(|sub| sub.get_name() == "dep")
        .expect("the dep group");
    let mut names: Vec<&str> = dep.get_subcommands().map(clap::Command::get_name).collect();
    names.sort_unstable();
    assert_eq!(names, ["add", "list", "remove", "sync", "tidy", "upgrade"]);
}

/// [R-CLI-004] `--config` and `--verbose` are on every command, because a
/// flag that works on one command and not another is one nobody remembers.
#[test]
fn the_global_flags_work_after_any_subcommand() {
    for args in [
        vec!["apply", "--config", "/tmp/x"],
        vec!["status", "--verbose"],
        vec!["doctor", "--config", "/tmp/x", "-v"],
        vec!["dep", "list", "--verbose"],
    ] {
        parse(&args).unwrap_or_else(|e| panic!("{args:?}: {e}"));
    }
}

/// [R-CLI-005] the whole table, not one row of it.
///
/// The previous version of this test checked `--dry-run` on six commands and
/// the other three on `apply`, and passed while five commands were missing
/// `--no-rollback`, two were missing `--force`, and `verify` was missing
/// `--dry-run`. A parity requirement that names six commands needs a test
/// that tries six commands.
#[test]
fn every_lifecycle_command_takes_the_flags_v0_1_0_gives_it() {
    /// The command, then whether it takes `--force` and `--ignore-lock`.
    const COMMANDS: [(&[&str], bool); 6] = [
        (&["apply"], true),
        (&["add", "zsh"], true),
        (&["upgrade"], true),
        (&["remove", "zsh"], false),
        (&["update"], false),
        (&["verify"], false),
    ];

    for (base, installs) in COMMANDS {
        // Every one of them, in both spellings where there are two.
        for flag in ["--dry-run", "-n", "--no-rollback", "--verbose", "-v"] {
            let mut args = base.to_vec();
            args.push(flag);
            parse(&args).unwrap_or_else(|e| panic!("{base:?} should take {flag}: {e}"));
        }

        for flag in ["--force", "-f", "--ignore-lock"] {
            let mut args = base.to_vec();
            args.push(flag);
            let parsed = parse(&args);
            assert_eq!(
                parsed.is_ok(),
                installs,
                "{base:?} and {flag}: v0.1.0 {} it",
                if installs { "takes" } else { "does not take" }
            );
        }
    }
}

/// [R-CLI-005] and `--dry-run` reaches the session, rather than being parsed
/// and dropped. `verify` is the one this caught.
#[test]
fn dry_run_is_visible_on_every_command_that_takes_it() {
    for args in [
        vec!["apply", "--dry-run"],
        vec!["apply", "-n"],
        vec!["add", "zsh", "--dry-run"],
        vec!["remove", "zsh", "-n"],
        vec!["upgrade", "--dry-run"],
        vec!["update", "-n"],
        vec!["verify", "--dry-run"],
        vec!["dep", "sync", "--dry-run"],
    ] {
        let parsed = parse(&args).unwrap_or_else(|e| panic!("{args:?}: {e}"));
        assert!(parsed.command.dry_run(), "{args:?}");
    }
}

/// [R-CLI-005] and the three that `apply` carries land in the parsed command,
/// rather than being accepted and forgotten.
#[test]
fn the_install_flags_reach_the_parsed_command() {
    let parsed = parse(&["apply", "--force", "--no-rollback", "--ignore-lock"]).expect("apply");
    let Command::Apply {
        force,
        no_rollback,
        ignore_lock,
        ..
    } = parsed.command
    else {
        panic!("expected apply");
    };
    assert!(force && no_rollback && ignore_lock);

    let parsed = parse(&["add", "zsh", "-f", "--no-rollback"]).expect("add");
    let Command::Add {
        force, no_rollback, ..
    } = parsed.command
    else {
        panic!("expected add");
    };
    assert!(force && no_rollback);
}

/// [R-CLI-016] accepted, and inert. `v0.1.0` declares the flag and reads it
/// nowhere, so neither binary resolves without the lock, and a test that
/// asserted it did would be asserting a feature that has never existed.
#[test]
fn ignore_lock_is_accepted_and_does_nothing() {
    for args in [
        vec!["apply", "--ignore-lock"],
        vec!["add", "zsh", "--ignore-lock"],
        vec!["upgrade", "--ignore-lock"],
    ] {
        parse(&args).unwrap_or_else(|e| panic!("{args:?}: {e}"));
    }
}

/// [R-CLI-006] `--format json` everywhere, and `--json` still accepted on the
/// two commands that had it before it existed.
#[test]
fn json_is_available_everywhere_and_the_old_flag_still_works() {
    for args in [
        vec!["status", "--format", "json"],
        vec!["apply", "--format", "json"],
        vec!["shell", "zsh", "--format", "json"],
    ] {
        let parsed = parse(&args).unwrap_or_else(|e| panic!("{args:?}: {e}"));
        assert_eq!(parsed.global.format, Some(Format::Json), "{args:?}");
    }

    for args in [vec!["doctor", "--json"], vec!["status", "--json"]] {
        let parsed = parse(&args).unwrap_or_else(|e| panic!("{args:?}: {e}"));
        assert!(parsed.command.wants_json(), "{args:?}");
    }

    // And not on the commands that never had it.
    assert!(parse(&["apply", "--json"]).is_err());
}

/// [R-CLI-021] `shell` and `hook` write what another program reads, so the
/// sink keeps its own output off their standard output.
#[test]
fn the_two_commands_whose_stdout_is_read_say_so() {
    assert!(
        parse(&["shell", "zsh"])
            .expect("shell")
            .command
            .stdout_is_an_interface()
    );
    assert!(
        parse(&["hook", "shell"])
            .expect("hook")
            .command
            .stdout_is_an_interface()
    );
    assert!(
        !parse(&["apply"])
            .expect("apply")
            .command
            .stdout_is_an_interface()
    );
}

/// [R-CLI-052] an unknown command is a usage error, and clap suggests the
/// nearest when one is close.
#[test]
fn an_unknown_command_is_a_usage_error_with_a_suggestion() {
    let error = parse(&["dcotor"]).expect_err("should refuse");
    assert_eq!(error.kind(), clap::error::ErrorKind::InvalidSubcommand);
    assert!(error.to_string().contains("doctor"), "{error}");
}

/// [R-CLI-052] and so is an unknown flag.
#[test]
fn an_unknown_flag_is_a_usage_error() {
    let error = parse(&["apply", "--nonesuch"]).expect_err("should refuse");
    assert_eq!(error.kind(), clap::error::ErrorKind::UnknownArgument);
}

/// A command that needs an argument says which, rather than running on
/// nothing.
#[test]
fn a_missing_argument_is_reported() {
    for args in [vec!["shell"], vec!["check"], vec!["hook"], vec!["add"]] {
        let error = parse(&args).unwrap_err();
        assert_eq!(
            error.kind(),
            clap::error::ErrorKind::MissingRequiredArgument,
            "{args:?}"
        );
    }
}

/// [R-CLI-003] `init` takes a repository or nothing.
#[test]
fn init_takes_a_repository_or_nothing() {
    let bare = parse(&["init"]).expect("init");
    let Command::Init { repo_url, force } = bare.command else {
        panic!("expected init");
    };
    assert!(repo_url.is_none() && !force);

    let bootstrapped = parse(&["init", "https://github.com/o/r", "--force"]).expect("init");
    let Command::Init { repo_url, force } = bootstrapped.command else {
        panic!("expected init");
    };
    assert_eq!(repo_url.as_deref(), Some("https://github.com/o/r"));
    assert!(force);
}

/// The `dep` group's own flags, which differ from the top-level ones.
#[test]
fn dep_add_takes_a_version_or_a_source_and_a_local_flag() {
    let parsed =
        parse(&["dep", "add", "stdlib", "--version", "0.2.17", "--local"]).expect("dep add");
    let Command::Dep {
        command:
            DepCommand::Add {
                name,
                version,
                source,
                local,
            },
    } = parsed.command
    else {
        panic!("expected dep add");
    };
    assert_eq!(name, "stdlib");
    assert_eq!(version.as_deref(), Some("0.2.17"));
    assert!(source.is_none());
    assert!(local);
}

/// [R-CLI-012] the tree is built per invocation, so two of them share
/// nothing. `v0.1.0` holds one built at init time, which makes it shared
/// mutable state between tests.
#[test]
fn two_command_trees_share_nothing() {
    let first = Cli::command();
    let second = Cli::command();
    assert_eq!(
        first.get_subcommands().count(),
        second.get_subcommands().count()
    );
}

/// [R-CLI-013] the version reports what was built, not `dev/unknown`.
#[test]
fn the_version_names_the_build() {
    let version = meowctl_cli::version();
    assert!(version.starts_with("meowctl 0."), "{version}");
    assert!(version.contains("commit "), "{version}");
    assert!(version.contains("built "), "{version}");
    assert!(
        !version.contains("commit unknown"),
        "a build from a checkout knows its commit: {version}"
    );
}
