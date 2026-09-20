//! What a hook can reach through `ctx`, and what happens when it reaches.
//!
//! The surface is frozen at what `v0.1.0` exposes: a component written for it
//! has to run unchanged. So the first tests here are about the names and the
//! shapes, and the rest are about what each one does.

// `clippy.toml` exempts a function carrying `#[test]`; a helper in a test
// binary is test code by construction.
#![allow(clippy::expect_used)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use meowctl_common::{Event, Phase};
use meowctl_ctx::{Capabilities, Ctx, Effects, Restricted, Surface};
use meowctl_exec::{ScriptedExecutor, ScriptedRun};
use meowctl_fs::{FileSystem, MemFs};
use meowctl_net::ScriptedHttp;
use meowctl_starlark::{Evaluator, NoLoader, Platform, StarlarkError};
use meowctl_tui::Always;

/// An absolute home directory for the platform the test runs on.
///
/// `/home/u` is not absolute on Windows, and [R-CTX-012] refuses a path that
/// is not absolute after `~` expands, so a test that hard-coded a Unix path
/// would fail there for the wrong reason.
#[cfg(unix)]
const HOME: &str = "/home/u";
#[cfg(not(unix))]
const HOME: &str = r"C:\Users\u";

#[cfg(unix)]
const COMPONENT_DIR: &str = "/component";
#[cfg(not(unix))]
const COMPONENT_DIR: &str = r"C:\component";

#[cfg(unix)]
const STATE_DIR: &str = "/state";
#[cfg(not(unix))]
const STATE_DIR: &str = r"C:\state";

/// A path under the home directory, built the way the code under test builds
/// it so the two agree on the separator.
fn under_home(rest: &str) -> PathBuf {
    let mut path = PathBuf::from(HOME);
    for part in rest.split('/') {
        path.push(part);
    }
    path
}

/// A `ctx` over an in-memory world, and the pieces a test asserts on.
struct World {
    fs: Arc<MemFs>,
    events: Arc<Mutex<Vec<Event>>>,
    exec: Arc<ScriptedExecutor>,
}

fn build(runs: Vec<ScriptedRun>, responses: ScriptedHttp, phase: Phase) -> (Ctx, World) {
    let fs = Arc::new(MemFs::new());
    fs.create_dir_all(Path::new(HOME)).expect("home");
    fs.create_dir_all(Path::new(COMPONENT_DIR))
        .expect("component dir");
    fs.create_dir_all(Path::new(STATE_DIR)).expect("state dir");

    let exec = Arc::new(ScriptedExecutor::new(runs).with_path(["git"]));
    let events: Arc<Mutex<Vec<Event>>> = Arc::new(Mutex::new(Vec::new()));
    let recorder = Arc::clone(&events);

    let capabilities = Capabilities {
        home: PathBuf::from(HOME),
        dry_run: false,
        component_dir: PathBuf::from(COMPONENT_DIR),
        state_dir: PathBuf::from(STATE_DIR),
        // Set in a runtime hook phase and nowhere else, as the engine sets
        // it; see [R-CTX-002].
        shell: phase.is_runtime_hook().then(|| "fish".to_owned()),
        platform: Platform {
            os: "macos".to_owned(),
            ..Platform::default()
        },
        environment: BTreeMap::from([("EDITOR".to_owned(), "nvim".to_owned())]),
        phase,
        component: "neovim".to_owned(),
    };
    let effects = Effects {
        fs: Arc::clone(&fs) as Arc<dyn FileSystem + Send + Sync>,
        exec: Arc::clone(&exec) as Arc<dyn meowctl_exec::Executor + Send + Sync>,
        http: Arc::new(responses),
        interaction: Arc::new(Mutex::new(Always(true))),
        // No journal: these tests are about the surface, and journaling is
        // `meowctl-ops`' to prove. One test below turns it on.
        journal: None,
        events: Arc::new(Mutex::new(move |event| {
            recorder.lock().expect("the recorder").push(event);
        })),
    };

    (Ctx::new(capabilities, effects), World { fs, events, exec })
}

fn plain() -> (Ctx, World) {
    build(Vec::new(), ScriptedHttp::new(), Phase::Install)
}

/// Runs a hook with this `ctx` and returns what the evaluation did.
fn call(ctx: &Ctx, surface: Surface, body: &str) -> Result<(), StarlarkError> {
    let loader = NoLoader;
    let argument = Restricted::new(ctx.clone(), surface);
    let source = format!("def install(ctx):\n{body}\n");
    Evaluator::new(Platform::default(), &loader)
        .call_hook("component.star", &source, "install", &argument)
        .map(|_| ())
}

