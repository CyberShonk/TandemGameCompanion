#![cfg(unix)]

use std::fs;
use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const TEST_TIMEOUT: Duration = Duration::from_secs(8);

struct TestDirectory {
    path: PathBuf,
}

impl TestDirectory {
    fn new(name: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after the epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "tandem-lifecycle-{name}-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("test directory should be created");
        Self { path }
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

struct RunOutput {
    status: ExitStatus,
    stdout: String,
    stderr: String,
    elapsed: Duration,
}

fn write_script(root: &Path, name: &str, body: &str) -> PathBuf {
    let path = root.join(name);
    fs::write(&path, format!("#!/bin/sh\nset -eu\n{body}\n")).expect("script should be written");
    let mut permissions = fs::metadata(&path)
        .expect("script metadata should be readable")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&path, permissions).expect("script should be executable");
    path
}

fn write_config(root: &Path, contents: &str) -> PathBuf {
    let path = root.join("Tandem.toml");
    fs::write(&path, contents).expect("configuration should be written");
    path
}

fn run_tandem(root: &Path, input: Option<&str>, environment: &[(&str, &str)]) -> RunOutput {
    run_tandem_with_input_delay(root, input, Duration::ZERO, environment)
}

fn run_tandem_with_input_delay(
    root: &Path,
    input: Option<&str>,
    input_delay: Duration,
    environment: &[(&str, &str)],
) -> RunOutput {
    let config = root.join("Tandem.toml");
    let mut command = Command::new(env!("CARGO_BIN_EXE_tandem-game-companion"));
    command
        .arg("--config")
        .arg(config)
        .current_dir(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for (name, value) in environment {
        command.env(name, value);
    }

    let started = Instant::now();
    let mut child = command.spawn().expect("Tandem should start");
    if let Some(input) = input {
        thread::sleep(input_delay);
        child
            .stdin
            .as_mut()
            .expect("Tandem stdin should be piped")
            .write_all(input.as_bytes())
            .expect("test input should be written");
    }
    drop(child.stdin.take());

    let status = loop {
        if let Some(status) = child.try_wait().expect("Tandem should remain inspectable") {
            break status;
        }
        if started.elapsed() >= TEST_TIMEOUT {
            let _ = child.kill();
            let _ = child.wait();
            panic!("Tandem exceeded the {TEST_TIMEOUT:?} integration-test timeout");
        }
        thread::sleep(Duration::from_millis(20));
    };

    let mut stdout = String::new();
    child
        .stdout
        .take()
        .expect("Tandem stdout should be piped")
        .read_to_string(&mut stdout)
        .expect("Tandem stdout should be readable");
    let mut stderr = String::new();
    child
        .stderr
        .take()
        .expect("Tandem stderr should be piped")
        .read_to_string(&mut stderr)
        .expect("Tandem stderr should be readable");

    RunOutput {
        status,
        stdout,
        stderr,
        elapsed: started.elapsed(),
    }
}

fn read_pid(path: &Path) -> u32 {
    fs::read_to_string(path)
        .expect("PID file should exist")
        .trim()
        .parse()
        .expect("PID file should contain a process ID")
}

fn process_exists(pid: u32) -> bool {
    PathBuf::from("/proc").join(pid.to_string()).exists()
}

fn wait_for_process_exit(pid: u32) -> bool {
    let deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < deadline {
        if !process_exists(pid) {
            return true;
        }
        thread::sleep(Duration::from_millis(20));
    }
    !process_exists(pid)
}

fn terminate_process(pid: u32) {
    let _ = Command::new("kill").arg(pid.to_string()).status();
    assert!(wait_for_process_exit(pid), "process {pid} should exit");
}

#[test]
fn launches_a_normal_game_and_returns_its_success() {
    let directory = TestDirectory::new("normal-game");
    write_script(&directory.path, "game.sh", "echo game-start >> events.txt");
    write_config(
        &directory.path,
        r#"config_version = 1
[game]
name = "Game"
path = "game.sh"
"#,
    );

    let output = run_tandem(&directory.path, None, &[]);

    assert!(output.status.success(), "stderr: {}", output.stderr);
    assert_eq!(
        fs::read_to_string(directory.path.join("events.txt")).unwrap(),
        "game-start\n"
    );
}

#[test]
fn preserves_a_nonzero_game_exit_code() {
    let directory = TestDirectory::new("game-exit-code");
    write_script(&directory.path, "game.sh", "exit 42");
    write_config(
        &directory.path,
        r#"config_version = 1
[game]
name = "Game"
path = "game.sh"
"#,
    );

    let output = run_tandem(&directory.path, None, &[]);

    assert_eq!(output.status.code(), Some(42), "stderr: {}", output.stderr);
}

#[test]
fn waits_for_user_confirmation_before_starting_the_game() {
    let directory = TestDirectory::new("user-confirmation");
    write_script(
        &directory.path,
        "trainer.sh",
        "echo trainer-start >> events.txt\necho $$ > trainer.pid\nwhile :; do sleep 1; done",
    );
    write_script(&directory.path, "game.sh", "echo game-start >> events.txt");
    write_config(
        &directory.path,
        r#"config_version = 1
[game]
name = "Game"
path = "game.sh"

[[tools]]
name = "Trainer"
path = "trainer.sh"
launch = "before-game"
before_game_wait = "user-confirmation"
required = true
close_when_game_exits = true
"#,
    );

    let output = run_tandem(&directory.path, Some("\n"), &[]);

    assert!(output.status.success(), "stderr: {}", output.stderr);
    assert_eq!(
        fs::read_to_string(directory.path.join("events.txt")).unwrap(),
        "trainer-start\ngame-start\n"
    );
    let pid = read_pid(&directory.path.join("trainer.pid"));
    assert!(wait_for_process_exit(pid), "trainer should be cleaned up");
}

#[test]
fn waits_for_a_successful_setup_tool_to_exit_before_the_game() {
    let directory = TestDirectory::new("tool-exit-success");
    write_script(
        &directory.path,
        "setup.sh",
        "echo setup-start >> events.txt\nsleep 0.1\necho setup-end >> events.txt",
    );
    write_script(&directory.path, "game.sh", "echo game-start >> events.txt");
    write_config(
        &directory.path,
        r#"config_version = 1
[game]
name = "Game"
path = "game.sh"

[[tools]]
name = "Setup"
path = "setup.sh"
launch = "before-game"
before_game_wait = "tool-exit"
required = true
"#,
    );

    let output = run_tandem(&directory.path, None, &[]);

    assert!(output.status.success(), "stderr: {}", output.stderr);
    assert_eq!(
        fs::read_to_string(directory.path.join("events.txt")).unwrap(),
        "setup-start\nsetup-end\ngame-start\n"
    );
}

#[test]
fn preserves_a_required_setup_tool_failure_and_does_not_start_the_game() {
    let directory = TestDirectory::new("tool-exit-failure");
    write_script(&directory.path, "setup.sh", "exit 23");
    write_script(
        &directory.path,
        "game.sh",
        "echo game-start > game-started.txt",
    );
    write_config(
        &directory.path,
        r#"config_version = 1
[game]
name = "Game"
path = "game.sh"

[[tools]]
name = "Setup"
path = "setup.sh"
launch = "before-game"
before_game_wait = "tool-exit"
required = true
"#,
    );

    let output = run_tandem(&directory.path, None, &[]);

    assert_eq!(output.status.code(), Some(23), "stderr: {}", output.stderr);
    assert!(!directory.path.join("game-started.txt").exists());
}

#[test]
fn cleans_up_a_started_tool_when_game_spawn_fails() {
    let directory = TestDirectory::new("game-spawn-failure");
    write_script(
        &directory.path,
        "tool.sh",
        "echo $$ > tool.pid\nwhile :; do sleep 1; done",
    );
    let game = directory.path.join("game.sh");
    fs::write(&game, "#!/missing/tandem-test-interpreter\n").unwrap();
    let mut permissions = fs::metadata(&game).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&game, permissions).unwrap();
    write_config(
        &directory.path,
        r#"config_version = 1
[game]
name = "Broken Game"
path = "game.sh"

[[tools]]
name = "Tool"
path = "tool.sh"
launch = "before-game"
before_game_wait = "user-confirmation"
required = true
close_when_game_exits = false
"#,
    );

    let output =
        run_tandem_with_input_delay(&directory.path, Some("\n"), Duration::from_millis(200), &[]);

    assert_eq!(output.status.code(), Some(1));
    let pid = read_pid(&directory.path.join("tool.pid"));
    assert!(wait_for_process_exit(pid), "tool should be cleaned up");
}

#[test]
fn leaves_a_persistent_tool_running_after_a_successful_game() {
    let directory = TestDirectory::new("persistent-tool");
    write_script(
        &directory.path,
        "tool.sh",
        "echo $$ > tool.pid\nwhile :; do sleep 1; done",
    );
    write_script(&directory.path, "game.sh", "exit 0");
    write_config(
        &directory.path,
        r#"config_version = 1
[game]
name = "Game"
path = "game.sh"

[[tools]]
name = "Persistent Tool"
path = "tool.sh"
launch = "before-game"
required = true
close_when_game_exits = false
"#,
    );

    let output = run_tandem(&directory.path, None, &[]);

    assert!(output.status.success(), "stderr: {}", output.stderr);
    let pid = read_pid(&directory.path.join("tool.pid"));
    assert!(process_exists(pid), "persistent tool should remain running");
    terminate_process(pid);
}

#[test]
fn skips_a_delayed_tool_when_the_game_exits_during_the_delay() {
    let directory = TestDirectory::new("interruptible-delay");
    write_script(&directory.path, "game.sh", "sleep 0.1");
    write_script(
        &directory.path,
        "late-tool.sh",
        "echo launched > late-tool-started.txt",
    );
    write_config(
        &directory.path,
        r#"config_version = 1
[game]
name = "Game"
path = "game.sh"

[[tools]]
name = "Late Tool"
path = "late-tool.sh"
launch = "after-game"
delay_ms = 3000
required = true
"#,
    );

    let output = run_tandem(&directory.path, None, &[]);

    assert!(output.status.success(), "stderr: {}", output.stderr);
    assert!(!directory.path.join("late-tool-started.txt").exists());
    assert!(
        output.elapsed < Duration::from_secs(2),
        "delay was not interrupted: {:?}",
        output.elapsed
    );
}

#[test]
fn child_protocol_output_cannot_spoof_the_guardian_or_hold_its_pipe_open() {
    let directory = TestDirectory::new("guardian-spoof");
    write_script(
        &directory.path,
        "spoof-tool.sh",
        "echo TANDEM_INTERNAL_GAME_PID=$$\necho $$ > spoof.pid\nwhile :; do sleep 1; done",
    );
    write_script(&directory.path, "game.sh", "sleep 0.1");
    write_config(
        &directory.path,
        r#"config_version = 1
[game]
name = "Game"
path = "game.sh"

[[tools]]
name = "Spoof Tool"
path = "spoof-tool.sh"
launch = "before-game"
required = true
close_when_game_exits = true
"#,
    );

    let output = run_tandem(&directory.path, None, &[]);

    assert!(output.status.success(), "stderr: {}", output.stderr);
    assert!(!output.stdout.contains("TANDEM_INTERNAL_GAME_PID="));
    assert!(
        fs::read_to_string(directory.path.join("Tandem.log"))
            .unwrap()
            .contains("TANDEM_INTERNAL_GAME_PID=")
    );
    let pid = read_pid(&directory.path.join("spoof.pid"));
    assert!(
        wait_for_process_exit(pid),
        "spoof tool should be cleaned up"
    );
}

#[test]
fn guardian_waits_for_the_game_but_preserves_worker_failure() {
    let directory = TestDirectory::new("worker-failure");
    write_script(
        &directory.path,
        "game.sh",
        "echo game-start >> events.txt\nsleep 0.3\necho game-end >> events.txt",
    );
    write_config(
        &directory.path,
        r#"config_version = 1
[game]
name = "Game"
path = "game.sh"
"#,
    );

    let output = run_tandem(
        &directory.path,
        None,
        &[("TANDEM_TEST_WORKER_EXIT_AFTER_GAME_START", "1")],
    );

    assert_eq!(output.status.code(), Some(1));
    assert!(output.elapsed >= Duration::from_millis(250));
    assert_eq!(
        fs::read_to_string(directory.path.join("events.txt")).unwrap(),
        "game-start\ngame-end\n"
    );
}
