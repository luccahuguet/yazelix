use std::{
    env,
    ffi::{OsStr, OsString},
    fs, io,
    io::{IsTerminal, Write},
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
    emit_initial_title();
    let args = env::args_os().skip(1).collect::<Vec<_>>();
    if let Some((command, args)) = args.split_first() {
        return exec_command(command, args);
    }
    let Some(state_dir) = state_dir() else {
        eprintln!("yzx-agent: YAZELIX_STATE_DIR or XDG_RUNTIME_DIR is required");
        return 1;
    };
    let provider_file = state_dir.join("agent/provider");

    if let Some(id) = read_provider(&provider_file) {
        return launch_configured(&id, &provider_file);
    }

    for (provider, provider_args) in PROVIDERS.iter().copied() {
        if command_available(provider) {
            let _ = write_provider(&provider_file, provider);
            return exec_command(OsStr::new(provider), provider_args);
        }
    }

    0
}

fn emit_initial_title() {
    let mut stdout = io::stdout().lock();
    let _ = stdout.write_all(b"\x1b]0;agent\x07");
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

fn launch_configured(id: &str, provider_file: &Path) -> i32 {
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

    exec_command(OsStr::new(provider), provider_args)
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
        .or_else(|| {
            nonempty_env("XDG_RUNTIME_DIR").map(|path| PathBuf::from(path).join("yazelix"))
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