/// A Starlark string literal for a path, with backslashes escaped.
fn quoted(path: &str) -> String {
    format!("\"{}\"", path.replace('\\', "\\\\"))
}

fn indent(lines: &str) -> String {
    lines
        .lines()
        .map(|l| format!("    {l}"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// [R-CTX-010] and [R-CTX-001]: the names a component written for `v0.1.0`
/// reaches for. `dir(ctx)` is what a hook sees, and a missing name is a
/// component that stops working.
#[test]
fn every_attribute_v0_1_0_registers_is_there() {
    let (ctx, _) = plain();
    let names = [
        // The six properties.
        "home",
        "dry_run",
        "component_dir",
        "state_dir",
        "shell",
        "platform",
        // The twenty-four methods.
        "log",
        "env",
        "write_file",
        "append_file",
        "delete_file",
        "copy_file",
        "symlink",
        "remove_symlink",
        "link_file",
        "mkdir",
        "read_file",
        "file_exists",
        "list_dir",
        "run",
        "git_clone",
        "download",
        "defaults_write",
        "plist_set",
        "prompt",
        "emit",
        "add_path",
        "render",
        "render_file",
        "which",
    ];
    for name in names {
        call(&ctx, Surface::Full, &indent(&format!("ctx.{name}")))
            .unwrap_or_else(|e| panic!("ctx.{name} is missing: {e}"));
    }
}

/// [R-CTX-011] a typo has to read as a typo rather than as an internal error.
#[test]
fn an_unknown_attribute_reports_attribute_not_found() {
    let (ctx, _) = plain();
    let err = call(&ctx, Surface::Full, "    ctx.nonesuch").expect_err("should fail");
    assert!(err.to_string().contains("nonesuch"), "{err}");
}

/// [R-CTX-001], [R-CTX-002] and [R-CTX-003]: what each property carries,
/// including the two directories a component writes into -- its own source
/// directory, which `render_file` reads against, and its persistent state
/// directory, which survives the run.
#[test]
fn the_properties_carry_what_the_run_was_given() {
    let (ctx, _) = plain();
    let source = format!(
        r#"
if ctx.home != {home}: fail("home is " + ctx.home)
if ctx.dry_run: fail("dry_run")
if ctx.component_dir != {component}: fail("component_dir")
if ctx.state_dir != {state}: fail("state_dir")
if ctx.shell != None: fail("shell should be None outside shell.star")
if ctx.platform.os != "macos": fail("platform")
"#,
        home = quoted(HOME),
        component = quoted(COMPONENT_DIR),
        state = quoted(STATE_DIR),
    );
    call(&ctx, Surface::Full, &indent(source.trim())).expect("the properties read");
}

/// [R-CTX-020] and [R-CTX-043]: the three fields, and a non-zero exit being a
/// result the hook reads rather than a failure.
#[test]
fn run_returns_three_fields_and_does_not_fail_on_a_non_zero_exit() {
    let (ctx, world) = build(
        vec![ScriptedRun::fails("brew list git", 1, "not installed")],
        ScriptedHttp::new(),
        Phase::Install,
    );
    call(
        &ctx,
        Surface::Full,
        &indent(
            r#"
result = ctx.run("brew", ["list", "git"])
if result["exit_code"] != 1: fail("exit_code is " + str(result["exit_code"]))
if result["stderr"] != "not installed": fail("stderr")
if result["stdout"] != "": fail("stdout")
"#
            .trim(),
        ),
    )
    .expect("a non-zero exit is a result");
    assert_eq!(world.exec.ran(), ["brew list git"]);
}

/// [R-CTX-042] a component tests and then reads, and conflating the two turns
/// a test into an error.
#[test]
fn file_exists_answers_where_read_file_fails() {
    let (ctx, world) = plain();
    world.fs.seed(under_home(".config/there"), "content");
    call(
        &ctx,
        Surface::Full,
        &indent(
            r#"
if not ctx.file_exists("~/.config/there"): fail("it is there")
if ctx.file_exists("~/.config/absent"): fail("it is not there")
if ctx.read_file("~/.config/there") != "content": fail("contents")
"#
            .trim(),
        ),
    )
    .expect("the test and the read");

    let err = call(
        &ctx,
        Surface::Full,
        "    ctx.read_file(\"~/.config/absent\")",
    )
    .expect_err("reading nothing fails");
    assert!(err.to_string().contains("read_file"), "{err}");
}

/// [R-CTX-012] a hook has no defined working directory, so a relative path
/// resolves somewhere its author cannot predict.
#[test]
fn a_relative_path_is_refused_and_a_tilde_is_expanded() {
    let (ctx, world) = plain();
    call(
        &ctx,
        Surface::Full,
        "    ctx.write_file(\"~/.zshrc\", \"export A=1\\n\")",
    )
    .expect("the tilde expands");
    assert_eq!(
        world.fs.read(&under_home(".zshrc")).expect("written"),
        b"export A=1\n"
    );

    let err = call(
        &ctx,
        Surface::Full,
        "    ctx.write_file(\"rel/path\", \"x\")",
    )
    .expect_err("a relative path is refused");
    assert!(err.to_string().contains("relative"), "{err}");
}

/// [R-CTX-013] every mutation goes through an `Op`, and an `Op` announces
/// itself. A method that touched the filesystem directly would be silent here.
#[test]
fn every_mutation_announces_itself_as_an_operation() {
    let (ctx, world) = plain();
    call(
        &ctx,
        Surface::Full,
        &indent(
            r#"
ctx.mkdir("~/.config/nvim")
ctx.write_file("~/.config/nvim/init.lua", "-- hi\n")
ctx.copy_file("~/.config/nvim/init.lua", "~/.config/nvim/copy.lua")
"#
            .trim(),
        ),
    )
    .expect("the mutations apply");

    let applied: Vec<String> = world
        .events
        .lock()
        .expect("events")
        .iter()
        .filter_map(|e| match e {
            Event::OpApplied { kind, .. } => Some(kind.to_string()),
            _ => None,
        })
        .collect();
    assert_eq!(applied, ["mkdir", "write_file", "copy_file"]);
}

/// [R-CTX-028] a component that re-runs with the same marker replaces its own
/// block rather than appending a second copy.
#[test]
fn appending_twice_with_one_marker_leaves_one_block() {
    let (ctx, world) = plain();
    let body = "    ctx.append_file(\"~/.zshrc\", \"export A=1\", marker = \"mine\")";
    call(&ctx, Surface::Full, body).expect("the first append");
    call(&ctx, Surface::Full, body).expect("the second append");

    let text =
        String::from_utf8(world.fs.read(&under_home(".zshrc")).expect("written")).expect("utf-8");
    assert_eq!(text.matches("export A=1").count(), 1, "{text}");
    assert_eq!(
        text.matches("mine").count(),
        2,
        "one begin and one end: {text}"
    );
}

/// [R-CTX-027] `render` substitutes and returns; it writes nothing.
#[test]
fn render_substitutes_and_render_file_reads_the_components_own_directory() {
    let (ctx, world) = plain();
    world.fs.seed(
        Path::new(COMPONENT_DIR).join("config.tmpl"),
        "editor = {{editor}}\n",
    );
    call(
        &ctx,
        Surface::Full,
        &indent(
            r#"
if ctx.render("hi {{who}}", {"who": "there"}) != "hi there": fail("render")
rendered = ctx.render_file("config.tmpl", {"editor": "nvim"})
if rendered != "editor = nvim\n": fail("render_file gave " + rendered)
"#
            .trim(),
        ),
    )
    .expect("both render");
}

/// [R-CTX-027] a variable that is not a string would otherwise be substituted
/// with whatever Starlark's formatting produces.
#[test]
fn a_non_string_variable_is_refused() {
    let (ctx, _) = plain();
    let err = call(&ctx, Surface::Full, "    ctx.render(\"{{n}}\", {\"n\": 3})")
        .expect_err("should refuse");
    assert!(err.to_string().contains("vars"), "{err}");
}

/// [R-CTX-024] stdout in any other phase corrupts a piped run.
#[test]
fn emit_speaks_only_in_a_runtime_hook_phase() {
    let (shell_ctx, shell_world) = build(Vec::new(), ScriptedHttp::new(), Phase::Shell);
    call(&shell_ctx, Surface::Shell, "    ctx.emit(\"export A=1\")").expect("emit");
    assert!(
        shell_world
            .events
            .lock()
            .expect("events")
            .iter()
            .any(|e| matches!(e, Event::ShellLine { line } if line == "export A=1")),
        "the line was emitted"
    );

    let (install_ctx, install_world) = plain();
    call(&install_ctx, Surface::Full, "    ctx.emit(\"export A=1\")").expect("emit");
    assert!(
        !install_world
            .events
            .lock()
            .expect("events")
            .iter()
            .any(|e| matches!(e, Event::ShellLine { .. })),
        "install is not a runtime hook phase"
    );
}

/// [R-CTX-030] and [R-CTX-032]: a check that writes is a check with a side
/// effect. `v0.1.0` has the machinery for this restriction and wires none of
/// it up.
///
/// Enforced by the value rather than by a check inside each method, which is
/// what the failure says: attribute-not-found, not "this phase may not
/// write". A method that guarded itself would have to remember to, and a
/// twenty-fifth method would forget.
#[test]
fn a_read_only_surface_has_no_mutating_method() {
    let (ctx, _) = build(Vec::new(), ScriptedHttp::new(), Phase::Verify);
    for name in [
        "write_file",
        "append_file",
        "delete_file",
        "copy_file",
        "symlink",
        "remove_symlink",
        "link_file",
        "mkdir",
        "git_clone",
        "download",
        "defaults_write",
        "plist_set",
        "add_path",
    ] {
        let err = call(&ctx, Surface::ReadOnly, &indent(&format!("ctx.{name}")))
            .expect_err(&format!("ctx.{name} should not be available"));
        assert!(err.to_string().contains(name), "{err}");
    }

    // And everything that only reads is still there, or a check could not
    // check anything.
    for name in [
        "read_file",
        "file_exists",
        "list_dir",
        "run",
        "which",
        "log",
    ] {
        call(&ctx, Surface::ReadOnly, &indent(&format!("ctx.{name}")))
            .unwrap_or_else(|e| panic!("ctx.{name} should be available: {e}"));
    }
}

/// [R-CTX-031] a shell hook runs on every shell spawn and must have no
/// persistent effect beyond what it emits.
#[test]
fn the_shell_surface_is_the_eight_attributes_and_nothing_else() {
    let (ctx, _) = build(Vec::new(), ScriptedHttp::new(), Phase::Shell);
    for allowed in [
        "emit",
        "file_exists",
        "list_dir",
        "platform",
        "read_file",
        "run",
        "shell",
        "state_dir",
    ] {
        call(&ctx, Surface::Shell, &indent(&format!("ctx.{allowed}")))
            .unwrap_or_else(|e| panic!("ctx.{allowed} should be available: {e}"));
    }
    for refused in ["write_file", "log", "env", "which", "home", "dry_run"] {
        let err = call(&ctx, Surface::Shell, &indent(&format!("ctx.{refused}")))
            .expect_err(&format!("ctx.{refused} should not be available"));
        assert!(err.to_string().contains(refused), "{err}");
    }
}

/// [R-CTX-044] a mistyped path must not delete something real.
#[test]
fn remove_symlink_refuses_a_regular_file() {
    let (ctx, world) = plain();
    world.fs.seed(under_home("real"), "not a link");
    let err =
        call(&ctx, Surface::Full, "    ctx.remove_symlink(\"~/real\")").expect_err("should refuse");
    assert!(err.to_string().contains("not a symlink"), "{err}");
    assert!(world.fs.read(&under_home("real")).is_ok(), "still there");
}

/// [R-CTX-021] asking whether a tool is installed is a question, and a
/// question's answer can be no.
#[test]
fn which_answers_none_rather_than_failing() {
    let (ctx, _) = plain();
    call(
        &ctx,
        Surface::Full,
        &indent(
            r#"
if ctx.which("git") == None: fail("git is on the scripted path")
if ctx.which("nonesuch") != None: fail("nonesuch is not")
"#
            .trim(),
        ),
    )
    .expect("both answers");
}

/// [R-CTX-023] a checksum is checked before anything is written, for the
/// reason [R-MODULE-030] gives: a check afterwards has already written.
#[test]
fn download_verifies_before_it_writes() {
    let body = b"payload".to_vec();
    let good = meowctl_common::Integrity::compute(&body).to_string();
    let (ctx, world) = build(
        Vec::new(),
        ScriptedHttp::new().with("https://example.invalid/f", body),
        Phase::Install,
    );

    let err = call(
        &ctx,
        Surface::Full,
        &indent(&format!(
            "ctx.download(\"https://example.invalid/f\", \"~/f\", checksum = \"sha384-{}\")",
            "A".repeat(64)
        )),
    )
    .expect_err("the checksum does not match");
    assert!(err.to_string().contains("expected"), "{err}");
    assert!(
        world.fs.read(&under_home("f")).is_err(),
        "nothing was written"
    );

    call(
        &ctx,
        Surface::Full,
        &indent(&format!(
            "ctx.download(\"https://example.invalid/f\", \"~/f\", checksum = \"{good}\")"
        )),
    )
    .expect("the matching checksum");
    assert_eq!(
        world.fs.read(&under_home("f")).expect("written"),
        b"payload"
    );
}

/// [R-CTX-022] cloning is running `git`, not implementing a fetch.
#[test]
fn git_clone_runs_git() {
    let (ctx, world) = build(
        vec![ScriptedRun::ok(
            format!(
                "git clone --branch v1 https://h/r {}",
                under_home("r").display()
            ),
            "",
        )],
        ScriptedHttp::new(),
        Phase::Install,
    );
    call(
        &ctx,
        Surface::Full,
        "    ctx.git_clone(\"https://h/r\", \"~/r\", ref = \"v1\")",
    )
    .expect("the clone runs");
    assert_eq!(
        world.exec.ran(),
        [format!(
            "git clone --branch v1 https://h/r {}",
            under_home("r").display()
        )]
    );
}

/// [R-CTX-025] the name suggests a shell statement and it is not one: the
/// standard library calls it so the next `ctx.run` finds a binary it just
/// installed.
#[test]
fn add_path_reports_the_directory_rather_than_emitting_a_statement() {
    let (ctx, world) = plain();
    call(&ctx, Surface::Full, "    ctx.add_path(\"/opt/bin\")").expect("add_path");
    let events = world.events.lock().expect("events");
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::PathPrepended { directory } if directory == "/opt/bin")),
        "{events:?}"
    );
    assert!(
        !events.iter().any(|e| matches!(e, Event::ShellLine { .. })),
        "it emits nothing: {events:?}"
    );
}

