use std::{
    env,
    ffi::{OsStr, OsString},
    io,
    os::unix::process::CommandExt,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
    time::{SystemTime, UNIX_EPOCH},
};

const YAZI: &str = "@yazi@";
const YZN_YAZI_CONFIG: &str = "@yznYaziConfig@";
const YZN_YAZI_MATERIALIZER: &str = "@yznYaziMaterializer@";
const YZN_OPEN: &str = "@yznOpen@";
const YZN_ZELLIJ: &str = "@zellij@";
const YZN_HELIX: &str = "@yznHelix@";
const YZN_EDITOR_LAUNCHER: &str = "@yznEditor@";
const YZN_CONFIG: &str = "@yznConfig@";
const PATH_PREFIX: &str = "@pathPrefix@";

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("yzn-yazi: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> io::Result<()> {
    let state_dir = state_dir();
    let yazi_config = yazi_config_home(&state_dir)?;
    let yzn_open_log = yzn_config_value("open.log_level")?;
    let editor = effective_editor_command(yzn_config_value("editor.command")?);
    let mut command = Command::new(YAZI);
    command
        .args(env::args_os().skip(1))
        .env("PATH", runtime_path())
        .env("YAZELIX_STATE_DIR", &state_dir)
        .env("YAZI_CONFIG_HOME", &yazi_config)
        .env(
            "YZN_YAZI_STARSHIP_CONFIG",
            yazi_config.join("yazelix_starship.toml"),
        )
        .env("YZN_OPEN", YZN_OPEN)
        .env("YZN_ZELLIJ", YZN_ZELLIJ)
        .env("YAZELIX_NEXT_EDITOR", &editor)
        .env("EDITOR", YZN_EDITOR_LAUNCHER)
        .env("VISUAL", YZN_EDITOR_LAUNCHER)
        .env("YZN_EDITOR", &editor)
        .env("GIT_EDITOR", YZN_EDITOR_LAUNCHER)
        .env("YZN_OPEN_LOG", yzn_open_log);

    if uses_helix_bridge(&editor) {
        command.env("YAZELIX_HELIX_BRIDGE_SESSION_ID", bridge_session_id());
    }

    if let Some(session) = nonempty_env("ZELLIJ_SESSION_NAME") {
        command
            .env("YAZELIX_ZELLIJ_SESSION_NAME", session)
            .env("ZELLIJ_SESSION_NAME", "")
            .env("KITTY_WINDOW_ID", "1");
    }

    Err(command.exec())
}

fn yazi_config_home(state_dir: &Path) -> io::Result<PathBuf> {
    let Some(user_yazi) = config_home().map(|path| path.join("yazi")) else {
        return Ok(YZN_YAZI_CONFIG.into());
    };
    let output = Command::new(YZN_YAZI_MATERIALIZER)
        .args([Path::new(YZN_YAZI_CONFIG), &user_yazi, state_dir])
        .output()?;
    if !output.status.success() {
        return Err(io::Error::other(trim_output(
            &[output.stdout, output.stderr].concat(),
        )));
    }
    Ok(PathBuf::from(trim_output(&output.stdout)))
}

fn yzn_config_value(path: &str) -> io::Result<String> {
    let output = Command::new(YZN_CONFIG).arg("--get").arg(path).output()?;
    if output.status.success() {
        return Ok(trim_output(&output.stdout));
    }
    Err(io::Error::other(trim_output(
        &[output.stdout, output.stderr].concat(),
    )))
}

fn effective_editor_command(command: String) -> String {
    if matches!(command.as_str(), "yzn-hx" | "hx") {
        YZN_HELIX.to_string()
    } else {
        command
    }
}

fn config_home() -> Option<PathBuf> {
    nonempty_env("YAZELIX_NEXT_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            nonempty_env("XDG_CONFIG_HOME").map(|path| PathBuf::from(path).join("yazelix-next"))
        })
        .or_else(|| {
            nonempty_env("HOME").map(|path| PathBuf::from(path).join(".config/yazelix-next"))
        })
}

fn state_dir() -> PathBuf {
    nonempty_env("YAZELIX_STATE_DIR")
        .map(PathBuf::from)
        .or_else(|| {
            nonempty_env("XDG_DATA_HOME").map(|path| PathBuf::from(path).join("yazelix-next"))
        })
        .or_else(|| {
            nonempty_env("HOME").map(|path| PathBuf::from(path).join(".local/share/yazelix-next"))
        })
        .unwrap_or_else(|| PathBuf::from("/tmp/yazelix-next"))
}

fn bridge_session_id() -> OsString {
    nonempty_env("YAZELIX_HELIX_BRIDGE_SESSION_ID").unwrap_or_else(|| {
        OsString::from(format!(
            "yzn-helper-{}-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|duration| duration.as_secs())
                .unwrap_or_default(),
            std::process::id()
        ))
    })
}

fn uses_helix_bridge(command: &str) -> bool {
    command == YZN_HELIX || Path::new(command).file_name() == Some(OsStr::new("yzn-hx"))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn managed_helix_names_map_to_packaged_editor_while_host_commands_pass_through() {
        assert_eq!(effective_editor_command("yzn-hx".to_string()), YZN_HELIX);
        assert_eq!(effective_editor_command("hx".to_string()), YZN_HELIX);
        assert_eq!(effective_editor_command("nvim".to_string()), "nvim");
        assert!(uses_helix_bridge(YZN_HELIX));
        assert!(uses_helix_bridge("/nix/store/example/bin/yzn-hx"));
        assert!(!uses_helix_bridge("nvim"));
    }
}
