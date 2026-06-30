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
    for (path, file, command) in [
        (&env_config, "env.nu", "source-env"),
        (&config, "config.nu", "source"),
    ] {
        write_layered_config(path, command, &packaged_nu.join(file), &user_nu.join(file))?;
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
    env::var_os("YAZELIX_NEXT_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            env::var_os("XDG_CONFIG_HOME").map(|path| PathBuf::from(path).join("yazelix-next"))
        })
        .or_else(|| {
            env::var_os("HOME").map(|path| PathBuf::from(path).join(".config/yazelix-next"))
        })
        .ok_or_else(|| io::Error::new(ErrorKind::NotFound, "HOME is required"))
}

fn state_dir() -> PathBuf {
    env::var_os("YAZELIX_STATE_DIR")
        .map(PathBuf::from)
        .or_else(|| {
            env::var_os("XDG_DATA_HOME").map(|path| PathBuf::from(path).join("yazelix-next"))
        })
        .or_else(|| {
            env::var_os("HOME").map(|path| PathBuf::from(path).join(".local/share/yazelix-next"))
        })
        .unwrap_or_else(|| env::temp_dir().join("yazelix-next"))
}

fn write_layered_config(
    path: &Path,
    command: &str,
    packaged: &Path,
    user: &Path,
) -> io::Result<()> {
    let mut contents = format!("{command} {}\n", nu_quote(packaged));
    if user.is_file() {
        contents.push_str(&format!("{command} {}\n", nu_quote(user)));
    }
    atomic_write(path, contents)
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
    match env::var_os("PATH") {
        Some(path) if !path.is_empty() => {
            let mut merged = OsString::from(PATH_PREFIX);
            merged.push(":");
            merged.push(path);
            merged
        }
        _ => PATH_PREFIX.into(),
    }
}