/// [R-CTX-014] no method asks whether this is a dry run. The property is
/// readable and changes nothing about what the methods do.
#[test]
fn dry_run_is_a_property_and_not_a_branch() {
    let (ctx, world) = plain();
    call(
        &ctx,
        Surface::Full,
        &indent(
            r#"
if ctx.dry_run: fail("this run is not a dry run")
ctx.write_file("~/written", "x")
"#
            .trim(),
        ),
    )
    .expect("the write happens");
    assert!(world.fs.read(&under_home("written")).is_ok());
}

/// Nothing above needs this, and a component that loses `ctx.log` loses the
/// only way it has to say anything.
#[test]
fn log_and_env_report_what_they_were_given() {
    let (ctx, world) = plain();
    call(
        &ctx,
        Surface::Full,
        &indent(
            r#"
ctx.log("configuring")
if ctx.env("EDITOR") != "nvim": fail("env")
if ctx.env("ABSENT") != "": fail("an unset variable is empty")
"#
            .trim(),
        ),
    )
    .expect("both");
    assert!(
        world
            .events
            .lock()
            .expect("events")
            .iter()
            .any(|e| matches!(e, Event::Message { text, .. } if text == "configuring")),
        "the log reached the sink"
    );
}

/// [R-CTX-013] and [R-OPS-021]: a mutation is journaled before it is applied,
/// so an interrupted run has something to undo.
///
/// The journal is a real file because that is what `Journal` writes; the rest
/// of the world stays in memory.
#[test]
fn a_mutation_is_journaled_before_it_happens() {
    let temp = tempfile::tempdir().expect("a temporary directory");
    let journal_path = temp.path().join("journal.ndjson");

    let fs = Arc::new(MemFs::new());
    fs.create_dir_all(Path::new(HOME)).expect("home");
    let journal = meowctl_ops::Journal::open(&journal_path).expect("the journal opens");

    let capabilities = Capabilities {
        home: PathBuf::from(HOME),
        dry_run: false,
        component_dir: PathBuf::from(COMPONENT_DIR),
        state_dir: PathBuf::from(STATE_DIR),
        shell: None,
        platform: Platform::default(),
        environment: BTreeMap::new(),
        phase: Phase::Install,
        component: "neovim".to_owned(),
    };
    let effects = Effects {
        fs: Arc::clone(&fs) as Arc<dyn FileSystem + Send + Sync>,
        exec: Arc::new(ScriptedExecutor::new(Vec::new())),
        http: Arc::new(ScriptedHttp::new()),
        interaction: Arc::new(Mutex::new(Always(true))),
        journal: Some(Arc::new(Mutex::new(journal))),
        events: Arc::new(Mutex::new(|_| {})),
    };
    let ctx = Ctx::new(capabilities, effects);

    call(
        &ctx,
        Surface::Full,
        "    ctx.write_file(\"~/.zshrc\", \"a\")",
    )
    .expect("the write happens");

    let written = std::fs::read_to_string(&journal_path).expect("the journal was written");
    assert!(written.contains("\"kind\":\"write_file\""), "{written}");
    assert!(written.contains("\"component\":\"neovim\""), "{written}");
    assert!(written.contains("\"phase\":\"install\""), "{written}");
}

