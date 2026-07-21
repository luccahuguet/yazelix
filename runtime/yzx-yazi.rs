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
const YZX_YAZI_CONFIG: &str = "@yzxYaziConfig@";
const YZX_YAZI_MATERIALIZER: &str = "@yzxYaziMaterializer@";
const YZX_OPEN: &str = "@yzxOpen@";
const YZX_ZELLIJ: &str = "@zellij@";
const YZX_HELIX: &str = "@yzxHelix@";
const YZX_EDITOR_LAUNCHER: &str = "@yzxEditor@";
const YZX_CONFIG: &str = "@yzxConfig@";
const PATH_PREFIX: &str = "@pathPrefix@";

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
    let state_dir = state_dir()?;
    let yazi_config = yazi_config_home(&state_dir)?;
    let yzx_open_log = yzx_config_value("open.log_level")?;
    let editor = effective_editor_command(yzx_config_value("editor.command")?);
    let mut command = Command::new(YAZI);
    command
        .args(env::args_os().skip(1))
        .env("PATH", runtime_path())
        .env("YAZELIX_STATE_DIR", &state_dir)
        .env("YAZI_CONFIG_HOME", &yazi_config)
        .env(
            "YZX_YAZI_STARSHIP_CONFIG",
            yazi_config.join("yazelix_starship.toml"),
        )
        .env("YZX_OPEN", YZX_OPEN)
        .env("YZX_ZELLIJ", YZX_ZELLIJ)
        .env("YAZELIX_EDITOR", &editor)
        .env("EDITOR", YZX_EDITOR_LAUNCHER)
        .env("VISUAL", YZX_EDITOR_LAUNCHER)
        .env("YZX_EDITOR", &editor)
        .env("GIT_EDITOR", YZX_EDITOR_LAUNCHER)
        .env("YZX_OPEN_LOG", yzx_open_log);

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
    let user_yazi = config_home()?.join("yazi");
    let output = Command::new(YZX_YAZI_MATERIALIZER)
        .args([Path::new(YZX_YAZI_CONFIG), &user_yazi, state_dir])
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

fn effective_editor_command(command: String) -> String {
    if matches!(command.as_str(), "yzx-hx" | "hx") {
        YZX_HELIX.to_string()
    } else {
        command
    }
}

fn config_home() -> io::Result<PathBuf> {
    nonempty_env("YAZELIX_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| nonempty_env("XDG_CONFIG_HOME").map(|path| PathBuf::from(path).join("yazelix")))
        .or_else(|| nonempty_env("HOME").map(|path| PathBuf::from(path).join(".config/yazelix")))
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "HOME is required when YAZELIX_CONFIG_HOME and XDG_CONFIG_HOME are unset",
            )
        })
}

fn state_dir() -> io::Result<PathBuf> {
    nonempty_env("YAZELIX_STATE_DIR")
        .map(PathBuf::from)
        .or_else(|| {
            nonempty_env("XDG_RUNTIME_DIR").map(|path| PathBuf::from(path).join("yazelix"))
        })
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "YAZELIX_STATE_DIR or XDG_RUNTIME_DIR is required",
            )
        })
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
}
