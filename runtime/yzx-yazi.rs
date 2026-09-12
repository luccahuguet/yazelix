use std::{
    env,
    ffi::{OsStr, OsString},
    io::{self, Write},
    os::unix::{ffi::OsStringExt, process::CommandExt},
    path::{Path, PathBuf},
    process::{Command, ExitCode, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

const YZX_YAZI_CONFIG: &str = "@yzxYaziConfig@";
const YZX_YAZI_STARTUP_CONFIG: &str = "@yzxYaziStartupConfig@";
const YZX_YAZI_MATERIALIZER: &str = "@yzxYaziMaterializer@";
const YZX_OPEN: &str = "@yzxOpen@";
const YZX_YAZI_RETURN: &str = "@yzxYaziReturn@";
const YZX_ZELLIJ: &str = "@zellij@";
const YZX_HELIX: &str = "@yzxHelix@";
const YZX_EDITOR_LAUNCHER: &str = "@yzxEditor@";
const YZX_CONFIG: &str = "@yzxConfig@";
const FZF: &str = "@fzf@";
const ZOXIDE: &str = "@zoxide@";
const PATH_PREFIX: &str = "@pathPrefix@";
const PANE_ORCHESTRATOR: &str = "yazelix_pane_orchestrator";

#[derive(Debug, PartialEq, Eq)]
enum QuickAction {
    Open(OsString),
    Browse,
    Cancel,
}

#[derive(Debug, PartialEq, Eq)]
enum YaziAction {
    QuickSearch(PathBuf),
    Cancel,
}

struct ManagedEnv {
    state_dir: PathBuf,
    appearance_mode: String,
    yzx_open_log: String,
    editor: String,
    bridge_session_id: Option<OsString>,
    zellij_session: Option<OsString>,
}

impl ManagedEnv {
    fn load() -> io::Result<Self> {
        let editor = effective_editor_command(yzx_config_value("editor.command")?);
        Ok(Self {
            state_dir: state_dir(),
            appearance_mode: current_appearance_mode(yzx_config_value("appearance.mode")?),
            yzx_open_log: yzx_config_value("open.log_level")?,
            bridge_session_id: uses_helix_bridge(&editor).then(bridge_session_id),
            zellij_session: nonempty_env("ZELLIJ_SESSION_NAME"),
            editor,
        })
    }

    fn command(&self, program: impl AsRef<OsStr>, role: Option<&str>) -> Command {
        let mut command = Command::new(program);
        command
            .env("PATH", runtime_path())
            .env("YAZELIX_STATE_DIR", &self.state_dir)
            .env("YZX_OPEN", YZX_OPEN)
            .env("YZX_YAZI_RETURN", YZX_YAZI_RETURN)
            .env("YZX_ZELLIJ", YZX_ZELLIJ)
            .env("YAZELIX_EDITOR", &self.editor)
            .env("EDITOR", YZX_EDITOR_LAUNCHER)
            .env("VISUAL", YZX_EDITOR_LAUNCHER)
            .env("YZX_EDITOR", &self.editor)
            .env("GIT_EDITOR", YZX_EDITOR_LAUNCHER)
            .env("YZX_OPEN_LOG", &self.yzx_open_log);
        if let Some(role) = role {
            command.env("YZX_YAZI_ROLE", role);
        } else {
            command.env_remove("YZX_YAZI_ROLE");
        }
        if let Some(bridge_session_id) = &self.bridge_session_id {
            command.env("YAZELIX_HELIX_BRIDGE_SESSION_ID", bridge_session_id);
        }
        if let Some(session) = &self.zellij_session {
            command
                .env("YAZELIX_ZELLIJ_SESSION_NAME", session)
                .env("ZELLIJ_SESSION_NAME", "")
                .env("KITTY_WINDOW_ID", "1");
        }
        command
    }

    fn yazi_command(&self, yazi: &OsStr, role: Option<&str>, config: &Path) -> Command {
        let mut command = self.command(yazi, role);
        command.env("YAZI_CONFIG_HOME", config).env(
            "YZX_YAZI_STARSHIP_CONFIG",
            config.join("yazelix_starship.toml"),
        );
        command
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("yzx-yazi: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> io::Result<()> {
    let yazi = nonempty_env("YZX_YAZI_BIN").ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "YZX_YAZI_BIN is missing; launch managed Yazi through yzx",
        )
    })?;
    let mut args = env::args_os().skip(1).collect::<Vec<_>>();
    let role = take_role_flag(&mut args);
    if role == Some("startup-picker") {
        let result = run_startup_picker(&yazi, &args);
        let cleanup = close_cancelled_startup_picker_tab();
        return match (result, cleanup) {
            (Ok(()), Ok(())) => Ok(()),
            (Err(error), Ok(())) | (Ok(()), Err(error)) => Err(error),
            (Err(error), Err(cleanup)) => Err(io::Error::other(format!(
                "{error}; could not close startup picker tab: {cleanup}"
            ))),
        };
    }

    let managed = ManagedEnv::load()?;
    let yazi_config = yazi_config_home(
        Path::new(YZX_YAZI_CONFIG),
        &managed.state_dir,
        &managed.appearance_mode,
    )?;
    let mut command = managed.yazi_command(&yazi, role, &yazi_config);
    command.args(args);
    Err(command.exec())
}

fn run_startup_picker(yazi: &OsStr, args: &[OsString]) -> io::Result<()> {
    let mut browse_from = env::current_dir()?;
    let mut action = quick_picker()?;
    if action == QuickAction::Cancel {
        return Ok(());
    }
    let managed = ManagedEnv::load()?;
    let mut yazi_config = None;
    loop {
        match action {
            QuickAction::Cancel => return Ok(()),
            QuickAction::Open(directory) => {
                let status = managed
                    .command(YZX_OPEN, Some("startup-picker"))
                    .arg("--retarget-workspace")
                    .arg(directory)
                    .status()?;
                return if status.success() {
                    Ok(())
                } else {
                    Err(io::Error::other(format!(
                        "could not open quick-search selection: {status}"
                    )))
                };
            }
            QuickAction::Browse => {}
        }

        let config = match &yazi_config {
            Some(config) => config,
            None => yazi_config.insert(yazi_config_home(
                Path::new(YZX_YAZI_STARTUP_CONFIG),
                &managed.state_dir.join("startup-picker"),
                &managed.appearance_mode,
            )?),
        };
        match browse_yazi(&managed, yazi, args, config, &browse_from)? {
            YaziAction::QuickSearch(directory) => browse_from = directory,
            YaziAction::Cancel => return Ok(()),
        }
        action = quick_picker()?;
    }
}

fn quick_picker() -> io::Result<QuickAction> {
    let history = Command::new(ZOXIDE).args(["query", "--list"]).output()?;
    if !history.status.success() {
        return Err(io::Error::other(format!(
            "zoxide query failed: {}",
            trim_output(&[history.stdout, history.stderr].concat())
        )));
    }
    let enter_binding = if history.stdout.is_empty() {
        "--bind=enter:ignore,ctrl-z:ignore,btab:up"
    } else {
        "--bind=enter:accept-non-empty,ctrl-z:ignore,btab:up"
    };
    let mut child = Command::new(FZF)
        .args([
            "--exact",
            "--no-sort",
            enter_binding,
            "--cycle",
            "--keep-right",
            "--info=inline",
            "--layout=reverse",
            "--tabstop=1",
            "--border=none",
            "--expect=tab",
            "--print0",
            "--prompt=Quick search > ",
            "--footer=Enter Open · Tab Browse with Yazi · Esc/Ctrl+C Cancel",
            "--footer-border=none",
            "--color=footer:-1",
        ])
        .env_remove("FZF_DEFAULT_OPTS")
        .env_remove("FZF_DEFAULT_OPTS_FILE")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;
    if let Err(error) = child.stdin.take().unwrap().write_all(&history.stdout)
        && error.kind() != io::ErrorKind::BrokenPipe
    {
        let _ = child.kill();
        let _ = child.wait();
        return Err(error);
    }
    let output = child.wait_with_output()?;
    parse_quick_picker(output.status.code(), &output.stdout)
}

fn browse_yazi(
    managed: &ManagedEnv,
    yazi: &OsStr,
    args: &[OsString],
    config: &Path,
    browse_from: &Path,
) -> io::Result<YaziAction> {
    let output = managed
        .yazi_command(yazi, Some("startup-picker"), config)
        .args(["--cwd-file", "/dev/stdout", "--"])
        .arg(browse_from)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .output()?;
    parse_yazi_result(output.status.code(), &output.stdout)
}

fn close_cancelled_startup_picker_tab() -> io::Result<()> {
    let Some(pane_id) = nonempty_env("ZELLIJ_PANE_ID") else {
        return Ok(());
    };
    let mut command = Command::new(YZX_ZELLIJ);
    if let Some(session) =
        nonempty_env("ZELLIJ_SESSION_NAME").or_else(|| nonempty_env("YAZELIX_ZELLIJ_SESSION_NAME"))
    {
        command.env("ZELLIJ_SESSION_NAME", session);
    }
    let output = command
        .args([
            "action",
            "pipe",
            "--plugin",
            PANE_ORCHESTRATOR,
            "--name",
            "close_startup_picker_tab",
            "--",
        ])
        .arg(pane_id)
        .output()?;
    let response = trim_output(&output.stdout);
    if output.status.success() && response == "ok" {
        return Ok(());
    }
    Err(io::Error::other(trim_output(
        &[output.stdout, output.stderr].concat(),
    )))
}

fn take_role_flag(args: &mut Vec<OsString>) -> Option<&'static str> {
    let role = match args.first().map(OsString::as_os_str) {
        Some(arg) if arg == OsStr::new("--yzx-workspace-popup") => "workspace-popup",
        Some(arg) if arg == OsStr::new("--yzx-startup-picker") => "startup-picker",
        _ => return None,
    };
    args.remove(0);
    Some(role)
}