/// [R-OPS-032] a journal that cannot be appended to fails the operation.
///
/// An effect applied with no record of how to undo it is the state the
/// journal exists to prevent, so the write must not happen at all. The
/// journal is opened on a path and then the directory is taken away, which is
/// how a full disk looks from here.
#[test]
fn an_effect_whose_journal_cannot_be_written_does_not_happen() {
    let temp = tempfile::tempdir().expect("a temporary directory");
    let journal_path = temp.path().join("gone").join("journal.ndjson");
    std::fs::create_dir_all(journal_path.parent().expect("a parent")).expect("the directory");

    let fs = Arc::new(MemFs::new());
    fs.create_dir_all(Path::new(HOME)).expect("home");
    let journal = meowctl_ops::Journal::open(&journal_path).expect("the journal opens");

    // Taken away after opening, so the append is what fails rather than the
    // open.
    std::fs::remove_dir_all(journal_path.parent().expect("a parent")).expect("remove");

    let capabilities = Capabilities {
        home: PathBuf::from(HOME),
        dry_run: false,
        component_dir: PathBuf::from(COMPONENT_DIR),
        state_dir: PathBuf::from(STATE_DIR),
        shell: None,
        platform: Platform::default(),
        environment: BTreeMap::new(),
        phase: Phase::Install,
        component: "neovim".to_owned(),
    };
    let effects = Effects {
        fs: Arc::clone(&fs) as Arc<dyn FileSystem + Send + Sync>,
        exec: Arc::new(ScriptedExecutor::new(Vec::new())),
        http: Arc::new(ScriptedHttp::new()),
        interaction: Arc::new(Mutex::new(Always(true))),
        journal: Some(Arc::new(Mutex::new(journal))),
        events: Arc::new(Mutex::new(|_| {})),
    };
    let ctx = Ctx::new(capabilities, effects);

    call(
        &ctx,
        Surface::Full,
        "    ctx.write_file(\"~/.zshrc\", \"a\")",
    )
    .expect_err("an unrecordable effect must not be applied");

    assert!(
        fs.read(Path::new(HOME).join(".zshrc").as_path()).is_err(),
        "the file was written with no way to undo it"
    );
}

