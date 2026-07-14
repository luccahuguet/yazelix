use std::{
    env,
    ffi::OsString,
    fs, io,
    io::IsTerminal,
    os::unix::{fs::PermissionsExt, process::CommandExt},
    path::{Path, PathBuf},
    process::{Command, exit},
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
    let args = env::args_os().skip(1).collect::<Vec<_>>();
    let Some(state_dir) = state_dir() else {
        eprintln!("yzx-agent: HOME is required when YAZELIX_STATE_DIR and XDG_DATA_HOME are unset");
        return 1;
    };
    let provider_file = state_dir.join("agent/provider");

    if let Some(id) = read_provider(&provider_file) {
        return launch_configured(&id, &provider_file, &args);
    }

    for (provider, provider_args) in PROVIDERS.iter().copied() {
        if command_available(provider) {
            let _ = write_provider(&provider_file, provider);
            return exec_provider(provider, provider_args, &args);
        }
    }

    0
}

fn launch_configured(id: &str, provider_file: &Path, args: &[OsString]) -> i32 {
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

    exec_provider(provider, provider_args, args)
}

fn exec_provider(provider: &str, provider_args: &[&str], user_args: &[OsString]) -> i32 {
    let error = Command::new(provider)
        .args(provider_args)
        .args(user_args)
        .exec();
    eprintln!(
        "Yazelix Nova agent popup\n\nFailed to launch `{}`: {error}",
        provider
    );
    pause_if_tty();
    127
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

fn state_dir() -> Option<PathBuf> {
    nonempty_env("YAZELIX_STATE_DIR")
        .map(PathBuf::from)
        .or_else(|| nonempty_env("XDG_DATA_HOME").map(|path| PathBuf::from(path).join("yazelix")))
        .or_else(|| {
            nonempty_env("HOME").map(|path| PathBuf::from(path).join(".local/share/yazelix"))
        })
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
