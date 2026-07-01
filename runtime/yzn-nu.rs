use std::{
    env,
    ffi::OsString,
    fs,
    io::{self, ErrorKind},
    os::unix::process::CommandExt,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
    time::{SystemTime, UNIX_EPOCH},
};

const NU: &str = "@nu@";
const PACKAGED_NU: &str = "@packagedNu@";
const EMPTY_STARSHIP_CONFIG: &str = "/dev/null";
const PATH_PREFIX: &str = "@pathPrefix@";

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("yzn-nu: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> io::Result<()> {
    let config_home = config_home()?;
    let user_nu = config_home.join("nu");
    let user_starship = config_home.join("starship.toml");
    let starship_config = if user_starship.is_file() {
        user_starship
    } else {
        PathBuf::from(EMPTY_STARSHIP_CONFIG)
    };
    let packaged_nu = PathBuf::from(PACKAGED_NU);
    let runtime_nu = state_dir().join("nu");
    fs::create_dir_all(&runtime_nu)?;

    let env_config = runtime_nu.join("env.nu");
    let config = runtime_nu.join("config.nu");
    let mise_init = host_mise_init();
    for (path, file, command, after_packaged) in [
        (&env_config, "env.nu", "source-env", None),
        (&config, "config.nu", "source", mise_init.as_deref()),
    ] {
        write_layered_config(
            path,
            command,
            &packaged_nu.join(file),
            &user_nu.join(file),
            after_packaged,
        )?;
    }

    let error = Command::new(NU)
        .arg("--env-config")
        .arg(env_config)
        .arg("--config")
        .arg(config)
        .args(env::args_os().skip(1))
        .env("PATH", runtime_path())
        .env("STARSHIP_CONFIG", starship_config)
        .exec();
    Err(error)
}

fn config_home() -> io::Result<PathBuf> {
    nonempty_env("YAZELIX_NEXT_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            nonempty_env("XDG_CONFIG_HOME").map(|path| PathBuf::from(path).join("yazelix-next"))
        })
        .or_else(|| {
            nonempty_env("HOME").map(|path| PathBuf::from(path).join(".config/yazelix-next"))
        })
        .ok_or_else(|| io::Error::new(ErrorKind::NotFound, "HOME is required"))
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
        .unwrap_or_else(|| env::temp_dir().join("yazelix-next"))
}

fn write_layered_config(
    path: &Path,
    command: &str,
    packaged: &Path,
    user: &Path,
    after_packaged: Option<&str>,
) -> io::Result<()> {
    let mut contents = format!("{command} {}\n", nu_quote(packaged));
    if let Some(snippet) = after_packaged {
        contents.push_str(snippet);
        if !snippet.ends_with('\n') {
            contents.push('\n');
        }
    }
    if user.is_file() {
        contents.push_str(&format!("{command} {}\n", nu_quote(user)));
    }
    atomic_write(path, contents)
}

fn host_mise_init() -> Option<String> {
    let output = Command::new("mise")
        .arg("activate")
        .arg("nu")
        .env("PATH", runtime_path())
        .output()
        .ok()?;
    if output.status.success() {
        String::from_utf8(output.stdout)
            .ok()
            .filter(|text| !text.is_empty())
    } else {
        None
    }
}

fn nu_quote(path: &Path) -> String {
    let mut quoted = String::from("\"");
    for ch in path.as_os_str().to_string_lossy().chars() {
        match ch {
            '\\' => quoted.push_str("\\\\"),
            '"' => quoted.push_str("\\\""),
            _ => quoted.push(ch),
        }
    }
    quoted.push('"');
    quoted
}

fn atomic_write(path: &Path, contents: String) -> io::Result<()> {
    let tmp = path.with_extension(format!("tmp.{}.{}", std::process::id(), unix_nanos()));
    fs::write(&tmp, contents)?;
    fs::rename(tmp, path)
}

fn unix_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default()
}

fn runtime_path() -> OsString {
    match nonempty_env("PATH") {
        Some(path) => {
            let mut merged = OsString::from(PATH_PREFIX);
            merged.push(":");
            merged.push(path);
            merged
        }
        _ => PATH_PREFIX.into(),
    }
}

fn nonempty_env(name: &str) -> Option<OsString> {
    env::var_os(name).filter(|value| !value.is_empty())
}