/// [R-COMMON-012] and [R-CTX-030]: which surface a phase gets follows from
/// whether the phase is read-only, and nothing else decides it.
#[test]
fn the_surface_follows_from_the_phase() {
    for phase in [
        Phase::InstallCheck,
        Phase::UpgradeCheck,
        Phase::UninstallCheck,
        Phase::Verify,
    ] {
        assert_eq!(Surface::for_phase(phase), Surface::ReadOnly, "{phase}");
    }
    for phase in [Phase::Install, Phase::Update, Phase::Uninstall] {
        assert_eq!(Surface::for_phase(phase), Surface::Full, "{phase}");
    }
}

/// [R-CTX-002] a component chooses between `set -gx` and `export` by reading
/// this, so it has to name the shell that will evaluate the line.
#[test]
fn shell_names_the_shell_in_a_runtime_hook_phase() {
    let (ctx, world) = build(Vec::new(), ScriptedHttp::new(), Phase::Shell);
    call(
        &ctx,
        Surface::Shell,
        "    ctx.emit(\"shell is \" + ctx.shell)",
    )
    .expect("the hook reads ctx.shell");

    assert!(
        world
            .events
            .lock()
            .expect("events")
            .iter()
            .any(|e| matches!(e, Event::ShellLine { line } if line == "shell is fish")),
        "ctx.shell named the shell"
    );
}

