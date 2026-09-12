use kinestra::{Recorder, Result, Size};
use std::{
    env, fs,
    io::{self, Write},
    path::Path,
    process::{Command, ExitCode, Stdio},
    thread,
    time::{Duration, Instant},
};

const TIMEOUT: Duration = Duration::from_secs(20);

fn panes(zellij: &Path, session: &str) -> Result<String> {
    let output = Command::new(zellij)
        .args(["-s", session, "action", "list-panes", "--all", "--json"])
        .output()?;
    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        if error.trim() == "There is no active session!" {
            return Ok("[]".into());
        }
        return Err(io::Error::other(error.into_owned()).into());
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn jq(json: &str, filter: &str) -> Result<bool> {
    let mut child = Command::new("jq")
        .args(["-e", filter])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()?;
    child.stdin.take().unwrap().write_all(json.as_bytes())?;
    Ok(child.wait()?.success())
}

fn wait_for_panes(
    recorder: &mut Recorder,
    zellij: &Path,
    session: &str,
    filter: &str,
) -> Result<()> {
    let deadline = Instant::now() + TIMEOUT;
    loop {
        let state = panes(zellij, session)?;
        if jq(&state, filter)? {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(
                io::Error::other(format!("pane state never matched {filter}: {state}")).into(),
            );
        }
        recorder.sleep(Duration::from_millis(100))?;
    }
}

fn wait_for_screen(
    recorder: &mut Recorder,
    zellij: &Path,
    session: &str,
    needle: &str,
) -> Result<()> {
    let deadline = Instant::now() + TIMEOUT;
    loop {
        let output = Command::new(zellij)
            .args(["-s", session, "action", "dump-screen"])
            .output()?;
        if output.status.success() && String::from_utf8_lossy(&output.stdout).contains(needle) {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(io::Error::other(format!("screen never contained {needle}")).into());
        }
        recorder.sleep(Duration::from_millis(100))?;
    }
}

fn wait_for_session_exit(zellij: &Path, session: &str) -> Result<()> {
    let deadline = Instant::now() + TIMEOUT;
    loop {
        let output = Command::new(zellij)
            .args(["list-sessions", "--no-formatting", "--short"])
            .output()?;
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            if error.trim() == "No active zellij sessions found." {
                return Ok(());
            }
            return Err(io::Error::other(error.into_owned()).into());
        }
        if !String::from_utf8_lossy(&output.stdout)
            .lines()
            .any(|name| name.trim() == session)
        {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(io::Error::other(format!("session {session} did not exit")).into());
        }
        thread::sleep(Duration::from_millis(100));
    }
}

fn send_key(zellij: &Path, session: &str, key: &str) -> Result<()> {
    let status = Command::new(zellij)
        .args(["-s", session, "action", "send-keys", key])
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!("could not send {key} to {session}")).into())
    }
}

fn launch(
    recorder: &mut Recorder,
    yzx: &Path,
    session: &str,
    cwd: &Path,
    home: &Path,
) -> Result<()> {
    recorder.launch(
        "nova-picker-cancel",
        Command::new("xterm")
            .current_dir(cwd)
            .env("HOME", home)
            .env("YAZELIX_CONFIG_HOME", home.join(".config/yazelix"))
            .env("YAZELIX_STATE_DIR", home.join(".local/share/yazelix"))
            .args(["-class", "nova-picker-cancel", "-e"])
            .arg(yzx)
            .args(["enter", "--session", session]),
    )
}

fn record(recorder: &mut Recorder) -> Result<()> {
    let yzx = env::var_os("YZX_BIN").expect("Nix supplies YZX_BIN");
    let zellij = env::var_os("ZELLIJ_BIN").expect("Nix supplies ZELLIJ_BIN");
    let yzx = Path::new(&yzx);
    let zellij = Path::new(&zellij);
    let sessions = ["nova-picker-cancel-later", "nova-picker-cancel-only"];
    let cleanup_exe = env::current_exe()?;
    for session in sessions {
        let _ = Command::new(zellij)
            .args(["kill-session", session])
            .status();
        let mut cleanup = Command::new(&cleanup_exe);
        cleanup.args(["--cleanup-session", session]);
        recorder.on_exit(cleanup);
    }

    let home = recorder.work().join("home");
    let picker_dir = recorder.work().join("picker");
    fs::create_dir_all(home.join(".config/yazelix"))?;
    fs::create_dir(&picker_dir)?;
    fs::write(
        home.join(".config/yazelix/config.toml"),
        "[welcome]\nenabled = false\n",
    )?;
    fs::write(picker_dir.join("target.txt"), "picker cancellation proof\n")?;

    recorder.display(Size::new(1200, 720)?, None)?;
    launch(recorder, yzx, sessions[0], &picker_dir, &home)?;
    wait_for_panes(
        recorder,
        zellij,
        sessions[0],
        r#"any(.[]; .title == "yazi_picker" and .is_focused)"#,
    )?;
    wait_for_screen(recorder, zellij, sessions[0], "target.txt")?;
    send_key(zellij, sessions[0], "End")?;
    send_key(zellij, sessions[0], "Enter")?;
    wait_for_panes(
        recorder,
        zellij,
        sessions[0],
        r#"any(.[]; .title == "editor" and .is_focused)"#,
    )?;

    let layout = yzx
        .parent()
        .and_then(Path::parent)
        .expect("yzx is installed under bin")
        .join("share/yazelix/layout.kdl");
    let status = Command::new(zellij)
        .args(["-s", sessions[0], "action", "new-tab", "--layout"])
        .arg(layout)
        .arg("--cwd")
        .arg(&picker_dir)
        .status()?;
    if !status.success() {
        return Err(io::Error::other("could not create the second tab").into());
    }
    wait_for_panes(
        recorder,
        zellij,
        sessions[0],
        r#"([.[].tab_position] | unique | length) == 2 and any(.[]; .title == "yazi_picker" and .is_focused)"#,
    )?;
    wait_for_screen(recorder, zellij, sessions[0], "target.txt")?;
    send_key(zellij, sessions[0], "q")?;
    wait_for_panes(
        recorder,
        zellij,
        sessions[0],
        r#"([.[].tab_position] | unique | length) == 1 and any(.[]; .title == "editor" and .is_focused)"#,
    )?;

    Command::new(zellij)
        .args(["kill-session", sessions[0]])
        .status()?;
    recorder.stop_app()?;

    launch(recorder, yzx, sessions[1], &picker_dir, &home)?;
    wait_for_panes(
        recorder,
        zellij,
        sessions[1],
        r#"any(.[]; .title == "yazi_picker" and .is_focused)"#,
    )?;
    wait_for_screen(recorder, zellij, sessions[1], "target.txt")?;
    send_key(zellij, sessions[1], "q")?;
    wait_for_session_exit(zellij, sessions[1])?;
    recorder.stop_app()
}

fn main() -> ExitCode {
    if env::args_os().nth(1).as_deref() == Some(std::ffi::OsStr::new("--cleanup-session")) {
        if let (Some(zellij), Some(session)) = (env::var_os("ZELLIJ_BIN"), env::args_os().nth(2)) {
            let _ = Command::new(zellij)
                .arg("kill-session")
                .arg(session)
                .status();
        }
        return ExitCode::SUCCESS;
    }
    kinestra::run(record)
}