fn yazi_config_home(
    packaged: &Path,
    state_dir: &Path,
    appearance_mode: &str,
) -> io::Result<PathBuf> {
    let Some(user_yazi) = config_home().map(|path| path.join("yazi")) else {
        return Ok(packaged.into());
    };
    let output = Command::new(YZX_YAZI_MATERIALIZER)
        .args([packaged, &user_yazi, state_dir])
        .arg(appearance_mode)
        .output()?;
    if !output.status.success() {
        return Err(io::Error::other(trim_output(
            &[output.stdout, output.stderr].concat(),
        )));
    }
    Ok(PathBuf::from(trim_output(&output.stdout)))
}

fn yzx_config_value(path: &str) -> io::Result<String> {
    let output = Command::new(YZX_CONFIG).arg("--get").arg(path).output()?;
    if output.status.success() {
        return Ok(trim_output(&output.stdout));
    }
    Err(io::Error::other(trim_output(
        &[output.stdout, output.stderr].concat(),
    )))
}

fn current_appearance_mode(configured: String) -> String {
    let session_mode = nonempty_env("YZX_APPEARANCE_MODE");
    select_appearance_mode(
        configured,
        session_mode.as_deref(),
        nonempty_env("YZX_APPEARANCE_LIVE").as_deref() == Some(OsStr::new("1")),
    )
}