/// [R-CTX-002] `None` everywhere else, which is how a component tests whether
/// it is being asked to contribute to a shell at all.
#[test]
fn shell_is_none_outside_a_runtime_hook_phase() {
    let (ctx, _) = plain();
    call(
        &ctx,
        Surface::Full,
        "    if ctx.shell != None:\n        fail(\"ctx.shell was set\")",
    )
    .expect("ctx.shell is None in install");
}

/// [R-CTX-031] a shell hook runs on every shell spawn, so the surface it gets
/// carries nothing that could leave a trace behind.
#[test]
fn a_runtime_hook_reaches_only_the_eight_attributes() {
    let (ctx, _) = build(Vec::new(), ScriptedHttp::new(), Phase::Shell);

    // An absolute path on both platforms: [R-CTX-012] refuses anything else,
    // and a Windows run would fail on the refusal rather than on the surface.
    let somewhere = format!("ctx.file_exists({})", quoted(COMPONENT_DIR));
    for allowed in [
        "ctx.emit(\"x\")",
        somewhere.as_str(),
        "ctx.platform",
        "ctx.shell",
        "ctx.state_dir",
    ] {
        call(&ctx, Surface::Shell, &format!("    {allowed}"))
            .unwrap_or_else(|e| panic!("{allowed} should be reachable: {e}"));
    }

    let write = format!("ctx.write_file({}, \"b\")", quoted(COMPONENT_DIR));
    let link = format!(
        "ctx.symlink({}, {})",
        quoted(COMPONENT_DIR),
        quoted(STATE_DIR)
    );
    for refused in [write.as_str(), link.as_str()] {
        let err = call(&ctx, Surface::Shell, &format!("    {refused}"))
            .expect_err("a shell hook must not reach a mutating method");
        assert!(err.to_string().contains("ctx"), "{refused}: {err}");
    }
}

