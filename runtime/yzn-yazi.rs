use std::{
    env,
    ffi::OsString,
    fs,
    io::{self, ErrorKind},
    os::{unix::fs::symlink, unix::process::CommandExt},
    path::{Path, PathBuf},
    process::{Command, ExitCode},
    time::{SystemTime, UNIX_EPOCH},
};

const YAZI: &str = "@yazi@";
const YZN_YAZI_CONFIG: &str = "@yznYaziConfig@";
const YZN_OPEN: &str = "@yznOpen@";
const YZN_ZELLIJ: &str = "@zellij@";
const YZN_HELIX: &str = "@yznHelix@";
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
    let yazi_config = match config_home()
        .map(|path| path.join("yazi"))
        .filter(|path| path.join("init.lua").exists() || path.join("keymap.toml").exists())
    {
        Some(user_yazi) => {
            materialize_user_config(&state_dir, &user_yazi, Path::new(YZN_YAZI_CONFIG))?
        }
        None => PathBuf::from(YZN_YAZI_CONFIG),
    };
    let yzn_open_log = yzn_config_value("open.log_level")?;
    let editor = effective_editor_command(yzn_config_value("editor.command")?);
    let mut command = Command::new(YAZI);
    command
        .args(env::args_os().skip(1))
        .env("PATH", runtime_path())
        .env("YAZELIX_STATE_DIR", &state_dir)
        .env("YAZELIX_HELIX_BRIDGE_SESSION_ID", bridge_session_id())
        .env("YAZI_CONFIG_HOME", &yazi_config)
        .env(
            "YZN_YAZI_STARSHIP_CONFIG",
            yazi_config.join("yazelix_starship.toml"),
        )
        .env("YZN_OPEN", YZN_OPEN)
        .env("YZN_ZELLIJ", YZN_ZELLIJ)
        .env("YAZELIX_NEXT_EDITOR", &editor)
        .env("EDITOR", &editor)
        .env("VISUAL", &editor)
        .env("YZN_EDITOR", &editor)
        .env("YZN_OPEN_LOG", yzn_open_log);

    if let Some(session) = nonempty_env("ZELLIJ_SESSION_NAME") {
        command
            .env("YAZELIX_ZELLIJ_SESSION_NAME", session)
            .env("ZELLIJ_SESSION_NAME", "")
            .env("KITTY_WINDOW_ID", "1");
    }

    Err(command.exec())
}

fn materialize_user_config(
    state_dir: &Path,
    user_yazi: &Path,
    packaged_yazi: &Path,
) -> io::Result<PathBuf> {
    let user_init = user_yazi.join("init.lua");
    let user_keymap = user_yazi.join("keymap.toml");
    let runtime_yazi = state_dir.join("yazi");
    remove_any(&runtime_yazi)?;
    fs::create_dir_all(&runtime_yazi)?;
    for path in ["yazi.toml", "yazelix_starship.toml"] {
        symlink(packaged_yazi.join(path), runtime_yazi.join(path))?;
    }
    write_layered_config(
        &packaged_yazi.join("init.lua"),
        &user_init,
        &runtime_yazi.join("init.lua"),
        "-- Yazelix Next user init.lua",
    )?;
    write_layered_config(
        &packaged_yazi.join("keymap.toml"),
        &user_keymap,
        &runtime_yazi.join("keymap.toml"),
        "# Yazelix Next user keymap.toml",
    )?;
    let runtime_plugins = runtime_yazi.join("plugins");
    fs::create_dir(&runtime_plugins)?;
    for entry in fs::read_dir(packaged_yazi.join("plugins"))? {
        let entry = entry?;
        symlink(entry.path(), runtime_plugins.join(entry.file_name()))?;
    }
    if user_init.exists() {
        overlay_user_plugins(&user_yazi.join("plugins"), &runtime_plugins)?;
    }
    Ok(runtime_yazi)
}

fn write_layered_config(
    packaged: &Path,
    user: &Path,
    target: &Path,
    marker: &str,
) -> io::Result<()> {
    let mut contents = fs::read_to_string(packaged)?;
    if user.exists() {
        if !user.is_file() {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                format!("cannot read {}", user.display()),
            ));
        }
        contents.push_str("\n");
        contents.push_str(marker);
        contents.push_str("\n");
        contents.push_str(&fs::read_to_string(user)?);
    }
    fs::write(target, contents)
}

