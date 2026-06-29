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
        .map(|path| path.join("yazi/init.lua"))
        .filter(|path| path.exists())
    {
        Some(user_init) => materialize_user_config(&state_dir, &user_init)?,
        None => PathBuf::from(YZN_YAZI_CONFIG),
    };
    let yzn_open_log = yzn_config_value("open.log_level")?;
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
        .env("EDITOR", YZN_HELIX)
        .env("VISUAL", YZN_HELIX)
        .env("YZN_EDITOR", YZN_HELIX)
        .env("YZN_OPEN_LOG", yzn_open_log);

    if let Some(session) = nonempty_env("ZELLIJ_SESSION_NAME") {
        command
            .env("YAZELIX_ZELLIJ_SESSION_NAME", session)
            .env("ZELLIJ_SESSION_NAME", "")
            .env("KITTY_WINDOW_ID", "1");
    }

    Err(command.exec())
}

fn materialize_user_config(state_dir: &Path, user_init: &Path) -> io::Result<PathBuf> {
    if !user_init.is_file() {
        return Err(io::Error::new(
            ErrorKind::InvalidInput,
            format!("cannot read {}", user_init.display()),
        ));
    }

    let runtime_yazi = state_dir.join("yazi");
    let packaged_yazi = Path::new(YZN_YAZI_CONFIG);
    remove_any(&runtime_yazi)?;
    fs::create_dir_all(&runtime_yazi)?;
    for path in [
        "keymap.toml",
        "yazi.toml",
        "yazelix_starship.toml",
        "plugins",
    ] {
        symlink(packaged_yazi.join(path), runtime_yazi.join(path))?;
    }

    let mut init = fs::read_to_string(packaged_yazi.join("init.lua"))?;
    init.push_str("\n-- Yazelix Next user init.lua\n");
    init.push_str(&fs::read_to_string(user_init)?);
    let tmp = runtime_yazi.join(format!("init.lua.tmp.{}", std::process::id()));
    fs::write(&tmp, &init)?;
    fs::rename(tmp, runtime_yazi.join("init.lua"))?;
    Ok(runtime_yazi)
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
            nonempty_env("XDG_RUNTIME_DIR").map(|path| PathBuf::from(path).join("yazelix-next"))
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