/// [R-CTX-040] a method called with the wrong type names the method and the
/// argument, because a component author reads this and has no source for the
/// binary that produced it.
#[test]
fn a_wrong_argument_type_names_the_method_and_the_argument() {
    let (ctx, _) = plain();
    let err = call(&ctx, Surface::Full, "    ctx.write_file(42, \"x\")")
        .expect_err("a number is not a path");

    let said = err.to_string();
    assert!(said.contains("write_file"), "{said}");
}

/// [R-CTX-041] an effect that fails fails the hook, which fails the component
/// and lets the run roll back. Swallowing it would leave a component
/// reporting success over a file that was never written.
#[test]
fn a_failed_effect_fails_the_hook() {
    let (ctx, _) = plain();
    // A directory that does not exist, which [R-FS-033] makes a failure on
    // every implementation.
    let err = call(
        &ctx,
        Surface::Full,
        &format!(
            "    ctx.write_file({}, \"x\")",
            quoted(&format!("{HOME}/missing/deeper/file"))
        ),
    )
    .expect_err("a write into nothing should fail");

    assert!(err.to_string().contains("missing"), "{err}");
}

/// [R-CTX-026] a prompt goes through the `Interaction` trait, so a run with
/// no terminal answers rather than blocking on a stdin nobody is typing at.
#[test]
fn a_prompt_goes_through_the_interaction_trait() {
    let (ctx, _) = plain();
    // `Always(true)` is what the world is built with, so a prompt answers
    // without a terminal. Reading stdin directly would hang here instead.
    call(&ctx, Surface::Full, "    ctx.prompt(\"go ahead?\")").expect("the prompt is answered");
}

/// [R-CTX-044] `remove_symlink` removes a symlink and refuses a file.
///
/// The guard is what stops a mistyped path from deleting something real: the
/// method's name says what it is for, and a component that reached a regular
/// file with it has made a mistake worth hearing about.
#[test]
fn remove_symlink_refuses_something_that_is_not_one() {
    let (ctx, world) = plain();
    let file = format!("{HOME}/an-ordinary-file");
    world
        .fs
        .write(Path::new(&file), b"not a link")
        .expect("the file");

    let err = call(
        &ctx,
        Surface::Full,
        &format!("    ctx.remove_symlink({})", quoted(&file)),
    )
    .expect_err("a file is not a symlink");
    assert!(err.to_string().contains("not a symlink"), "{err}");

    assert!(
        world.fs.entry(Path::new(&file)).expect("ask").is_some(),
        "the file was removed anyway"
    );
}

/// [R-CTX-020] `list_dir` answers with what is in the directory, sorted, and
/// with names rather than paths.
#[test]
fn list_dir_answers_with_the_names_in_the_directory() {
    let (ctx, world) = plain();
    let dir = format!("{HOME}/a-directory");
    world
        .fs
        .create_dir_all(Path::new(&dir))
        .expect("the directory");
    for name in ["b.star", "a.star"] {
        world
            .fs
            .write(Path::new(&format!("{dir}/{name}")), b"x")
            .expect("a file");
    }

    call(
        &ctx,
        Surface::Full,
        &format!(
            "    found = ctx.list_dir({})\n    if found != [\"a.star\", \"b.star\"]:\n        fail(\"listed \" + str(found))",
            quoted(&dir)
        ),
    )
    .expect("the listing is what is there");
}

