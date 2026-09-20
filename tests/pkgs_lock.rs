//! `pkgs.lock` and `pkgs.local.lock`, written by a real run.
//!
//! Driven through the binary because what [R-CONFIG-025] promises is about
//! which of two files a component's packages land in, and that depends on
//! which entry point declared it -- a split no library call can show.

// `clippy.toml` exempts a function carrying `#[test]`; a helper in a test
// binary is test code by construction.
#![allow(clippy::expect_used)]

use std::path::{Path, PathBuf};
use std::process::Command;

/// A package manager component, so `pkg()` has something to dispatch to.
const MANAGER: &str = "\
pm_name = \"fake\"
def install_pkg(ctx, name, version):
    pass
def uninstall_pkg(ctx, name):
    pass
def interrogate(ctx):
    return {}
";

/// A configuration with one shared component and one machine-local one.
fn sandbox(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("meowctl-pkgs-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let components = root.join("components");
    std::fs::create_dir_all(&components).expect("the sandbox");

    std::fs::write(
        root.join("init.star"),
        "component(\"fakepm\")\ncomponent(\"tool\")\n",
    )
    .expect("init.star");
    std::fs::write(root.join("local.star"), "component(\"localtool\")\n").expect("local.star");
    std::fs::write(components.join("fakepm.star"), MANAGER).expect("the manager");
    std::fs::write(
        components.join("tool.star"),
        "after = [\"fakepm\"]\npkg(\"fake\", \"ripgrep\", \"14.1.0\")\ndef install(ctx):\n    pass\n",
    )
    .expect("the shared component");
    std::fs::write(
        components.join("localtool.star"),
        "after = [\"fakepm\"]\npkg(\"fake\", \"bat\")\ndef install(ctx):\n    pass\n",
    )
    .expect("the local component");
    root
}

fn apply(root: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_meowctl"))
        .args(["--config", &root.display().to_string(), "apply", "--force"])
        .output()
        .expect("the binary runs")
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// [R-CONFIG-025] a component declared in `init.star` records into the shared
/// file, and one declared in `local.star` into the machine-local one. Getting
/// this backwards commits a machine's private packages to a shared
/// repository.
#[test]
fn the_declaring_file_decides_which_lock_records_the_package() {
    let root = sandbox("split");
    let output = apply(&root);
    assert!(output.status.success(), "{output:?}");

    let shared = read(&root.join("pkgs.lock"));
    assert!(shared.contains("ripgrep"), "{shared}");
    assert!(
        !shared.contains("bat"),
        "a local package reached the shared lock: {shared}"
    );

    let local = read(&root.join("pkgs.local.lock"));
    assert!(local.contains("bat"), "{local}");
    assert!(
        !local.contains("ripgrep"),
        "a shared package reached the local lock: {local}"
    );
}

/// [R-CONFIG-025] and [R-CONFIG-027]: the constraint is recorded as both the
/// requested and the installed version, which is what `appendPkgsLock`
/// writes -- nothing interrogates the manager for what it actually put down.
/// Asserted on the content rather than on the bytes, because these two are
/// written by a different encoder than `deps.lock` and are not byte-compared.
#[test]
fn the_constraint_is_recorded_as_both_versions() {
    let root = sandbox("versions");
    apply(&root);

    let shared = read(&root.join("pkgs.lock"));
    assert!(shared.contains("requested = \"14.1.0\""), "{shared}");
    assert!(shared.contains("installed = \"14.1.0\""), "{shared}");
}

/// [R-CONFIG-025] a package declared with no version records empty strings
/// rather than being left out. The entry is what says the package is managed.
#[test]
fn a_package_with_no_constraint_still_gets_an_entry() {
    let root = sandbox("noversion");
    apply(&root);

    let local = read(&root.join("pkgs.local.lock"));
    assert!(local.contains("[packages.fake.bat]"), "{local}");
}

/// [R-CONFIG-025] and [R-CONFIG-026]: merged into what is there, not
/// replacing it. A second component's packages must not remove the first's,
/// and an entry for a package nothing declares any more survives.
#[test]
fn a_second_run_keeps_what_the_first_recorded() {
    let root = sandbox("merge");
    apply(&root);

    // A package nothing declares any more, left by an earlier run.
    let path = root.join("pkgs.lock");
    let before = read(&path);
    std::fs::write(
        &path,
        before.replace(
            "[packages.fake.ripgrep]",
            "[packages.fake.antique]\n      requested = \"1.0\"\n      installed = \"1.0\"\n\n    [packages.fake.ripgrep]",
        ),
    )
    .expect("seed");

    apply(&root);
    let after = read(&path);
    assert!(
        after.contains("antique"),
        "the earlier entry was dropped: {after}"
    );
    assert!(after.contains("ripgrep"), "{after}");
}

/// [R-CONFIG-026] a run that installs nothing writes no file. `v0.1.0` skips
/// the write when nothing was pinned, and a configuration with no packages
/// should not grow an empty lock.
#[test]
fn a_configuration_with_no_packages_writes_no_lock() {
    let root = std::env::temp_dir().join(format!("meowctl-pkgs-none-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("components")).expect("the sandbox");
    std::fs::write(root.join("init.star"), "component(\"quiet\")\n").expect("init.star");
    std::fs::write(
        root.join("components").join("quiet.star"),
        "def install(ctx):\n    pass\n",
    )
    .expect("the component");

    assert!(apply(&root).status.success());
    assert!(
        !root.join("pkgs.lock").exists(),
        "an empty lock was written"
    );
    assert!(!root.join("pkgs.local.lock").exists());
}