fn select_appearance_mode(configured: String, session_mode: Option<&OsStr>, live: bool) -> String {
    if !live {
        if let Some(mode @ ("dark" | "light")) = session_mode.and_then(OsStr::to_str) {
            return mode.to_string();
        }
    }
    configured
}

fn effective_editor_command(command: String) -> String {
    if matches!(command.as_str(), "yzx-hx" | "hx") {
        YZX_HELIX.to_string()
    } else {
        command
    }
}

fn config_home() -> Option<PathBuf> {
    nonempty_env("YAZELIX_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| nonempty_env("XDG_CONFIG_HOME").map(|path| PathBuf::from(path).join("yazelix")))
        .or_else(|| nonempty_env("HOME").map(|path| PathBuf::from(path).join(".config/yazelix")))
}

fn state_dir() -> PathBuf {
    nonempty_env("YAZELIX_STATE_DIR")
        .map(PathBuf::from)
        .or_else(|| nonempty_env("XDG_DATA_HOME").map(|path| PathBuf::from(path).join("yazelix")))
        .or_else(|| {
            nonempty_env("HOME").map(|path| PathBuf::from(path).join(".local/share/yazelix"))
        })
        .unwrap_or_else(|| PathBuf::from("/tmp/yazelix"))
}

fn bridge_session_id() -> OsString {
    nonempty_env("YAZELIX_HELIX_BRIDGE_SESSION_ID").unwrap_or_else(|| {
        OsString::from(format!(
            "yzx-helper-{}-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|duration| duration.as_secs())
                .unwrap_or_default(),
            std::process::id()
        ))
    })
}

fn uses_helix_bridge(command: &str) -> bool {
    command == YZX_HELIX || Path::new(command).file_name() == Some(OsStr::new("yzx-hx"))
}

fn runtime_path() -> OsString {
    match nonempty_env("PATH") {
        Some(path) => {
            let mut merged = OsString::from(PATH_PREFIX);
            merged.push(":");
            merged.push(path);
            merged
        }
        None => PATH_PREFIX.into(),
    }
}

fn nonempty_env(name: &str) -> Option<OsString> {
    env::var_os(name).filter(|value| !value.is_empty())
}

fn trim_output(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).trim().to_owned()
}