fn overlay_user_plugins(user_plugins: &Path, runtime_plugins: &Path) -> io::Result<()> {
    if !user_plugins.exists() {
        return Ok(());
    }
    if !user_plugins.is_dir() {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            format!("cannot read plugin directory {}", user_plugins.display()),
        ));
    }
    for entry in fs::read_dir(user_plugins)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name();
        if !name.to_string_lossy().ends_with(".yazi") {
            continue;
        }
        if !path.is_dir() {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                format!("user plugin must be a directory: {}", path.display()),
            ));
        }
        let target = runtime_plugins.join(&name);
        if target.exists() {
            return Err(io::Error::new(
                ErrorKind::AlreadyExists,
                format!(
                    "user plugin `{}` collides with a packaged plugin",
                    name.to_string_lossy()
                ),
            ));
        }
        symlink(path, target)?;
    }
    Ok(())
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
    if command == "yzn-hx" {
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

fn remove_any(path: &Path) -> io::Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {
            fs::remove_dir_all(path)
        }
        Ok(_) => fs::remove_file(path),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
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
    fn yzn_hx_maps_to_packaged_editor_while_host_commands_pass_through() {
        assert_eq!(effective_editor_command("yzn-hx".to_string()), YZN_HELIX);
        assert_eq!(effective_editor_command("nvim".to_string()), "nvim");
    }

    fn packaged_yazi(path: &Path) {
        fs::create_dir_all(path.join("plugins")).unwrap();
        for file in [
            "init.lua",
            "keymap.toml",
            "yazi.toml",
            "yazelix_starship.toml",
        ] {
            fs::write(path.join(file), format!("packaged {file}\n")).unwrap();
        }
        for plugin in ["git.yazi", "sidebar-state.yazi", "zoxide-editor.yazi"] {
            fs::create_dir(path.join("plugins").join(plugin)).unwrap();
        }
    }

    #[test]
    fn materialization_overlays_user_plugins_and_rejects_collisions() {
        let temp = env::temp_dir().join(format!("yzn-yazi-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        let packaged = temp.join("packaged");
        let state = temp.join("state");
        let user_yazi = temp.join("config/yazi");
        let user_plugins = user_yazi.join("plugins");
        packaged_yazi(&packaged);
        fs::create_dir_all(user_plugins.join("example.yazi")).unwrap();
        fs::write(user_plugins.join("ignored.txt"), "").unwrap();
        fs::write(user_yazi.join("init.lua"), "user init\n").unwrap();

        let runtime = materialize_user_config(&state, &user_yazi, &packaged).unwrap();

        assert_eq!(
            fs::read_to_string(runtime.join("init.lua")).unwrap(),
            "packaged init.lua\n\n-- Yazelix Next user init.lua\nuser init\n"
        );
        for plugin in ["git.yazi", "sidebar-state.yazi", "zoxide-editor.yazi"] {
            assert!(
                fs::symlink_metadata(runtime.join("plugins").join(plugin))
                    .unwrap()
                    .file_type()
                    .is_symlink()
            );
        }
        assert!(
            fs::symlink_metadata(runtime.join("plugins/example.yazi"))
                .unwrap()
                .file_type()
                .is_symlink()
        );
        assert!(!runtime.join("plugins/ignored.txt").exists());

        fs::create_dir_all(user_plugins.join("git.yazi")).unwrap();
        let error = materialize_user_config(&state, &user_yazi, &packaged)
            .unwrap_err()
            .to_string();

        assert!(error.contains("collides with a packaged plugin"));
        let _ = fs::remove_dir_all(&temp);
    }

    #[test]
    fn materialization_layers_user_keymap_without_user_init() {
        let temp = env::temp_dir().join(format!("yzn-yazi-keymap-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp);
        let packaged = temp.join("packaged");
        let state = temp.join("state");
        let user_yazi = temp.join("config/yazi");
        packaged_yazi(&packaged);
        fs::create_dir_all(&user_yazi).unwrap();
        fs::write(user_yazi.join("keymap.toml"), "user keymap\n").unwrap();

        let runtime = materialize_user_config(&state, &user_yazi, &packaged).unwrap();

        assert_eq!(
            fs::read_to_string(runtime.join("keymap.toml")).unwrap(),
            "packaged keymap.toml\n\n# Yazelix Next user keymap.toml\nuser keymap\n"
        );
        assert_eq!(
            fs::read_to_string(runtime.join("init.lua")).unwrap(),
            "packaged init.lua\n"
        );
        assert!(!runtime.join("plugins/example.yazi").exists());
        let _ = fs::remove_dir_all(&temp);
    }
}
