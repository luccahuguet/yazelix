use std::{
    env,
    ffi::{OsStr, OsString},
    fs, io,
    io::{BufRead, IsTerminal, Write},
    os::unix::{fs::PermissionsExt, process::CommandExt},
    path::{Path, PathBuf},
    process::{exit, Command, Stdio},
};

const PROVIDERS: &[(&str, &[&str])] = &[
    ("codex", &["resume"]),
    ("grok", &[]),
    ("opencode", &[]),
    ("pi", &[]),
    ("claude", &["--resume"]),
];

fn main() {
    exit(run());
}

fn run() -> i32 {
    emit_initial_title();
    let state_dir = state_dir();
    let args = env::args_os().skip(1).collect::<Vec<_>>();
    if let Some((command, args)) = args.split_first() {
        return launch(command, args, &state_dir);
    }
    let provider_file = state_dir.join("agent/provider");

    if let Some(id) = read_provider(&provider_file) {
        return launch_configured(&id, &provider_file, &state_dir);
    }

    for (provider, provider_args) in PROVIDERS.iter().copied() {
        if command_available(provider) {
            let _ = write_provider(&provider_file, provider);
            return launch(OsStr::new(provider), provider_args, &state_dir);
        }
    }

    0
}

fn launch<T: AsRef<OsStr>>(command: &OsStr, args: &[T], state_dir: &Path) -> i32 {
    if Path::new(command).file_name() == Some(OsStr::new("codex")) && radar_enabled() {
        offer_codex_radar_setup(command, state_dir);
    }
    exec_command(command, args)
}

fn radar_enabled() -> bool {
    env::var_os("YZX_RADAR_ENABLED").as_deref() != Some(OsStr::new("false"))
}

fn emit_initial_title() {
    let mut stdout = io::stdout().lock();
    let _ = stdout.write_all(b"\x1b]0;agent popup\x07");
    let _ = stdout.flush();
}

fn exec_command<T: AsRef<OsStr>>(command: &OsStr, args: &[T]) -> i32 {
    let error = Command::new(command).args(args).exec();
    eprintln!(
        "Yazelix Nova agent popup\n\nFailed to launch `{}`: {error}",
        command.to_string_lossy()
    );
    pause_if_tty();
    127
}

fn launch_configured(id: &str, provider_file: &Path, state_dir: &Path) -> i32 {
    let Some((provider, provider_args)) = PROVIDERS
        .iter()
        .copied()
        .find(|(provider, _)| *provider == id)
    else {
        eprintln!(
            "Yazelix Nova agent popup\n\nConfigured agent provider `{id}` is unknown.\nRemove {} to let Yazelix choose again.",
            provider_file.display()
        );
        pause_if_tty();
        return 127;
    };

    if !command_available(provider) {
        eprintln!(
            "Yazelix Nova agent popup\n\nConfigured agent provider `{id}` is not available on PATH.\nInstall it or remove {} to let Yazelix choose again.",
            provider_file.display()
        );
        pause_if_tty();
        return 127;
    }

    launch(OsStr::new(provider), provider_args, state_dir)
}

fn offer_codex_radar_setup(codex: &OsStr, state_dir: &Path) {
    let marker = state_dir.join("agent/radar-codex-setup-offered");
    if marker.is_file() || !command_available("zj-radar") {
        return;
    }

    match radar_setup(codex, "--check", true) {
        Ok(status) if status.success() => {
            remember_radar_offer(&marker);
            return;
        }
        Ok(_) => {}
        Err(error) => {
            eprintln!(
                "Yazelix Nova: failed to check Radar's Codex hooks: {error}; Codex will still start."
            );
            return;
        }
    }

    if !io::stdin().is_terminal() || !io::stderr().is_terminal() {
        return;
    }
    eprint!(
        "Radar needs Codex hooks to show agent activity.\nYou can enable this later: yzx radar-setup\n\nEnable Codex activity in Radar? [Y/n] "
    );
    let _ = io::stderr().flush();
    let Some(install) = read_offer_consent(io::stdin().lock()) else {
        return;
    };
    remember_radar_offer(&marker);
    if install {
        match radar_setup(codex, "--yes", false) {
            Ok(status) if status.success() => {}
            Ok(status) => {
                eprintln!("Yazelix Nova: Radar setup failed with {status}; Codex will still start.")
            }
            Err(error) => eprintln!(
                "Yazelix Nova: failed to run Radar setup: {error}; Codex will still start."
            ),
        }
    }
}

fn read_offer_consent(mut input: impl BufRead) -> Option<bool> {
    let mut answer = String::new();
    if input.read_line(&mut answer).ok()? == 0 {
        return None;
    }
    match answer.trim().to_ascii_lowercase().as_str() {
        "" | "y" | "yes" => Some(true),
        "n" | "no" => Some(false),
        _ => None,
    }
}

fn radar_setup(codex: &OsStr, flag: &str, quiet: bool) -> io::Result<std::process::ExitStatus> {
    let mut command = Command::new("zj-radar");
    command.args(["setup", "codex", flag]);
    if let Some(parent) = Path::new(codex)
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
    {
        let mut path = env::var_os("PATH").unwrap_or_default();
        if !path.is_empty() {
            path.push(":");
        }
        path.push(parent);
        command.env("PATH", path);
    }
    if quiet {
        command.stdout(Stdio::null()).stderr(Stdio::null());
    }
    command.status()
}

fn remember_radar_offer(marker: &Path) {
    if let Err(error) = write_provider(marker, "1") {
        eprintln!(
            "Yazelix Nova: could not remember the Radar setup choice at {}: {error}",
            marker.display()
        );
    }
}

fn read_provider(path: &Path) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|text| text.trim().to_string())
        .filter(|id| !id.is_empty())
}

fn write_provider(path: &Path, id: &str) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, format!("{id}\n"))
}

fn command_available(command: &str) -> bool {
    let Some(path) = env::var_os("PATH").filter(|path| !path.is_empty()) else {
        return false;
    };

    env::split_paths(&path).any(|entry| is_executable(&entry.join(command)))
}

fn is_executable(path: &Path) -> bool {
    fs::metadata(path)
        .map(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
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

fn nonempty_env(name: &str) -> Option<OsString> {
    env::var_os(name).filter(|value| !value.is_empty())
}

fn pause_if_tty() {
    if io::stdin().is_terminal() {
        eprint!("\nPress Enter to close this popup...");
        let mut line = String::new();
        let _ = io::stdin().read_line(&mut line);
    }
}

#[cfg(test)]
mod tests {
    use super::read_offer_consent;

    #[test]
    fn radar_setup_offer_accepts_default_and_explicit_answers() {
        assert_eq!(read_offer_consent("".as_bytes()), None);
        assert_eq!(read_offer_consent("\n".as_bytes()), Some(true));
        assert_eq!(read_offer_consent("yes\n".as_bytes()), Some(true));
        assert_eq!(read_offer_consent("no\n".as_bytes()), Some(false));
        assert_eq!(read_offer_consent("later\n".as_bytes()), None);
    }
}