/// An interaction that answers with what it was told to, so a test can tell
/// the answer from a placeholder.
struct Says(&'static str);

impl meowctl_tui::Interaction for Says {
    fn ask(&mut self, _question: &str) -> Result<String, meowctl_tui::InteractionError> {
        Ok(self.0.to_owned())
    }

    fn confirm(&mut self, _question: &str) -> Result<bool, meowctl_tui::InteractionError> {
        Ok(true)
    }
}

/// [R-CTX-026] `prompt` answers with what the `Interaction` gave it, so a
/// component that asks a question gets the answer rather than a placeholder.
///
/// `Always` answers with an empty string, which is also what a `prompt` that
/// forgot to ask would return -- so the double has to say something.
#[test]
fn prompt_answers_with_what_the_interaction_said() {
    let fs = Arc::new(MemFs::new());
    fs.create_dir_all(Path::new(HOME)).expect("home");
    let ctx = Ctx::new(
        Capabilities {
            home: PathBuf::from(HOME),
            dry_run: false,
            component_dir: PathBuf::from(COMPONENT_DIR),
            state_dir: PathBuf::from(STATE_DIR),
            shell: None,
            platform: Platform::default(),
            environment: BTreeMap::new(),
            phase: Phase::Install,
            component: "neovim".to_owned(),
        },
        Effects {
            fs: Arc::clone(&fs) as Arc<dyn FileSystem + Send + Sync>,
            exec: Arc::new(ScriptedExecutor::new(Vec::new())),
            http: Arc::new(ScriptedHttp::new()),
            interaction: Arc::new(Mutex::new(Says("the answer"))),
            journal: None,
            events: Arc::new(Mutex::new(|_| {})),
        },
    );

    call(
        &ctx,
        Surface::Full,
        "    said = ctx.prompt(\"go ahead?\")\n    if said != \"the answer\":\n        fail(\"the prompt said \" + str(said))",
    )
    .expect("the answer arrives");
}

/// [R-CTX-016] a value reaches a command as the shell spells it: `defaults
/// write` and `PlistBuddy` both reject Starlark's `True`.
#[test]
fn a_boolean_reaches_a_command_as_the_shell_spells_it() {
    let (ctx, world) = build(
        vec![ScriptedRun::ok(
            "defaults write com.apple.dock autohide -bool true",
            "",
        )],
        ScriptedHttp::new(),
        Phase::Install,
    );

    call(
        &ctx,
        Surface::Full,
        "    ctx.defaults_write(\"com.apple.dock\", \"autohide\", \"-bool\", True)",
    )
    .expect("True became true");

    assert_eq!(
        world.exec.ran(),
        ["defaults write com.apple.dock autohide -bool true"]
    );
}

/// [R-CTX-030] and [R-CTX-031]: the restricted surfaces answer `hasattr`
/// honestly, because a component that checks before calling would otherwise
/// be told a method is there and then refused.
#[test]
fn the_restricted_surfaces_do_not_claim_what_they_refuse() {
    let (read_only, _) = build(Vec::new(), ScriptedHttp::new(), Phase::Verify);
    call(
        &read_only,
        Surface::ReadOnly,
        "    if hasattr(ctx, \"write_file\"):\n        fail(\"a read-only ctx claims write_file\")\n    if not hasattr(ctx, \"read_file\"):\n        fail(\"a read-only ctx denies read_file\")",
    )
    .expect("the read-only surface is honest");

    let (shell, _) = build(Vec::new(), ScriptedHttp::new(), Phase::Shell);
    call(
        &shell,
        Surface::Shell,
        "    if hasattr(ctx, \"write_file\"):\n        fail(\"a shell ctx claims write_file\")\n    if not hasattr(ctx, \"emit\"):\n        fail(\"a shell ctx denies emit\")",
    )
    .expect("the shell surface is honest");
}

/// [R-CTX-001] and [R-CTX-031]: `dir(ctx)` lists what is there, which is how
/// a component author finds out what they have without reading the source of
/// a binary they do not have.
#[test]
fn dir_lists_the_surface_the_phase_gets() {
    let (full, _) = plain();
    call(
        &full,
        Surface::Full,
        "    names = dir(ctx)\n    for wanted in [\"home\", \"write_file\", \"run\", \"platform\"]:\n        if wanted not in names:\n            fail(wanted + \" is not in dir(ctx): \" + str(names))",
    )
    .expect("the full surface lists itself");

    let (shell, _) = build(Vec::new(), ScriptedHttp::new(), Phase::Shell);
    call(
        &shell,
        Surface::Shell,
        "    names = dir(ctx)\n    if \"write_file\" in names:\n        fail(\"a shell ctx lists write_file\")\n    for wanted in [\"emit\", \"shell\", \"state_dir\"]:\n        if wanted not in names:\n            fail(wanted + \" is not in dir(ctx): \" + str(names))",
    )
    .expect("the shell surface lists itself");
}

/// [R-CTX-015] `link_file` with no backup given puts the user's file at
/// `<name>.meowctl-backup` beside it, which is where `v0.1.0` puts it and
/// therefore where a user who has been through this before will look.
#[test]
fn a_displaced_file_goes_beside_itself_with_a_known_name() {
    let (ctx, world) = plain();
    let link = format!("{HOME}/.zshrc");
    let target = format!("{COMPONENT_DIR}/zshrc");
    world
        .fs
        .write(Path::new(&link), b"the user's own")
        .expect("their file");
    world
        .fs
        .write(Path::new(&target), b"ours")
        .expect("the component's file");

    call(
        &ctx,
        Surface::Full,
        &format!("    ctx.link_file({}, {})", quoted(&target), quoted(&link)),
    )
    .expect("the link");

    let backup = format!("{link}.meowctl-backup");
    assert_eq!(
        world.fs.read(Path::new(&backup)).expect("the backup"),
        b"the user's own",
        "the displaced file is not at {backup}"
    );
}
