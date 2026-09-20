//! Stopping a run with a signal.
//!
//! Driven through the binary because what [R-CLI-051] promises is that a
//! signal arriving at the process reaches the engine, and there is no signal
//! without a process. Unix only: the watcher is `signal-hook`, and Windows
//! has no `SIGINT` to send from another process.
#![cfg(unix)]
// `clippy.toml` exempts a function carrying `#[test]`; a helper in a test
// binary is test code by construction.
#![allow(clippy::expect_used)]

use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// A configuration whose first component waits, so the signal arrives while
/// the run is between components rather than before it starts.
fn sandbox() -> PathBuf {
    let root = std::env::temp_dir().join(format!("meowctl-interrupt-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let components = root.join("components");
    std::fs::create_dir_all(&components).expect("the sandbox");

    std::fs::write(
        root.join("init.star"),
        "component(\"slow\")\ncomponent(\"after\")\n",
    )
    .expect("init.star");
    std::fs::write(
        components.join("slow.star"),
        "def install(ctx):\n    ctx.run(\"sleep\", [\"2\"])\n",
    )
    .expect("the slow component");
    std::fs::write(
        components.join("after.star"),
        "after = [\"slow\"]\ndef install(ctx):\n    ctx.write_file(ctx.home + \"/after-ran\", \"x\")\n",
    )
    .expect("the second component");
    root
}

/// [R-CLI-051], [R-CLI-053], [R-CLI-054] and [R-ENGINE-062]: the run stops
/// rather than the process dying where it stood, the component that had not
/// started does not start, and the exit code says the command did not do what
/// was asked.
///
/// [R-CLI-014] is here too: the handler is installed by the binary, and a
/// machine where it could not be would kill the process on the first signal
/// instead of reaching any of this.
#[test]
fn an_interrupt_stops_the_run_between_components() {
    let root = sandbox();
    let child = Command::new(env!("CARGO_BIN_EXE_meowctl"))
        .args(["--config", &root.display().to_string(), "apply", "--force"])
        .env("HOME", &root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the binary starts");

    // Long enough that the first component is running, short enough that it
    // has not finished its sleep.
    std::thread::sleep(Duration::from_millis(900));
    // Through `kill` rather than `libc`, so the test adds no dependency to
    // the workspace for the sake of one signal.
    let sent = Command::new("kill")
        .args(["-INT", &child.id().to_string()])
        .status()
        .expect("kill runs");
    assert!(sent.success(), "the signal was not sent");

    let started = Instant::now();
    let output = child.wait_with_output().expect("the binary exits");
    assert!(
        started.elapsed() < Duration::from_secs(10),
        "the run did not stop"
    );

    assert!(
        !root.join("after-ran").exists(),
        "the component after the interrupt ran anyway"
    );

    let said = String::from_utf8_lossy(&output.stderr);
    assert!(said.contains("interrupted"), "{said}");
    assert_eq!(output.status.code(), Some(1), "{output:?}");
}