fn parse_quick_picker(status: Option<i32>, output: &[u8]) -> io::Result<QuickAction> {
    if status == Some(130) {
        return Ok(QuickAction::Cancel);
    }
    let fields = output.split(|byte| *byte == 0).collect::<Vec<_>>();
    match (status, fields.as_slice()) {
        (Some(0 | 1), [b"tab", b""] | [b"tab", _, b""]) => Ok(QuickAction::Browse),
        (Some(0), [b"", directory, b""]) if !directory.is_empty() => {
            Ok(QuickAction::Open(OsString::from_vec(directory.to_vec())))
        }
        (Some(0), _) => Err(io::Error::other(
            "quick search returned an invalid selection",
        )),
        _ => Err(io::Error::other(format!(
            "quick search exited with status {}",
            status
                .map(|code| code.to_string())
                .unwrap_or_else(|| "signal".into())
        ))),
    }
}

fn parse_yazi_result(status: Option<i32>, output: &[u8]) -> io::Result<YaziAction> {
    match status {
        Some(10) if !output.is_empty() => Ok(YaziAction::QuickSearch(PathBuf::from(
            OsString::from_vec(output.to_vec()),
        ))),
        Some(10) => Err(io::Error::other(
            "Yazi returned to quick search without a directory",
        )),
        Some(0 | 130) => Ok(YaziAction::Cancel),
        _ => Err(io::Error::other(format!(
            "Yazi exited with status {}",
            status
                .map(|code| code.to_string())
                .unwrap_or_else(|| "signal".into())
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn managed_helix_names_map_to_packaged_editor_while_host_commands_pass_through() {
        assert_eq!(effective_editor_command("yzx-hx".to_string()), YZX_HELIX);
        assert_eq!(effective_editor_command("hx".to_string()), YZX_HELIX);
        assert_eq!(effective_editor_command("nvim".to_string()), "nvim");
        assert!(uses_helix_bridge(YZX_HELIX));
        assert!(uses_helix_bridge("/nix/store/example/bin/yzx-hx"));
        assert!(!uses_helix_bridge("nvim"));
    }

    #[test]
    fn private_role_flags_are_removed_before_managed_yazi_launch() {
        let mut popup = vec![
            OsString::from("--yzx-workspace-popup"),
            OsString::from("/workspace with spaces"),
        ];
        assert_eq!(take_role_flag(&mut popup), Some("workspace-popup"));
        assert_eq!(popup, [OsString::from("/workspace with spaces")]);

        let mut picker = vec![OsString::from("--yzx-startup-picker")];
        assert_eq!(take_role_flag(&mut picker), Some("startup-picker"));
        assert!(picker.is_empty());

        let mut ordinary = vec![OsString::from("/workspace"), OsString::from("--debug")];
        assert_eq!(take_role_flag(&mut ordinary), None);
        assert_eq!(
            ordinary,
            [OsString::from("/workspace"), OsString::from("--debug")]
        );
    }

    #[test]
    fn read_only_session_yazi_keeps_its_captured_appearance() {
        assert_eq!(
            select_appearance_mode("light".into(), Some(OsStr::new("dark")), false),
            "dark"
        );
        assert_eq!(
            select_appearance_mode("light".into(), Some(OsStr::new("dark")), true),
            "light"
        );
    }

    #[test]
    fn startup_quick_picker_distinguishes_selection_browse_cancel_and_invalid_output() {
        assert_eq!(
            parse_quick_picker(Some(0), b"\0/workspace\xff\0").unwrap(),
            QuickAction::Open(OsString::from_vec(b"/workspace\xff".to_vec()))
        );
        assert_eq!(
            parse_quick_picker(Some(0), b"tab\0/workspace\0").unwrap(),
            QuickAction::Browse
        );
        assert_eq!(
            parse_quick_picker(Some(1), b"tab\0").unwrap(),
            QuickAction::Browse
        );
        assert_eq!(
            parse_quick_picker(Some(130), b"").unwrap(),
            QuickAction::Cancel
        );
        assert!(parse_quick_picker(Some(0), b"").is_err());
    }

    #[test]
    fn startup_yazi_distinguishes_return_cancel_and_failure() {
        assert_eq!(
            parse_yazi_result(Some(10), b"/workspace\xff").unwrap(),
            YaziAction::QuickSearch(PathBuf::from(OsString::from_vec(
                b"/workspace\xff".to_vec()
            )))
        );
        assert_eq!(parse_yazi_result(Some(0), b"").unwrap(), YaziAction::Cancel);
        assert_eq!(
            parse_yazi_result(Some(130), b"").unwrap(),
            YaziAction::Cancel
        );
        assert!(parse_yazi_result(Some(10), b"").is_err());
        assert!(parse_yazi_result(Some(2), b"").is_err());
    }
}
